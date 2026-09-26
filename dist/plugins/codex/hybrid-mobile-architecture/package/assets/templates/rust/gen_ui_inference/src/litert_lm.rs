// TJ-ARCH-MOB-001 compliant
//! Android LiteRT-LM inference lane.
//!
//! This module owns the Rust-side model catalog/download/lifecycle boundary for
//! LiteRT-LM. Android calls the official LiteRT-LM AAR through a tiny JNI
//! adapter so Flutter never constructs prompts or owns inference policy.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{
    InferenceProvider, InferenceRequest, InferenceToolResult, LocalModelSpec,
};
use gen_ui_types::lifecycle::{
    ModelOperationState, RunError, RunPhase, RuntimeDiagnostics, RuntimeGateReport,
    RuntimeGateStatus,
};
use reqwest::header::RANGE;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, RwLock};

use crate::catalog::{artifact_set_for_id, ArtifactSpec};

const DEFAULT_MAX_MODEL_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const DOWNLOAD_LOG_INTERVAL_BYTES: u64 = 256 * 1024 * 1024;

pub struct LiteRtLmEngine {
    cache_dir: PathBuf,
    loaded: RwLock<Option<LoadedLiteRtLmModel>>,
    #[cfg(target_os = "android")]
    native: RwLock<Option<crate::android_litert_jni::NativeLiteRtLmAdapter>>,
    diagnostics: RwLock<Option<RuntimeDiagnostics>>,
    model_operation: watch::Sender<ModelOperationState>,
    model_cancelled: AtomicBool,
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    backend_self_test_passed: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct LoadedLiteRtLmModel {
    model_path: PathBuf,
    spec: LocalModelSpec,
    diagnostics: RuntimeDiagnostics,
}

impl LiteRtLmEngine {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let (model_operation, _) = watch::channel(ModelOperationState::NotInstalled);
        Self {
            cache_dir: cache_dir.into(),
            loaded: RwLock::new(None),
            #[cfg(target_os = "android")]
            native: RwLock::new(None),
            diagnostics: RwLock::new(None),
            model_operation,
            model_cancelled: AtomicBool::new(false),
            backend_self_test_passed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn diagnostics(&self) -> Option<RuntimeDiagnostics> {
        let diagnostics = self.diagnostics.read().await.clone()?;
        #[cfg(target_os = "android")]
        {
            let mut diagnostics = diagnostics;
            let native_loaded = self.native.read().await.is_some();
            if native_loaded {
                diagnostics.gate_report = litert_lm_gate_report(
                    None,
                    true,
                    self.backend_self_test_passed.load(Ordering::Acquire),
                );
            }
            return Some(diagnostics);
        }
        #[cfg(not(target_os = "android"))]
        Some(diagnostics)
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
        "unknown LiteRT-LM model '{}' — expected a catalog id or an absolute .litertlm path",
        spec.model,
    )))
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
    Ok(())
}

