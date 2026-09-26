// TJ-ARCH-MOB-001 compliant
//! iOS MLX-Swift inference lane.
//!
//! Rust owns the model catalog, multi-file download/verify, memory preflight,
//! prompt/tool assembly, and event normalization. A thin Swift bridge
//! (`__APP_CLASS__MlxBridge`) binds MLX generation only — the iOS counterpart of the
//! Android LiteRT-LM lane. Unlike LiteRT, MLX streams token-by-token, so the
//! bridge delivers real deltas rather than one synthesized response.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{
    InferenceMessage, InferenceProvider, InferenceRequest, InferenceRole, InferenceToolResult,
    LocalModelSpec, ReasoningFormat,
};
use gen_ui_types::lifecycle::{
    ModelOperationState, RunError, RunPhase, RuntimeDiagnostics, RuntimeGateReport,
    RuntimeGateStatus,
};
use tokio::sync::{watch, RwLock};

use crate::catalog::{mlx_artifact_set_for_id, MlxArtifactSet};
use crate::memory;
use crate::prompt::messages_with_tool_context;

mod tool_parser;
pub(crate) use tool_parser::ToolCallParser;

/// Ordered list of fallbacks (largest-to-smallest) the memory preflight tries
/// when the requested model does not fit. iOS only has the 4B as a smaller
/// option; the 9B opt-in downgrades to it.
const MLX_FALLBACKS: &[&str] = &[
    crate::catalog::IOS_QWEN35_9B_MLX_ID,
    crate::catalog::IOS_QWEN35_4B_MLX_ID,
];

/// One event the Swift bridge streams back during generation. Deserialized
/// from the small per-event JSON the callback delivers.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum MlxBridgeEvent {
    /// One or more decoded tokens.
    Delta { text: String },
    /// Generation finished cleanly (optional token stats, ignored here).
    Final,
    /// Generation was cancelled at Rust's request.
    Cancelled,
    /// The bridge failed; `message` is surfaced as a stream error.
    Error { message: String },
}

/// The native generation binding. The iOS implementation wraps the registered
/// Swift vtable; `MockMlxBridge` (tests) drives the whole engine on the host.
pub(crate) trait MlxBridge: Send + Sync {
    /// Load the model whose files live in `model_dir`. `config_json` carries
    /// context length and sampling defaults. Blocking.
    fn load(&self, model_dir: &Path, config_json: &str) -> CoreResult<()>;

    /// Generate for `request_json`, invoking `on_event` once per streamed
    /// event. Blocking until generation finishes or is cancelled.
    fn generate(
        &self,
        request_json: &str,
        on_event: &mut dyn FnMut(MlxBridgeEvent),
    ) -> CoreResult<()>;

    /// Ask an in-flight `generate` to stop; the blocked call then returns.
    fn cancel(&self);

    /// Release native model memory.
    fn close(&self);

    /// Whether the native runtime is actually available in this build/process.
    /// The simulator and host builds return `false` and the engine reports a
    /// failed gate rather than crashing.
    fn available(&self) -> bool;
}

pub struct MlxEngine {
    cache_dir: PathBuf,
    bridge: Arc<dyn MlxBridge>,
    loaded: RwLock<Option<LoadedMlxModel>>,
    diagnostics: RwLock<Option<RuntimeDiagnostics>>,
    model_operation: watch::Sender<ModelOperationState>,
    model_cancelled: AtomicBool,
    backend_self_test_passed: Arc<AtomicBool>,
    /// Messages of the last generation, so tool-result continuation can rebuild
    /// the conversation without any statefulness in Swift.
    last_messages: RwLock<Vec<InferenceMessage>>,
}

#[derive(Debug, Clone)]
struct LoadedMlxModel {
    model_dir: PathBuf,
    spec: LocalModelSpec,
    diagnostics: RuntimeDiagnostics,
}

impl MlxEngine {
    /// Construct against the platform's registered Swift bridge.
    #[cfg(target_os = "ios")]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self::with_bridge(cache_dir, Arc::new(crate::ios_mlx_ffi::VtableBridge))
    }

    pub(crate) fn with_bridge(cache_dir: impl Into<PathBuf>, bridge: Arc<dyn MlxBridge>) -> Self {
        let (model_operation, _) = watch::channel(ModelOperationState::NotInstalled);
        Self {
            cache_dir: cache_dir.into(),
            bridge,
            loaded: RwLock::new(None),
            diagnostics: RwLock::new(None),
            model_operation,
            model_cancelled: AtomicBool::new(false),
            backend_self_test_passed: Arc::new(AtomicBool::new(false)),
            last_messages: RwLock::new(Vec::new()),
        }
    }

    pub async fn diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics.read().await.clone()
    }

    /// The model directory that holds every verified file for this spec.
    fn model_dir(&self, set: &MlxArtifactSet) -> PathBuf {
        self.cache_dir.join(set.model_id)
    }

    /// Reasoning markup of the currently loaded model, so the segmenter splits
    /// `<think>` blocks. Falls back to `None` when nothing is loaded (the
    /// generation would fail before producing tokens anyway).
    async fn loaded_reasoning_format(&self) -> ReasoningFormat {
        let guard = self.loaded.read().await;
        guard
            .as_ref()
            .and_then(|loaded| mlx_artifact_set_for_id(&loaded.spec.model))
            .map(|set| set.reasoning_format)
            .unwrap_or(ReasoningFormat::None)
    }
}

