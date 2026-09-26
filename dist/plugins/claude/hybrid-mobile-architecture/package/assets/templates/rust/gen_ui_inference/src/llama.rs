// TJ-ARCH-MOB-001 compliant
//! Mobile CPU inference through llama.cpp. Model download is revision- and
//! SHA-256-pinned, atomic, and rooted in the platform app-data cache supplied by
//! the mobile leaf.

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::catalog::{
    artifact_set_for_id, ArtifactSpec, ANDROID_QWEN3_4B, ANDROID_QWEN3_4B_ID, IPHONE_QWEN35_9B_ID,
};
use crate::prompt::messages_with_tool_context;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{
    GrammarConstraint, InferenceMessage, InferenceProvider, InferenceRequest, LocalModelSpec,
    SampleParams,
};
use gen_ui_types::lifecycle::{ModelOperationState, RunError, RunPhase, RuntimeDiagnostics};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
#[cfg(feature = "local-llama-mtmd")]
use llama_cpp_2::mtmd::{
    mtmd_default_marker, MtmdBitmap, MtmdContext, MtmdContextParams, MtmdInputText,
};
use llama_cpp_2::sampling::LlamaSampler;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use std::io::Read;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use tokio::sync::RwLock;

const DEFAULT_CONTEXT: u32 = 8192;
const DEFAULT_MAX_MODEL_BYTES: u64 = 8 * 1024 * 1024 * 1024;

pub struct LlamaCppEngine {
    cache_dir: PathBuf,
    state: RwLock<Option<LoadedModel>>,
    diagnostics: RwLock<Option<RuntimeDiagnostics>>,
    model_operation: watch::Sender<ModelOperationState>,
    model_cancelled: AtomicBool,
    #[cfg(feature = "local-llama-mtmd")]
    multimodal: RwLock<Option<Arc<MtmdContext>>>,
}

struct LoadedModel {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    spec: LocalModelSpec,
    diagnostics: RuntimeDiagnostics,
}

impl LlamaCppEngine {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let (model_operation, _) = watch::channel(ModelOperationState::NotInstalled);
        Self {
            cache_dir: cache_dir.into(),
            state: RwLock::new(None),
            diagnostics: RwLock::new(None),
            model_operation,
            model_cancelled: AtomicBool::new(false),
            #[cfg(feature = "local-llama-mtmd")]
            multimodal: RwLock::new(None),
        }
    }

    pub async fn diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics.read().await.clone()
    }

    /// Ensure the projector paired with a catalog model is present and
    /// checksum-verified. The mtmd loader calls this after model selection so
    /// a mismatched projector can never be attached to a GGUF.
    pub async fn ensure_projector(&self, model_id: &str) -> CoreResult<PathBuf> {
        let set = artifact_set_for_id(model_id).ok_or_else(|| {
            CoreError::NotFound(format!(
                "unknown multimodal model '{model_id}' — no projector is cataloged"
            ))
        })?;
        let projector = set.projector.ok_or_else(|| {
            CoreError::NotFound(format!(
                "model '{model_id}' is text-only and has no cataloged projector"
            ))
        })?;
        preflight_artifact(projector)?;
        let path = self.cache_dir.join(projector.file_name);
        ensure_catalog_model(&path, projector).await?;
        Ok(path)
    }

    /// Download the paired projector and initialize llama.cpp's MTMD context
    /// against the currently loaded text model. The context is cached until
    /// unload, so each image turn does not rebuild the vision encoder.
    #[cfg(feature = "local-llama-mtmd")]
    pub async fn ensure_multimodal(&self, model_id: &str) -> CoreResult<()> {
        if self.multimodal.read().await.is_some() {
            return Ok(());
        }
        let projector = self.ensure_projector(model_id).await?;
        let model = {
            let state = self.state.read().await;
            let loaded = state
                .as_ref()
                .ok_or_else(|| terminal("multimodal inference", "call load before images"))?;
            if loaded.spec.model != model_id {
                return Err(terminal(
                    "multimodal inference",
                    format!(
                        "projector model {model_id} does not match loaded {}",
                        loaded.spec.model
                    ),
                ));
            }
            Arc::clone(&loaded.model)
        };
        let projector = projector.to_string_lossy().into_owned();
        let context = gen_ui_runtime::spawn_blocking(move || {
            MtmdContext::init_from_file(&projector, &model, &MtmdContextParams::default())
                .map_err(|error| terminal("mtmd init", error))
        })
        .await
        .map_err(|error| terminal("mtmd init task", error))??;
        if !context.support_vision() {
            return Err(terminal(
                "mtmd init",
                "projector does not expose vision support",
            ));
        }
        *self.multimodal.write().await = Some(Arc::new(context));
        Ok(())
    }
}