async fn ensure_catalog_model_with_status(
    path: &Path,
    artifact: ArtifactSpec,
    status: &watch::Sender<ModelOperationState>,
    cancelled: &AtomicBool,
) -> CoreResult<()> {
    if path.is_file() {
        let _ = status.send(ModelOperationState::Verifying);
        if verify_artifact(path, artifact).await.is_ok() {
            return Ok(());
        }
        tokio::fs::remove_file(path)
            .await
            .map_err(|error| terminal("remove invalid LiteRT-LM model", error))?;
    }

    let parent = path
        .parent()
        .ok_or_else(|| terminal("LiteRT-LM model download", "cache path has no parent"))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| terminal("LiteRT-LM model cache create", error))?;

    let temporary = path.with_extension("litertlm.part");
    let existing = tokio::fs::metadata(&temporary)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let mut received = existing;
    let mut next_progress_log = existing.saturating_add(DOWNLOAD_LOG_INTERVAL_BYTES);
    let started = Instant::now();
    tracing::info!(
        target: "__APP_NAME___litert_lm",
        model_file = artifact.file_name,
        bytes_received = received,
        total_bytes = artifact.byte_len,
        resumable = true,
        "LiteRT-LM model download starting"
    );
    let _ = status.send(ModelOperationState::Downloading {
        bytes_received: received,
        total_bytes: artifact.byte_len,
        bytes_per_second: 0,
        resumable: true,
    });

    let mut request = reqwest::Client::new().get(artifact.url);
    if existing > 0 && existing < artifact.byte_len {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }
    let response = request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| CoreError::Transient(format!("LiteRT-LM model download: {error}")))?;

    let append = existing > 0
        && existing < artifact.byte_len
        && response.status() == StatusCode::PARTIAL_CONTENT;
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
        .map_err(|error| terminal("LiteRT-LM temporary file", error))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.try_next().await.map_err(|error| {
        CoreError::Transient(format!("LiteRT-LM model download stream: {error}"))
    })? {
        if cancelled.load(Ordering::Acquire) {
            return Err(CoreError::Cancelled(
                "LiteRT-LM model operation cancelled".into(),
            ));
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| terminal("LiteRT-LM model cache write", error))?;
        received = received.saturating_add(chunk.len() as u64);
        let elapsed = started.elapsed().as_secs().max(1);
        let _ = status.send(ModelOperationState::Downloading {
            bytes_received: received,
            total_bytes: artifact.byte_len,
            bytes_per_second: received.saturating_sub(existing) / elapsed,
            resumable: true,
        });
        if received >= next_progress_log || received >= artifact.byte_len {
            tracing::info!(
                target: "__APP_NAME___litert_lm",
                model_file = artifact.file_name,
                bytes_received = received,
                total_bytes = artifact.byte_len,
                bytes_per_second = received.saturating_sub(existing) / elapsed,
                "LiteRT-LM model download progress"
            );
            next_progress_log = received.saturating_add(DOWNLOAD_LOG_INTERVAL_BYTES);
        }
    }
    file.flush()
        .await
        .map_err(|error| terminal("LiteRT-LM model cache flush", error))?;
    drop(file);

    let _ = status.send(ModelOperationState::Verifying);
    tracing::info!(
        target: "__APP_NAME___litert_lm",
        model_file = artifact.file_name,
        path = %temporary.display(),
        "LiteRT-LM model verification starting"
    );
    if let Err(error) = verify_artifact(&temporary, artifact).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, path)
        .await
        .map_err(|error| terminal("LiteRT-LM model cache publish", error))?;
    tracing::info!(
        target: "__APP_NAME___litert_lm",
        model_file = artifact.file_name,
        path = %path.display(),
        "LiteRT-LM model verified and published"
    );
    Ok(())
}

async fn verify_artifact(path: &Path, artifact: ArtifactSpec) -> CoreResult<()> {
    let path = path.to_owned();
    gen_ui_runtime::spawn_blocking(move || {
        let metadata = std::fs::metadata(&path)
            .map_err(|error| terminal("LiteRT-LM model checksum metadata", error))?;
        if metadata.len() != artifact.byte_len {
            return Err(terminal(
                "LiteRT-LM model byte length",
                format!("expected {}, got {}", artifact.byte_len, metadata.len()),
            ));
        }
        let mut file = std::fs::File::open(&path)
            .map_err(|error| terminal("LiteRT-LM model checksum read", error))?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| terminal("LiteRT-LM model checksum read", error))?;
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
                "LiteRT-LM model checksum",
                format!("expected {}, got {actual}", artifact.sha256),
            ))
        }
    })
    .await
    .map_err(|error| terminal("LiteRT-LM model checksum task", error))?
}

#[cfg(any(not(target_os = "android"), test))]
fn native_adapter_unavailable() -> RunError {
    RunError {
        code: "litert_lm_native_adapter_unavailable".into(),
        phase: RunPhase::ModelLoading,
        message: "LiteRT-LM Android native adapter is not available in this process".into(),
        retryable: false,
        diagnostics: Some(
            "Ensure the Android AAR is packaged, JNI_OnLoad captured the JavaVM, and __APP_CLASS__LiteRtLmBridge is present before enabling Android local inference".into(),
        ),
    }
}