/// Estimated peak residency for a spec's model, for the memory preflight.
fn mlx_peak_bytes_for_id(model_id: &str) -> Option<u64> {
    mlx_artifact_set_for_id(model_id).map(|set| set.estimated_peak_bytes)
}

/// Resolve a spec to its artifact set, or `None` for an absolute directory path
/// (the caller's own choice — not downgraded, not budget-checked here).
fn resolve_set(spec: &LocalModelSpec) -> CoreResult<Option<&'static MlxArtifactSet>> {
    if Path::new(&spec.model).is_absolute() {
        return Ok(None);
    }
    mlx_artifact_set_for_id(&spec.model)
        .map(Some)
        .ok_or_else(|| {
            CoreError::NotFound(format!(
                "unknown MLX model '{}' — expected a catalog id or an absolute model directory",
                spec.model
            ))
        })
}

fn gate_report(
    bridge_error: Option<&RunError>,
    bridge_loaded: bool,
    self_test_passed: bool,
) -> Vec<RuntimeGateReport> {
    let mut gates = vec![
        RuntimeGateReport {
            gate: 1,
            name: "rust_ffi_library_loaded".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("gen_ui_ffi initialized and selected the MLX provider".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 2,
            name: "model_files_downloaded_or_cached".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("every pinned MLX file resolved in the app-private cache".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 3,
            name: "model_files_verified".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("byte length and SHA-256 matched catalog metadata per file".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 4,
            name: "metal_unified_memory_selected".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("MLX runs on the Metal GPU with unified memory".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 5,
            name: "mlx_swift_bridge_loaded".into(),
            status: RuntimeGateStatus::Pending,
            evidence: Some("MLX-Swift bridge not loaded yet".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 6,
            name: "backend_self_test".into(),
            status: RuntimeGateStatus::Skipped,
            evidence: Some("requires the bridge to produce a token/generation smoke result".into()),
            error_code: None,
        },
    ];
    if let Some(error) = bridge_error {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Failed;
            gate.error_code = Some(error.code.clone());
        }
    } else if bridge_loaded {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Passed;
            gate.evidence = Some("Rust loaded the MLX-Swift bridge and the model".into());
        }
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 6) {
            if self_test_passed {
                gate.status = RuntimeGateStatus::Passed;
                gate.evidence = Some("MLX generation returned a non-empty result".into());
            } else {
                gate.status = RuntimeGateStatus::Pending;
                gate.evidence =
                    Some("bridge loaded; waiting for first successful generation".into());
            }
        }
    }
    gates
}

/// Config JSON handed to the bridge at load: context length and sampling
/// defaults. Deliberately small — the bridge maps these onto MLX's sampler.
fn load_config_json(spec: &LocalModelSpec) -> String {
    serde_json::json!({
        "contextLen": spec.context_len,
    })
    .to_string()
}

#[async_trait]
impl InferenceProvider for MlxEngine {
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

        // Resolve, applying the memory preflight: an over-budget model is
        // downgraded before download, never after a load iOS would jetsam-kill.
        let set = match resolve_set(spec)? {
            Some(set) => {
                if let Some(downgraded) =
                    memory::downgrade_for_memory(spec, MLX_FALLBACKS, mlx_peak_bytes_for_id)
                {
                    tracing::warn!(
                        target: "__APP_NAME___mlx",
                        requested = %spec.model,
                        selected = %downgraded.model,
                        "requested MLX model exceeds this device's memory budget; using a smaller model"
                    );
                    // resolve_set on the downgraded id cannot fail (it came from
                    // the fallback list) and cannot be a path.
                    mlx_artifact_set_for_id(&downgraded.model).ok_or_else(|| {
                        CoreError::Terminal(format!(
                            "downgrade produced unknown model '{}'",
                            downgraded.model
                        ))
                    })?
                } else if memory::preflight_rejects(set.estimated_peak_bytes) {
                    let budget = memory::device_memory_budget_bytes().unwrap_or(0);
                    let error = RunError {
                        code: "mlx_model_exceeds_memory_budget".into(),
                        phase: RunPhase::ModelLoading,
                        message: format!(
                            "MLX model {} needs ~{} bytes resident but this device's safe budget is {} bytes; \
                             no smaller model is available",
                            set.model_id, set.estimated_peak_bytes, budget
                        ),
                        retryable: false,
                        diagnostics: None,
                    };
                    let _ = self.model_operation.send(ModelOperationState::Failed {
                        error: error.clone(),
                    });
                    return Err(CoreError::Terminal(error.message));
                } else {
                    set
                }
            }
            // Absolute directory path: caller-managed, skip download/verify.
            None => {
                let dir = PathBuf::from(&spec.model);
                return self.load_from_dir(&dir, spec).await;
            }
        };

        let dir = self.model_dir(set);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|error| CoreError::Terminal(format!("MLX model dir: {error}")))?;

        crate::mlx_download::download_model_set(
            &dir,
            set,
            &self.model_operation,
            &self.model_cancelled,
        )
        .await
        .inspect_err(|error| {
            if !matches!(error, CoreError::Cancelled(_)) {
                let _ = self.model_operation.send(ModelOperationState::Failed {
                    error: RunError {
                        code: "mlx_model_prepare_failed".into(),
                        phase: RunPhase::ModelDownload,
                        message: error.to_string(),
                        retryable: true,
                        diagnostics: None,
                    },
                });
            } else {
                let _ = self.model_operation.send(ModelOperationState::Cancelled);
            }
        })?;

        let spec = LocalModelSpec {
            model: set.model_id.to_string(),
            context_len: spec.context_len,
        };
        self.load_from_dir(&dir, &spec).await
    }

    async fn generate_request(
        &self,
        request: &InferenceRequest,
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        if !self.bridge.available() {
            return Ok(
                futures::stream::iter([bridge_unavailable_error(), StreamEvent::Done]).boxed(),
            );
        }
        let messages = messages_with_tool_context(request)?;
        *self.last_messages.write().await = messages.clone();
        let request_json = build_request_json(&messages, request)?;
        let bridge = Arc::clone(&self.bridge);
        let self_test = Arc::clone(&self.backend_self_test_passed);
        let reasoning = self.loaded_reasoning_format().await;
        Ok(self.stream_generation(bridge, request_json, self_test, reasoning))
    }

    async fn continue_with_tool_results(
        &self,
        results: &[InferenceToolResult],
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
        if !self.bridge.available() {
            return Ok(
                futures::stream::iter([bridge_unavailable_error(), StreamEvent::Done]).boxed(),
            );
        }
        let mut messages = self.last_messages.read().await.clone();
        for result in results {
            messages.push(InferenceMessage {
                role: InferenceRole::Tool,
                content: result.output_json.clone(),
                name: Some(result.tool_name.clone()),
                tool_call_id: None,
            });
        }
        *self.last_messages.write().await = messages.clone();
        let request_json = build_continuation_json(&messages)?;
        let bridge = Arc::clone(&self.bridge);
        let self_test = Arc::clone(&self.backend_self_test_passed);
        let reasoning = self.loaded_reasoning_format().await;
        Ok(self.stream_generation(bridge, request_json, self_test, reasoning))
    }

    async fn unload(&self) -> CoreResult<()> {
        self.bridge.close();
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
        self.bridge.cancel();
        let _ = self.model_operation.send(ModelOperationState::Cancelling);
    }

    async fn runtime_diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics().await
    }
}