fn terminal(context: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Terminal(format!("{context}: {error}"))
}

fn resolve_model(
    spec: &LocalModelSpec,
    cache_dir: &Path,
) -> CoreResult<(PathBuf, Option<ArtifactSpec>)> {
    let path = PathBuf::from(&spec.model);
    if path.is_absolute() {
        return Ok((path, None));
    }
    if let Some(set) = artifact_set_for_id(&spec.model) {
        return Ok((cache_dir.join(set.model.file_name), Some(set.model)));
    }
    Err(CoreError::NotFound(format!(
        "unknown mobile model '{}' — expected a catalog id or an absolute GGUF path",
        spec.model,
    )))
}

async fn ensure_catalog_model(path: &Path, artifact: ArtifactSpec) -> CoreResult<()> {
    ensure_catalog_model_with_status(path, artifact, None, None).await
}

async fn ensure_catalog_model_with_status(
    path: &Path,
    artifact: ArtifactSpec,
    status: Option<&watch::Sender<ModelOperationState>>,
    cancelled: Option<&AtomicBool>,
) -> CoreResult<()> {
    if path.is_file() {
        if let Some(status) = status {
            let _ = status.send(ModelOperationState::Verifying);
        }
        if verify_artifact(path, artifact).await.is_ok() {
            return Ok(());
        }
        tokio::fs::remove_file(path)
            .await
            .map_err(|error| terminal("remove invalid model", error))?;
    }
    let parent = path
        .parent()
        .ok_or_else(|| terminal("model download", "cache path has no parent"))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| terminal("model cache create", error))?;
    let temporary = path.with_extension("gguf.part");
    let existing = tokio::fs::metadata(&temporary)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let mut received = existing;
    let started = Instant::now();
    if let Some(status) = status {
        let _ = status.send(ModelOperationState::Downloading {
            bytes_received: received,
            total_bytes: artifact.byte_len,
            bytes_per_second: 0,
            resumable: true,
        });
    }
    let mut request = reqwest::Client::new().get(artifact.url);
    if existing > 0 && existing < artifact.byte_len {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }
    let response = request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| CoreError::Transient(format!("model download: {error}")))?;
    let append = should_append_download(existing, artifact.byte_len, response.status());
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options
        .open(&temporary)
        .await
        .map_err(|error| terminal("model temporary file", error))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|error| CoreError::Transient(format!("model download stream: {error}")))?
    {
        if cancelled.is_some_and(|token| token.load(Ordering::Acquire)) {
            return Err(CoreError::Transient("model operation cancelled".into()));
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| terminal("model cache write", error))?;
        received = received.saturating_add(chunk.len() as u64);
        if let Some(status) = status {
            let elapsed = started.elapsed().as_secs().max(1);
            let _ = status.send(ModelOperationState::Downloading {
                bytes_received: received,
                total_bytes: artifact.byte_len,
                bytes_per_second: received.saturating_sub(existing) / elapsed,
                resumable: true,
            });
        }
    }
    file.flush()
        .await
        .map_err(|error| terminal("model cache flush", error))?;
    drop(file);
    if let Some(status) = status {
        let _ = status.send(ModelOperationState::Verifying);
    }
    if let Err(error) = verify_artifact(&temporary, artifact).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, path)
        .await
        .map_err(|error| terminal("model cache publish", error))?;
    tracing::info!(
        revision = artifact.revision,
        bytes = artifact.byte_len,
        path = %path.display(),
        "mobile model downloaded"
    );
    Ok(())
}