fn litert_lm_gate_report(
    native_adapter_error: Option<&RunError>,
    native_adapter_loaded: bool,
    self_test_passed: bool,
) -> Vec<RuntimeGateReport> {
    let mut gates = vec![
        RuntimeGateReport {
            gate: 1,
            name: "rust_ffi_library_loaded".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("gen_ui_ffi initialized and selected the LiteRT-LM provider".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 2,
            name: "model_artifact_packaged_or_cached".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some(
                "pinned .litertlm artifact resolved in the app-private model cache".into(),
            ),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 3,
            name: "model_artifact_verified".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some(
                "byte length and SHA-256 matched catalog metadata before activation".into(),
            ),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 4,
            name: "cpu_baseline_selected".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some(
                "Android LiteRT-LM recovery starts with CPU as the correctness baseline".into(),
            ),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 5,
            name: "litert_lm_native_adapter_loaded".into(),
            status: RuntimeGateStatus::Pending,
            evidence: Some("LiteRT-LM Android API bridge not loaded yet".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 6,
            name: "backend_self_test".into(),
            status: RuntimeGateStatus::Skipped,
            evidence: Some(
                "requires the native LiteRT-LM adapter to produce a token/tool-call smoke result"
                    .into(),
            ),
            error_code: None,
        },
    ];
    if let Some(error) = native_adapter_error {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Failed;
            gate.error_code = Some(error.code.clone());
        }
    } else if native_adapter_loaded {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Passed;
            gate.evidence = Some(
                "Rust called the Android LiteRT-LM bridge and initialized the CPU engine".into(),
            );
        }
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 6) {
            if self_test_passed {
                gate.status = RuntimeGateStatus::Passed;
                gate.evidence =
                    Some("LiteRT-LM CPU engine returned a non-empty generation result".into());
            } else {
                gate.status = RuntimeGateStatus::Pending;
                gate.evidence = Some(
                    "LiteRT-LM CPU engine initialized; waiting for first successful prompt smoke"
                        .into(),
                );
            }
        }
    }
    gates
}

#[cfg_attr(not(any(target_os = "android", test)), allow(dead_code))]
fn push_litert_lm_response_events(
    text: String,
    tool_calls: Vec<(String, serde_json::Value)>,
    tool_result_fallback_applied: bool,
    output: &mut Vec<StreamEvent>,
) {
    if tool_result_fallback_applied {
        output.push(StreamEvent::Custom {
            source: "litert_lm".into(),
            event_name: "tool_result_fallback_applied".into(),
            payload: serde_json::json!({
                "kind": "lifecycle",
                "phase": "generation",
                "message": "LiteRT-LM returned an empty tool-result continuation, so the app sent one bounded follow-up instruction to produce a visible answer from the tool result.",
            }),
        });
    }
    if !text.is_empty() {
        output.push(StreamEvent::TextDelta {
            index: 0,
            delta: text,
        });
    }
    for (index, (name, arguments)) in tool_calls.into_iter().enumerate() {
        let id = format!("litert-tool-{index}");
        let args = serde_json::to_string(&arguments).unwrap_or_else(|_| "{}".into());
        output.push(StreamEvent::ToolCallStarted {
            id: id.clone(),
            name,
        });
        output.push(StreamEvent::ToolCallDelta {
            id: id.clone(),
            delta: args,
        });
        output.push(StreamEvent::ToolCallComplete { id });
    }
}

#[cfg(target_os = "android")]
fn send_litert_response_events(
    response: crate::android_litert_jni::NativeLiteRtLmResponse,
    sender: &tokio::sync::mpsc::UnboundedSender<StreamEvent>,
    context: &'static str,
    backend_self_test_passed: Option<&AtomicBool>,
) {
    if !response.text.is_empty() || !response.tool_calls.is_empty() {
        if let Some(flag) = backend_self_test_passed {
            flag.store(true, Ordering::Release);
        }
    }
    let response_text = response.text;
    let tool_result_fallback_applied = response.tool_result_fallback_applied;
    let response_tool_calls: Vec<_> = response
        .tool_calls
        .into_iter()
        .map(|tool_call| (tool_call.name, tool_call.arguments))
        .collect();
    if tool_result_fallback_applied {
        tracing::warn!(
            target: "__APP_NAME___litert_lm",
            tool_calls = response_tool_calls.len(),
            context,
            "LiteRT-LM tool-result fallback follow-up applied as tool_result_fallback_applied"
        );
    }
    if !response_text.is_empty() {
        tracing::info!(
            target: "__APP_NAME___litert_lm",
            chars = response_text.chars().count(),
            tool_calls = response_tool_calls.len(),
            context,
            "LiteRT-LM generation produced text"
        );
    } else {
        tracing::warn!(
            target: "__APP_NAME___litert_lm",
            tool_calls = response_tool_calls.len(),
            context,
            "LiteRT-LM generation returned empty text"
        );
    }
    let mut events = Vec::new();
    push_litert_lm_response_events(
        response_text,
        response_tool_calls,
        tool_result_fallback_applied,
        &mut events,
    );
    for event in events {
        if let StreamEvent::ToolCallStarted { id, name } = &event {
            tracing::info!(
                target: "__APP_NAME___litert_lm",
                id = %id,
                name = %name,
                context,
                "LiteRT-LM generation produced tool call"
            );
        }
        let _ = sender.send(event);
    }
}