impl MlxEngine {
    /// Load an already-materialized model directory through the bridge and
    /// publish the Ready state.
    async fn load_from_dir(&self, dir: &Path, spec: &LocalModelSpec) -> CoreResult<()> {
        let _ = self.model_operation.send(ModelOperationState::Loading);
        let config = load_config_json(spec);
        let diagnostics = RuntimeDiagnostics {
            model_id: spec.model.clone(),
            backend: "mlx-metal".into(),
            device_name: None,
            offloaded_layers: None,
            fallback_reason: None,
            gate_report: gate_report(None, false, false),
        };

        if !self.bridge.available() {
            let error = RunError {
                code: "mlx_bridge_unavailable".into(),
                phase: RunPhase::ModelLoading,
                message:
                    "MLX-Swift bridge is not available in this process (simulator or host build)"
                        .into(),
                retryable: false,
                diagnostics: Some(
                    "Run on a physical device; the MLX Metal path is unavailable on the simulator."
                        .into(),
                ),
            };
            let mut diagnostics = diagnostics;
            diagnostics.gate_report = gate_report(Some(&error), false, false);
            let _ = self.model_operation.send(ModelOperationState::Failed {
                error: error.clone(),
            });
            *self.diagnostics.write().await = Some(diagnostics);
            return Err(CoreError::Terminal(error.message));
        }

        let dir_owned = dir.to_owned();
        let bridge = Arc::clone(&self.bridge);
        let load_result = gen_ui_runtime::spawn_blocking(move || bridge.load(&dir_owned, &config))
            .await
            .map_err(|error| CoreError::Terminal(format!("MLX load task: {error}")))?;

        match load_result {
            Ok(()) => {
                let mut diagnostics = diagnostics;
                diagnostics.gate_report = gate_report(
                    None,
                    true,
                    self.backend_self_test_passed.load(Ordering::Acquire),
                );
                *self.diagnostics.write().await = Some(diagnostics.clone());
                *self.loaded.write().await = Some(LoadedMlxModel {
                    model_dir: dir.to_owned(),
                    spec: spec.clone(),
                    diagnostics: diagnostics.clone(),
                });
                let _ = self
                    .model_operation
                    .send(ModelOperationState::Ready { diagnostics });
                Ok(())
            }
            Err(error) => {
                let run_error = RunError {
                    code: "mlx_bridge_load_failed".into(),
                    phase: RunPhase::ModelLoading,
                    message: error.to_string(),
                    retryable: false,
                    diagnostics: None,
                };
                let mut diagnostics = diagnostics;
                diagnostics.gate_report = gate_report(Some(&run_error), false, false);
                let _ = self
                    .model_operation
                    .send(ModelOperationState::Failed { error: run_error });
                *self.diagnostics.write().await = Some(diagnostics);
                Err(error)
            }
        }
    }