fn preflight_artifact(artifact: ArtifactSpec) -> CoreResult<()> {
    let max_bytes = std::env::var("__ENV_PREFIX___LOCAL_MODEL_MAX_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_MODEL_BYTES);
    if artifact.byte_len > max_bytes {
        return Err(CoreError::Transient(format!(
            "model {} is {} bytes, above the {} byte mobile preflight budget",
            artifact.file_name, artifact.byte_len, max_bytes
        )));
    }
    if crate::memory::preflight_rejects(gguf_peak_bytes(artifact)) {
        let budget = crate::memory::device_memory_budget_bytes().unwrap_or(0);
        return Err(CoreError::Transient(format!(
            "model {} needs ~{} bytes resident (artifact + {} bytes runtime headroom) but this device's safe budget is {} bytes; \
             select a smaller model",
            artifact.file_name,
            gguf_peak_bytes(artifact),
            RUNTIME_MEMORY_HEADROOM_BYTES,
            budget
        )));
    }
    Ok(())
}

/// Bytes to reserve beyond the model file for the KV cache, compute buffers,
/// and the rest of the app. Deliberately coarse — this gate exists to keep a
/// hopeless load from ever starting, not to predict peak usage precisely.
/// GGUF-specific: llama.cpp mmaps weights, so artifact bytes approximate the
/// resident working set; MLX's unified-memory profile uses its own estimate.
const RUNTIME_MEMORY_HEADROOM_BYTES: u64 = 1024 * 1024 * 1024;

/// Estimated peak residency for a GGUF artifact under llama.cpp.
fn gguf_peak_bytes(artifact: ArtifactSpec) -> u64 {
    artifact
        .byte_len
        .saturating_add(RUNTIME_MEMORY_HEADROOM_BYTES)
}

/// Whether this artifact is too large for the device, without producing an
/// error. Lets the caller choose a smaller model instead of failing outright.
fn memory_preflight_rejects(artifact: ArtifactSpec) -> bool {
    crate::memory::preflight_rejects(gguf_peak_bytes(artifact))
}

/// The next-smaller cataloged GGUF model that fits this device, if one exists.
fn downgrade_for_memory(spec: &LocalModelSpec) -> Option<LocalModelSpec> {
    // Ordered largest-to-smallest; pick the first that clears the budget.
    const FALLBACKS: &[&str] = &[ANDROID_QWEN3_4B_ID];
    crate::memory::downgrade_for_memory(spec, FALLBACKS, |id| {
        artifact_set_for_id(id).map(|set| gguf_peak_bytes(set.model))
    })
}

fn gpu_backend_name() -> &'static str {
    #[cfg(target_os = "ios")]
    {
        return "metal";
    }
    #[cfg(target_os = "android")]
    {
        return "opencl";
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        "gpu"
    }
}

fn should_append_download(existing: u64, expected: u64, status: StatusCode) -> bool {
    existing > 0 && existing < expected && status == StatusCode::PARTIAL_CONTENT
}

fn load_model_with_backend(
    backend: Arc<LlamaBackend>,
    path: PathBuf,
    spec: LocalModelSpec,
    fallback_reason: Option<String>,
    allow_gpu: bool,
) -> CoreResult<LoadedModel> {
    let supports_gpu = allow_gpu && backend.supports_gpu_offload();
    let attempts = if supports_gpu {
        vec![true, false]
    } else {
        vec![false]
    };
    let mut last_error = String::from("no model load attempt");
    for use_gpu in attempts {
        let params = if use_gpu {
            LlamaModelParams::default().with_n_gpu_layers(u32::MAX)
        } else {
            LlamaModelParams::default()
        };
        match LlamaModel::load_from_file(&backend, &path, &params) {
            Ok(model) => {
                let backend_name = if use_gpu {
                    gpu_backend_name().to_string()
                } else {
                    "cpu".to_string()
                };
                let mut reason = fallback_reason.clone();
                if supports_gpu && !use_gpu && reason.is_none() {
                    reason = Some("GPU model load failed; CPU fallback selected".to_string());
                }
                return Ok(LoadedModel {
                    backend,
                    model: Arc::new(model),
                    spec: spec.clone(),
                    diagnostics: RuntimeDiagnostics {
                        model_id: spec.model,
                        backend: backend_name,
                        device_name: None,
                        offloaded_layers: None,
                        fallback_reason: reason,
                        gate_report: Vec::new(),
                    },
                });
            }
            Err(error) => last_error = error.to_string(),
        }
    }
    Err(terminal("GGUF load", last_error))
}

fn gpu_is_release_eligible(cache_dir: &Path) -> bool {
    #[cfg(target_os = "android")]
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Certification {
            correct: bool,
            thermally_safe: bool,
            median_improvement: f32,
        }
        let path = cache_dir.join("opencl-certification.json");
        return std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Certification>(&bytes).ok())
            .is_some_and(|result| {
                result.correct && result.thermally_safe && result.median_improvement >= 0.15
            });
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = cache_dir;
        true
    }
}