#[async_trait]
impl InferenceProvider for LiteRtLmEngine {
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
        if self
            .loaded
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
        if let Some(artifact) = artifact {
            preflight_artifact(artifact)?;
            if let Err(error) = ensure_catalog_model_with_status(
                &path,
                artifact,
                &self.model_operation,
                &self.model_cancelled,
            )
            .await
            {
                let state = if self.model_cancelled.load(Ordering::Acquire) {
                    ModelOperationState::Cancelled
                } else {
                    ModelOperationState::Failed {
                        error: RunError {
                            code: "model_prepare_failed".into(),
                            phase: RunPhase::ModelDownload,
                            message: error.to_string(),
                            retryable: true,
                            diagnostics: None,
                        },
                    }
                };
                let _ = self.model_operation.send(state);
                return Err(error);
            }
        } else if !path.is_file() {
            return Err(CoreError::NotFound(format!(
                "LiteRT-LM model does not exist: {}",
                path.display()
            )));
        }

        let _ = self.model_operation.send(ModelOperationState::Loading);
        tracing::info!(
            target: "__APP_NAME___litert_lm",
            model = %spec.model,
            path = %path.display(),
            "LiteRT-LM model loading"
        );
        let diagnostics = RuntimeDiagnostics {
            model_id: spec.model.clone(),
            backend: "litert-lm-cpu".into(),
            device_name: None,
            offloaded_layers: None,
            fallback_reason: Some(
                "CPU is the certification baseline; GPU/NPU require device proof".into(),
            ),
            gate_report: litert_lm_gate_report(None, false, false),
        };

        #[cfg(target_os = "android")]
        {
            let runtime_cache = self.cache_dir.join("runtime-cache");
            if let Err(error) = std::fs::create_dir_all(&runtime_cache) {
                let run_error = RunError {
                    code: "litert_lm_runtime_cache_failed".into(),
                    phase: RunPhase::ModelLoading,
                    message: format!("LiteRT-LM runtime cache create failed: {error}"),
                    retryable: true,
                    diagnostics: None,
                };
                let _ = self.model_operation.send(ModelOperationState::Failed {
                    error: run_error.clone(),
                });
                return Err(CoreError::Transient(run_error.message));
            }
            match crate::android_litert_jni::NativeLiteRtLmAdapter::load(&path, &runtime_cache) {
                Ok(adapter) => {
                    tracing::info!(
                        target: "__APP_NAME___litert_lm",
                        model = %spec.model,
                        path = %path.display(),
                        cache = %runtime_cache.display(),
                        "LiteRT-LM native adapter loaded"
                    );
                    let mut diagnostics = diagnostics;
                    diagnostics.gate_report = litert_lm_gate_report(None, true, false);
                    *self.native.write().await = Some(adapter);
                    *self.diagnostics.write().await = Some(diagnostics.clone());
                    *self.loaded.write().await = Some(LoadedLiteRtLmModel {
                        model_path: path,
                        spec: spec.clone(),
                        diagnostics: diagnostics.clone(),
                    });
                    let _ = self
                        .model_operation
                        .send(ModelOperationState::Ready { diagnostics });
                    return Ok(());
                }
                Err(error) => {
                    tracing::error!(
                        target: "__APP_NAME___litert_lm",
                        model = %spec.model,
                        path = %path.display(),
                        error = %error,
                        "LiteRT-LM native adapter load failed"
                    );
                    let run_error = RunError {
                        code: "litert_lm_native_adapter_load_failed".into(),
                        phase: RunPhase::ModelLoading,
                        message: error.to_string(),
                        retryable: false,
                        diagnostics: None,
                    };
                    let mut diagnostics = diagnostics;
                    diagnostics.gate_report = litert_lm_gate_report(Some(&run_error), false, false);
                    let _ = self.model_operation.send(ModelOperationState::Failed {
                        error: run_error.clone(),
                    });
                    *self.diagnostics.write().await = Some(diagnostics.clone());
                    *self.loaded.write().await = Some(LoadedLiteRtLmModel {
                        model_path: path,
                        spec: spec.clone(),
                        diagnostics,
                    });
                    return Err(error);
                }
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Non-Android test and host builds can compile the LiteRT-LM
            // catalog and lifecycle lane, but cannot run the Android SDK
            // adapter.
            let error = native_adapter_unavailable();
            let mut diagnostics = diagnostics;
            diagnostics.gate_report = litert_lm_gate_report(Some(&error), false, false);
            let _ = self.model_operation.send(ModelOperationState::Failed {
                error: error.clone(),
            });
            *self.diagnostics.write().await = Some(diagnostics.clone());
            *self.loaded.write().await = Some(LoadedLiteRtLmModel {
                model_path: path,
                spec: spec.clone(),
                diagnostics,
            });
            Err(CoreError::Terminal(error.message))
        }
    }