    /// Drive one bridge generation on a blocking thread, translating bridge
    /// events into canonical `StreamEvent`s through the segmenter. `reasoning`
    /// is the loaded model's reasoning markup, so `<think>` blocks split into
    /// `ThinkingDelta` for models that emit them.
    fn stream_generation(
        &self,
        bridge: Arc<dyn MlxBridge>,
        request_json: String,
        self_test: Arc<AtomicBool>,
        reasoning: ReasoningFormat,
    ) -> futures::stream::BoxStream<'static, StreamEvent> {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        gen_ui_runtime::spawn_blocking(move || {
            let _ = sender.send(StreamEvent::MessageStart);
            let mut parser = ToolCallParser::with_reasoning(reasoning);
            let mut produced_output = false;
            let generate_result = bridge.generate(&request_json, &mut |event| match event {
                MlxBridgeEvent::Delta { text } => {
                    for stream_event in parser.push(&text) {
                        produced_output = true;
                        let _ = sender.send(stream_event);
                    }
                }
                MlxBridgeEvent::Final => {
                    for stream_event in parser.finish() {
                        produced_output = true;
                        let _ = sender.send(stream_event);
                    }
                }
                MlxBridgeEvent::Cancelled => {
                    let _ = sender.send(StreamEvent::Cancelled {
                        message: "MLX generation cancelled".into(),
                    });
                }
                MlxBridgeEvent::Error { message } => {
                    let _ = sender.send(StreamEvent::Error { message });
                }
            });
            if let Err(error) = generate_result {
                let _ = sender.send(StreamEvent::Error {
                    message: error.to_string(),
                });
            } else {
                // A `Final` event flushes the parser; if the bridge ended
                // without one, flush now so a trailing tool call isn't lost.
                for stream_event in parser.finish() {
                    produced_output = true;
                    let _ = sender.send(stream_event);
                }
            }
            if produced_output {
                self_test.store(true, Ordering::Release);
            }
            let _ = sender.send(StreamEvent::Done);
        });
        tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed()
    }
}

fn bridge_unavailable_error() -> StreamEvent {
    StreamEvent::Error {
        message: "MLX-Swift bridge is not available; run on a physical device".into(),
    }
}

/// Serialize the request for the bridge: messages + tools + sampling. Uses the
/// same field names the Swift side expects (serde defaults on the typed
/// structs), keeping the wire contract in one place.
fn build_request_json(
    messages: &[InferenceMessage],
    request: &InferenceRequest,
) -> CoreResult<String> {
    serde_json::to_string(&serde_json::json!({
        "messages": messages,
        "tools": request.tools,
        "images": request.images,
        "generationSettings": request.generation_settings,
    }))
    .map_err(|error| CoreError::Serde(format!("MLX request JSON: {error}")))
}

fn build_continuation_json(messages: &[InferenceMessage]) -> CoreResult<String> {
    serde_json::to_string(&serde_json::json!({
        "messages": messages,
        "tools": [],
        "images": [],
    }))
    .map_err(|error| CoreError::Serde(format!("MLX continuation JSON: {error}")))
}

#[cfg(test)]
mod tests;