async fn verify_artifact(path: &Path, artifact: ArtifactSpec) -> CoreResult<()> {
    let path = path.to_owned();
    gen_ui_runtime::spawn_blocking(move || {
        let metadata =
            std::fs::metadata(&path).map_err(|error| terminal("model checksum metadata", error))?;
        if metadata.len() != artifact.byte_len {
            return Err(terminal(
                "model byte length",
                format!("expected {}, got {}", artifact.byte_len, metadata.len()),
            ));
        }
        let mut file =
            std::fs::File::open(&path).map_err(|error| terminal("model checksum read", error))?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| terminal("model checksum read", error))?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        let actual = format!("{:x}", digest.finalize());
        if actual == artifact.sha256 {
            Ok(())
        } else {
            Err(terminal(
                "model checksum",
                format!("expected {}, got {actual}", artifact.sha256),
            ))
        }
    })
    .await
    .map_err(|error| terminal("model checksum task", error))?
}

#[async_trait]
impl InferenceProvider for LlamaCppEngine {
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
        if self
            .state
            .read()
            .await
            .as_ref()
            .is_some_and(|loaded| &loaded.spec == spec)
        {
            if let Some(diagnostics) = self.diagnostics().await {
                let _ = self
                    .model_operation
                    .send(ModelOperationState::Ready { diagnostics });
            }
            return Ok(());
        }
        self.model_cancelled.store(false, Ordering::Release);
        let (path, artifact) = resolve_model(spec, &self.cache_dir)?;

        // A model that cannot fit this device's memory budget must be swapped
        // out BEFORE the download and load, not after: on iOS an over-budget
        // load is terminated by jetsam, which is a process kill rather than an
        // `Err`, so the post-load fallback below would never run. Downgrading
        // here also avoids spending a multi-gigabyte download on an artifact
        // that could never have been loaded.
        let (spec, path, artifact) = match artifact {
            Some(artifact) if memory_preflight_rejects(artifact) => {
                match downgrade_for_memory(spec) {
                    Some(smaller) => {
                        tracing::warn!(
                            target: "__APP_NAME___llama",
                            requested = %spec.model,
                            selected = %smaller.model,
                            "requested local model exceeds this device's memory budget; using a smaller model"
                        );
                        let (smaller_path, smaller_artifact) =
                            resolve_model(&smaller, &self.cache_dir)?;
                        (
                            std::borrow::Cow::Owned(smaller),
                            smaller_path,
                            smaller_artifact,
                        )
                    }
                    // No smaller option: surface the budget error rather than
                    // starting a load this device cannot survive.
                    None => {
                        preflight_artifact(artifact)?;
                        (std::borrow::Cow::Borrowed(spec), path, Some(artifact))
                    }
                }
            }
            other => (std::borrow::Cow::Borrowed(spec), path, other),
        };
        let spec = spec.as_ref();