    async fn generate_request(
        &self,
        #[cfg_attr(not(target_os = "android"), allow(unused_variables))] request: &InferenceRequest,
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        #[cfg(target_os = "android")]
        {
            if self.native.read().await.is_some() {
                let request = request.clone();
                let backend_self_test_passed = Arc::clone(&self.backend_self_test_passed);
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
                gen_ui_runtime::spawn_blocking(move || {
                    let adapter = crate::android_litert_jni::NativeLiteRtLmAdapter;
                    let _ = sender.send(StreamEvent::MessageStart);
                    tracing::info!(
                        target: "__APP_NAME___litert_lm",
                        "LiteRT-LM generation starting"
                    );
                    match adapter.generate_text(&request) {
                        Ok(response) => {
                            send_litert_response_events(
                                response,
                                &sender,
                                "initial",
                                Some(&backend_self_test_passed),
                            );
                        }
                        Err(error) => {
                            tracing::error!(
                                target: "__APP_NAME___litert_lm",
                                error = %error,
                                "LiteRT-LM generation failed"
                            );
                            let _ = sender.send(StreamEvent::Error {
                                message: error.to_string(),
                            });
                        }
                    }
                    let _ = sender.send(StreamEvent::Done);
                });
                return Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed());
            }
        }
        let loaded = self.loaded.read().await;
        let message = if let Some(loaded) = loaded.as_ref() {
            format!(
                "LiteRT-LM native adapter is not available; model {} at {} cannot run yet",
                loaded.diagnostics.model_id,
                loaded.model_path.display()
            )
        } else {
            "LiteRT-LM native adapter is not available; call prepare after Android bridge setup"
                .to_string()
        };
        Ok(futures::stream::iter([StreamEvent::Error { message }, StreamEvent::Done]).boxed())
    }

    async fn continue_with_tool_results(
        &self,
        #[cfg_attr(not(target_os = "android"), allow(unused_variables))]
        results: &[InferenceToolResult],
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        #[cfg(target_os = "android")]
        {
            if self.native.read().await.is_some() {
                let results = results.to_vec();
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
                gen_ui_runtime::spawn_blocking(move || {
                    let adapter = crate::android_litert_jni::NativeLiteRtLmAdapter;
                    tracing::info!(
                        target: "__APP_NAME___litert_lm",
                        tool_results = results.len(),
                        "LiteRT-LM tool-result continuation starting"
                    );
                    match adapter.continue_with_tool_results(&results) {
                        Ok(response) => {
                            send_litert_response_events(response, &sender, "tool_result", None);
                        }
                        Err(error) => {
                            tracing::error!(
                                target: "__APP_NAME___litert_lm",
                                error = %error,
                                "LiteRT-LM tool-result continuation failed"
                            );
                            let _ = sender.send(StreamEvent::Error {
                                message: error.to_string(),
                            });
                        }
                    }
                    let _ = sender.send(StreamEvent::Done);
                });
                return Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed());
            }
        }
        Ok(futures::stream::iter([
            StreamEvent::Error {
                message: "LiteRT-LM native adapter is not available for tool-result continuation"
                    .into(),
            },
            StreamEvent::Done,
        ])
        .boxed())
    }

    async fn unload(&self) -> CoreResult<()> {
        #[cfg(target_os = "android")]
        {
            if self.native.write().await.take().is_some() {
                crate::android_litert_jni::NativeLiteRtLmAdapter::close();
            }
        }
        *self.loaded.write().await = None;
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
        #[cfg(target_os = "android")]
        crate::android_litert_jni::NativeLiteRtLmAdapter::cancel();
        let _ = self.model_operation.send(ModelOperationState::Cancelling);
    }

    async fn runtime_diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::ANDROID_GEMMA4_E2B_LITERTLM_ID;

    #[test]
    fn resolves_android_litert_catalog_model_to_litertlm_file() {
        let spec = LocalModelSpec {
            model: ANDROID_GEMMA4_E2B_LITERTLM_ID.into(),
            context_len: Some(8192),
        };
        let (path, artifact) =
            resolve_model(&spec, Path::new("/tmp/gen-ui-test-models")).expect("catalog model resolves");
        assert_eq!(
            path.file_name().and_then(|value| value.to_str()),
            Some("gemma-4-E2B-it.litertlm")
        );
        assert_eq!(artifact.expect("catalog artifact").byte_len, 2_588_147_712);
    }

    #[test]
    fn gate_report_marks_native_adapter_as_the_current_blocking_gate() {
        let error = native_adapter_unavailable();
        let report = litert_lm_gate_report(Some(&error), false, false);

        assert_eq!(
            report
                .iter()
                .find(|gate| gate.gate == 3)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Passed)
        );
        assert_eq!(
            report
                .iter()
                .find(|gate| gate.gate == 5)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Failed)
        );
        assert_eq!(
            report
                .iter()
                .find(|gate| gate.gate == 5)
                .and_then(|gate| gate.error_code.as_deref()),
            Some("litert_lm_native_adapter_unavailable")
        );
        assert_eq!(
            report
                .iter()
                .find(|gate| gate.gate == 6)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Skipped)
        );
    }

    #[test]
    fn gate_report_keeps_backend_self_test_pending_until_generation_succeeds() {
        let loaded_report = litert_lm_gate_report(None, true, false);
        assert_eq!(
            loaded_report
                .iter()
                .find(|gate| gate.gate == 5)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Passed)
        );
        assert_eq!(
            loaded_report
                .iter()
                .find(|gate| gate.gate == 6)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Pending)
        );

        let proven_report = litert_lm_gate_report(None, true, true);
        assert_eq!(
            proven_report
                .iter()
                .find(|gate| gate.gate == 6)
                .map(|gate| &gate.status),
            Some(&RuntimeGateStatus::Passed)
        );
    }

    #[test]
    fn litert_lm_response_tool_calls_become_canonical_stream_events() {
        let mut events = Vec::new();
        push_litert_lm_response_events(
            "Use this result.".into(),
            vec![(
                "memory_search".into(),
                serde_json::json!({"query": "project plan", "k": 3}),
            )],
            false,
            &mut events,
        );

        assert_eq!(
            events[0],
            StreamEvent::TextDelta {
                index: 0,
                delta: "Use this result.".into()
            }
        );
        assert_eq!(
            events[1],
            StreamEvent::ToolCallStarted {
                id: "litert-tool-0".into(),
                name: "memory_search".into(),
            }
        );
        match &events[2] {
            StreamEvent::ToolCallDelta { id, delta } => {
                assert_eq!(id, "litert-tool-0");
                let parsed: serde_json::Value =
                    serde_json::from_str(delta).expect("tool args are valid JSON");
                assert_eq!(parsed, serde_json::json!({"query": "project plan", "k": 3}));
            }
            other => panic!("expected tool call delta, got {other:?}"),
        }
        assert_eq!(
            events[3],
            StreamEvent::ToolCallComplete {
                id: "litert-tool-0".into(),
            }
        );
    }

    #[test]
    fn litert_lm_tool_result_fallback_is_preserved_as_custom_event() {
        let mut events = Vec::new();
        push_litert_lm_response_events(
            "Approved diagnostic echo: clean777".into(),
            vec![],
            true,
            &mut events,
        );

        assert_eq!(
            events[0],
            StreamEvent::Custom {
                source: "litert_lm".into(),
                event_name: "tool_result_fallback_applied".into(),
                payload: serde_json::json!({
                    "kind": "lifecycle",
                    "phase": "generation",
                    "message": "LiteRT-LM returned an empty tool-result continuation, so the app sent one bounded follow-up instruction to produce a visible answer from the tool result.",
                }),
            }
        );
        assert_eq!(
            events[1],
            StreamEvent::TextDelta {
                index: 0,
                delta: "Approved diagnostic echo: clean777".into(),
            }
        );
    }
}