        if let Some(artifact) = artifact {
            preflight_artifact(artifact)?;
            if let Err(error) = ensure_catalog_model_with_status(
                &path,
                artifact,
                Some(&self.model_operation),
                Some(&self.model_cancelled),
            )
            .await
            {
                if self.model_cancelled.load(Ordering::Acquire) {
                    let _ = self.model_operation.send(ModelOperationState::Cancelled);
                } else {
                    let _ = self.model_operation.send(ModelOperationState::Failed {
                        error: RunError {
                            code: "model_prepare_failed".into(),
                            phase: RunPhase::ModelDownload,
                            message: error.to_string(),
                            retryable: true,
                            diagnostics: None,
                        },
                    });
                }
                return Err(error);
            }
        } else if !path.is_file() {
            return Err(CoreError::NotFound(format!(
                "GGUF model does not exist: {}",
                path.display()
            )));
        }
        let _ = self.model_operation.send(ModelOperationState::Loading);
        let backend =
            Arc::new(LlamaBackend::init().map_err(|error| terminal("llama backend init", error))?);
        let allow_gpu = gpu_is_release_eligible(&self.cache_dir);
        #[cfg(target_os = "android")]
        let backend_policy_reason = (!allow_gpu).then(|| {
            "CPU selected: OpenCL requires a physical-device certification showing correctness, thermal safety, and at least 15% median improvement".to_string()
        });
        #[cfg(not(target_os = "android"))]
        let backend_policy_reason = None;
        let loaded_spec = spec.clone();
        let primary_result = gen_ui_runtime::spawn_blocking({
            let backend = Arc::clone(&backend);
            let path = path.clone();
            move || {
                load_model_with_backend(
                    backend,
                    path,
                    loaded_spec,
                    backend_policy_reason,
                    allow_gpu,
                )
            }
        })
        .await
        .map_err(|error| terminal("GGUF load task", error))?;
        let primary_model = spec.model.clone();
        let loaded = match primary_result {
            Ok(loaded) => loaded,
            Err(primary_error) if spec.model == IPHONE_QWEN35_9B_ID => {
                let fallback = ANDROID_QWEN3_4B;
                preflight_artifact(fallback.model)?;
                let fallback_path = self.cache_dir.join(fallback.model.file_name);
                ensure_catalog_model(&fallback_path, fallback.model).await?;
                let fallback_spec = LocalModelSpec {
                    model: ANDROID_QWEN3_4B_ID.to_string(),
                    context_len: spec.context_len,
                };
                gen_ui_runtime::spawn_blocking({
                    let backend = Arc::clone(&backend);
                    move || {
                        load_model_with_backend(
                            backend,
                            fallback_path,
                            fallback_spec,
                            Some(format!(
                                "primary model {} failed: {primary_error}",
                                primary_model
                            )),
                            allow_gpu,
                        )
                    }
                })
                .await
                .map_err(|error| terminal("fallback GGUF load task", error))??
            }
            Err(error) => return Err(error),
        };
        if self.model_cancelled.load(Ordering::Acquire) {
            // llama.cpp model loading is not safely interruptible on every
            // backend. Honour cancellation at the first safe boundary and
            // drop the newly loaded handles before they become observable.
            drop(loaded);
            let _ = self.model_operation.send(ModelOperationState::Cancelled);
            return Err(CoreError::Cancelled(
                "local model loading was cancelled".into(),
            ));
        }
        *self.diagnostics.write().await = Some(loaded.diagnostics.clone());
        let _ = self.model_operation.send(ModelOperationState::Ready {
            diagnostics: loaded.diagnostics.clone(),
        });
        *self.state.write().await = Some(loaded);
        Ok(())
    }

    async fn generate_request(
        &self,
        request: &InferenceRequest,
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        if request.images.is_empty() {
            let (backend, model, context_len) = {
                let state = self.state.read().await;
                let loaded = state
                    .as_ref()
                    .ok_or_else(|| terminal("mobile inference", "call load before generate"))?;
                (
                    Arc::clone(&loaded.backend),
                    Arc::clone(&loaded.model),
                    loaded.spec.context_len.unwrap_or(DEFAULT_CONTEXT),
                )
            };
            let messages = messages_with_tool_context(request)?;
            let params = request.generation_settings.clone();
            let grammar = request.grammar.clone();
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            gen_ui_runtime::spawn_blocking(move || {
                if let Err(error) = generate_blocking(
                    &backend,
                    &model,
                    &messages,
                    context_len,
                    &params,
                    grammar.as_ref(),
                    &sender,
                ) {
                    let _ = sender.send(StreamEvent::Error {
                        message: error.to_string(),
                    });
                }
            });
            return Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed());
        }
        #[cfg(not(feature = "local-llama-mtmd"))]
        {
            return Err(terminal(
                "multimodal inference",
                "this build is text-only; process attachments through Xberg/OCR",
            ));
        }
        #[cfg(feature = "local-llama-mtmd")]
        {
            if request.images.len() > 4 {
                return Err(terminal(
                    "multimodal inference",
                    "at most four images are supported per turn",
                ));
            }
            if request
                .images
                .iter()
                .any(|image| image.bytes.len() > 25 * 1024 * 1024)
            {
                return Err(terminal(
                    "multimodal inference",
                    "an image exceeds the 25 MB limit",
                ));
            }
            let model_id = {
                let state = self.state.read().await;
                let loaded = state
                    .as_ref()
                    .ok_or_else(|| terminal("multimodal inference", "call load before generate"))?;
                loaded.spec.model.clone()
            };
            self.ensure_multimodal(&model_id).await?;
            let (backend, model, mtmd, context_len) = {
                let state = self.state.read().await;
                let loaded = state
                    .as_ref()
                    .ok_or_else(|| terminal("multimodal inference", "call load before generate"))?;
                let mtmd = self
                    .multimodal
                    .read()
                    .await
                    .as_ref()
                    .ok_or_else(|| terminal("multimodal inference", "MTMD is not initialized"))
                    .map(Arc::clone)?;
                (
                    Arc::clone(&loaded.backend),
                    Arc::clone(&loaded.model),
                    mtmd,
                    loaded.spec.context_len.unwrap_or(DEFAULT_CONTEXT),
                )
            };
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            let request = request.clone();
            let params = request.generation_settings.clone();
            gen_ui_runtime::spawn_blocking(move || {
                if let Err(error) = generate_multimodal_blocking(
                    &backend,
                    &model,
                    &mtmd,
                    &request,
                    context_len,
                    &params,
                    &sender,
                ) {
                    let _ = sender.send(StreamEvent::Error {
                        message: error.to_string(),
                    });
                }
            });
            Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed())
        }
    }

    async fn unload(&self) -> CoreResult<()> {
        *self.state.write().await = None;
        #[cfg(feature = "local-llama-mtmd")]
        {
            *self.multimodal.write().await = None;
        }
        Ok(())
    }

    fn model_operation_state(&self) -> ModelOperationState {
        self.model_operation.borrow().clone()
    }

    fn watch_model_operation(&self) -> futures::stream::BoxStream<'static, ModelOperationState> {
        let receiver = self.model_operation.subscribe();
        futures::stream::unfold((receiver, true), |(mut receiver, first)| async move {
            if !first && receiver.changed().await.is_err() {
                return None;
            }
            let value = receiver.borrow().clone();
            Some((value, (receiver, false)))
        })
        .boxed()
    }

    fn cancel_model_operation(&self) {
        self.model_cancelled.store(true, Ordering::Release);
        let _ = self.model_operation.send(ModelOperationState::Cancelling);
    }

    async fn runtime_diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics().await
    }
}

fn generate_blocking(
    backend: &LlamaBackend,
    model: &LlamaModel,
    messages: &[InferenceMessage],
    context_len: u32,
    params: &SampleParams,
    grammar: Option<&GrammarConstraint>,
    sender: &tokio::sync::mpsc::UnboundedSender<StreamEvent>,
) -> CoreResult<()> {
    let context_len = NonZeroU32::new(context_len)
        .ok_or_else(|| terminal("mobile inference", "context length cannot be zero"))?;
    let context_params = LlamaContextParams::default().with_n_ctx(Some(context_len));
    let mut context = model
        .new_context(backend, context_params)
        .map_err(|error| terminal("llama context", error))?;
    let template = model
        .chat_template(None)
        .map_err(|error| terminal("chat template", error))?;
    let chat_messages = messages
        .iter()
        .map(|message| {
            LlamaChatMessage::new(message.role.as_str().to_string(), message.content.clone())
                .map_err(|error| terminal("chat message", error))
        })
        .collect::<CoreResult<Vec<_>>>()?;
    let formatted = model
        .apply_chat_template(&template, &chat_messages, true)
        .map_err(|error| terminal("apply chat template", error))?;
    let tokens = model
        .str_to_token(&formatted, AddBos::Never)
        .map_err(|error| terminal("prompt tokenize", error))?;
    if tokens.len() + params.max_tokens as usize > context_len.get() as usize {
        return Err(terminal(
            "mobile inference",
            "prompt plus requested output exceeds the context length",
        ));
    }
    let mut batch = LlamaBatch::new(tokens.len(), 1);
    let last = tokens.len().saturating_sub(1);
    for (position, token) in tokens.into_iter().enumerate() {
        batch
            .add(token, position as i32, &[0], position == last)
            .map_err(|error| terminal("prompt batch", error))?;
    }
    context
        .decode(&mut batch)
        .map_err(|error| terminal("prompt decode", error))?;
    let mut sampler_chain = vec![
        LlamaSampler::top_k(params.top_k),
        LlamaSampler::top_p(params.top_p, 1),
        LlamaSampler::min_p(params.min_p, 1),
        LlamaSampler::penalties(-1, 1.0, 0.0, params.presence_penalty),
        LlamaSampler::temp(params.temperature),
        LlamaSampler::dist(0xC0FFEE),
    ];
    if let Some(grammar) = grammar {
        sampler_chain.insert(
            0,
            LlamaSampler::llguidance(model, &grammar.kind, &grammar.data)
                .map_err(|error| terminal("llguidance grammar", error))?,
        );
    }
    let mut sampler = LlamaSampler::chain_simple(sampler_chain);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    for (position, index) in (batch.n_tokens()..).zip(0..params.max_tokens) {
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        let delta = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|error| terminal("token decode", error))?;
        if !delta.is_empty()
            && sender
                .send(StreamEvent::TextDelta { index, delta })
                .is_err()
        {
            return Ok(());
        }
        batch.clear();
        batch
            .add(token, position, &[0], true)
            .map_err(|error| terminal("token batch", error))?;
        context
            .decode(&mut batch)
            .map_err(|error| terminal("token decode step", error))?;
    }
    let _ = sender.send(StreamEvent::Done);
    Ok(())
}

#[cfg(feature = "local-llama-mtmd")]
fn generate_multimodal_blocking(
    backend: &LlamaBackend,
    model: &LlamaModel,
    mtmd: &MtmdContext,
    request: &InferenceRequest,
    context_len: u32,
    params: &SampleParams,
    sender: &tokio::sync::mpsc::UnboundedSender<StreamEvent>,
) -> CoreResult<()> {
    let context_len = NonZeroU32::new(context_len)
        .ok_or_else(|| terminal("multimodal inference", "context length cannot be zero"))?;
    let context_params = LlamaContextParams::default().with_n_ctx(Some(context_len));
    let mut context = model
        .new_context(backend, context_params)
        .map_err(|error| terminal("multimodal context", error))?;
    let bitmaps = request
        .images
        .iter()
        .map(|image| {
            MtmdBitmap::from_buffer(mtmd, &image.bytes, false)
                .map_err(|error| terminal("image decode", error))
        })
        .collect::<CoreResult<Vec<_>>>()?;
    let bitmap_refs = bitmaps.iter().collect::<Vec<_>>();
    let input = MtmdInputText {
        text: format!("{}\n{}", mtmd_default_marker(), request.prompt),
        add_special: true,
        parse_special: true,
    };
    let chunks = mtmd
        .tokenize(input, &bitmap_refs)
        .map_err(|error| terminal("image tokenize", error))?;
    let mut n_past = chunks
        .eval_chunks(mtmd, &context, 0, 0, 512, true)
        .map_err(|error| terminal("image embedding", error))?;
    if n_past as u32 + params.max_tokens > context_len.get() {
        return Err(terminal(
            "multimodal inference",
            "image prompt plus requested output exceeds the context length",
        ));
    }
    let mut sampler_chain = vec![
        LlamaSampler::top_k(params.top_k),
        LlamaSampler::top_p(params.top_p, 1),
        LlamaSampler::min_p(params.min_p, 1),
        LlamaSampler::penalties(-1, 1.0, 0.0, params.presence_penalty),
        LlamaSampler::temp(params.temperature),
        LlamaSampler::dist(0xC0FFEE),
    ];
    if let Some(grammar) = &request.grammar {
        sampler_chain.insert(
            0,
            LlamaSampler::llguidance(model, &grammar.kind, &grammar.data)
                .map_err(|error| terminal("llguidance grammar", error))?,
        );
    }
    let mut sampler = LlamaSampler::chain_simple(sampler_chain);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut batch = LlamaBatch::new(1, 1);
    for index in 0..params.max_tokens {
        let token = sampler.sample(&context, -1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        let delta = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|error| terminal("token decode", error))?;
        if !delta.is_empty()
            && sender
                .send(StreamEvent::TextDelta { index, delta })
                .is_err()
        {
            return Ok(());
        }
        batch.clear();
        batch
            .add(token, n_past, &[0], true)
            .map_err(|error| terminal("token batch", error))?;
        context
            .decode(&mut batch)
            .map_err(|error| terminal("token decode step", error))?;
        n_past += 1;
    }
    let _ = sender.send(StreamEvent::Done);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ANDROID_QWEN3_4B_ID, IPHONE_QWEN35_9B_ID};

    #[test]
    fn resolves_each_catalog_model_to_its_pinned_file() {
        let temp = Path::new("/tmp/gen-ui-model-cache");
        let (iphone, iphone_artifact) = resolve_model(
            &LocalModelSpec {
                model: IPHONE_QWEN35_9B_ID.to_string(),
                context_len: Some(8192),
            },
            temp,
        )
        .expect("iPhone catalog model should resolve");
        assert!(iphone.ends_with("Qwen3.5-9B-Q4_K_M.gguf"));
        assert_eq!(iphone_artifact.expect("artifact").byte_len, 5_680_522_464);

        let (android, android_artifact) = resolve_model(
            &LocalModelSpec {
                model: ANDROID_QWEN3_4B_ID.to_string(),
                context_len: Some(8192),
            },
            temp,
        )
        .expect("Android catalog model should resolve");
        assert!(android.ends_with("Qwen_Qwen3-4B-Instruct-2507-Q5_K_M.gguf"));
        assert_eq!(android_artifact.expect("artifact").byte_len, 2_889_513_696);
    }

    #[test]
    fn range_download_only_appends_on_partial_response() {
        assert!(should_append_download(10, 100, StatusCode::PARTIAL_CONTENT));
        assert!(!should_append_download(10, 100, StatusCode::OK));
        assert!(!should_append_download(
            100,
            100,
            StatusCode::PARTIAL_CONTENT
        ));
        assert!(!should_append_download(0, 100, StatusCode::PARTIAL_CONTENT));
    }

    #[test]
    fn preflight_rejects_artifacts_above_configured_default_budget() {
        let oversized = ArtifactSpec {
            byte_len: DEFAULT_MAX_MODEL_BYTES + 1,
            ..crate::catalog::ANDROID_QWEN3_4B.model
        };
        assert!(preflight_artifact(oversized).is_err());
    }

    /// Run `body` with `__ENV_PREFIX___DEVICE_MEMORY_BUDGET_BYTES` set, restoring the
    /// prior value afterward. Serialized because env vars are process-global.
    fn with_memory_budget(bytes: u64, body: impl FnOnce()) {
        use std::sync::Mutex;
        static GUARD: Mutex<()> = Mutex::new(());
        let _lock = GUARD
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = "__ENV_PREFIX___DEVICE_MEMORY_BUDGET_BYTES";
        let previous = std::env::var(key).ok();
        std::env::set_var(key, bytes.to_string());
        body();
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn memory_budget_override_gates_the_9b_model() {
        // 6 GB budget: the 5.68 GB Qwen 9B + 1 GB headroom overflows it.
        with_memory_budget(6 * 1024 * 1024 * 1024, || {
            assert!(
                memory_preflight_rejects(crate::catalog::IPHONE_QWEN35_9B.model),
                "9B + headroom must exceed a 6 GB budget"
            );
            // The 2.89 GB 4B model + 1 GB headroom fits.
            assert!(
                !memory_preflight_rejects(crate::catalog::ANDROID_QWEN3_4B.model),
                "4B + headroom must fit inside a 6 GB budget"
            );
        });
    }

    #[test]
    fn downgrade_picks_the_smaller_model_when_the_9b_is_too_large() {
        with_memory_budget(6 * 1024 * 1024 * 1024, || {
            let downgraded = downgrade_for_memory(&LocalModelSpec {
                model: IPHONE_QWEN35_9B_ID.to_string(),
                context_len: Some(4096),
            })
            .expect("a smaller cataloged model should be selected");
            assert_eq!(downgraded.model, ANDROID_QWEN3_4B_ID);
            assert_eq!(
                downgraded.context_len,
                Some(4096),
                "context_len is preserved"
            );
        });
    }

    #[test]
    fn downgrade_returns_none_when_no_option_fits() {
        // 1 GB budget: even the 4B fallback (2.89 GB) cannot fit.
        with_memory_budget(1024 * 1024 * 1024, || {
            assert!(downgrade_for_memory(&LocalModelSpec {
                model: IPHONE_QWEN35_9B_ID.to_string(),
                context_len: None,
            })
            .is_none());
        });
    }

    #[test]
    fn downgrade_returns_none_for_non_catalog_specs() {
        with_memory_budget(1, || {
            assert!(downgrade_for_memory(&LocalModelSpec {
                model: "/abs/path/to/custom.gguf".to_string(),
                context_len: None,
            })
            .is_none());
        });
    }

    #[test]
    fn generous_budget_rejects_nothing() {
        with_memory_budget(64 * 1024 * 1024 * 1024, || {
            assert!(!memory_preflight_rejects(
                crate::catalog::IPHONE_QWEN35_9B.model
            ));
            assert!(downgrade_for_memory(&LocalModelSpec {
                model: IPHONE_QWEN35_9B_ID.to_string(),
                context_len: None,
            })
            .is_none());
        });
    }
}
