// TJ-ARCH-MOB-001 compliant
//! macOS MLX inference lane, in-process via the `mlex` crate (Apple's official
//! mlx-c C API, statically vendored — no Python).
//!
//! Rust owns catalog, multi-file download/verify, memory preflight, and event
//! normalization, exactly like the iOS lane; the difference is the binding —
//! here generation runs in-process on the Metal GPU through `mlex::Session`
//! rather than through a Swift bridge. `mlex` applies the model's own chat
//! template and parses Hermes-format (`<tool_call>`) calls natively, so this
//! lane uses those directly instead of the Rust-side tool-prompt injection the
//! GGUF/Swift lanes need.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::events::StreamEvent;
use gen_ui_types::inference::{
    InferenceMessage, InferenceProvider, InferenceRequest, InferenceRole, InferenceToolResult,
    LocalModelSpec, SampleParams,
};
use gen_ui_types::lifecycle::{
    ModelOperationState, RunError, RunPhase, RuntimeDiagnostics, RuntimeGateReport,
    RuntimeGateStatus,
};
use parking_lot::Mutex;
use tokio::sync::{watch, RwLock};

use crate::catalog::{mlx_artifact_set_for_id, MlxArtifactSet};
use crate::memory;

use mlex::generate::{GenerateOptions, Session};
use mlex::sampling::SamplingConfig;
use mlex::streaming::TokenKind;
use mlex::tokenizer::ChatMessage;
use mlex::tools::{parse_tool_calls, Tool, ToolCallFormat};

/// Desktop MLX fallback order (largest-to-smallest) for the memory preflight.
/// Desktop reuses the same pinned Qwen3.5 MLX artifacts as iOS — the files are
/// identical; only the device budget differs, and a Mac clears both easily.
const MLXC_FALLBACKS: &[&str] = &[
    crate::catalog::IOS_QWEN35_9B_MLX_ID,
    crate::catalog::IOS_QWEN35_4B_MLX_ID,
];

pub struct MlxcEngine {
    cache_dir: PathBuf,
    /// The loaded `mlex::Session`. `mlex::Session` is not `Sync` (it holds a
    /// `RefCell`-free but `!Sync` prompt-cache `Mutex` internally that we still
    /// serialize on), so all access is serialized through this `Mutex` and
    /// generation runs on a blocking thread.
    session: Arc<Mutex<Option<LoadedSession>>>,
    diagnostics: RwLock<Option<RuntimeDiagnostics>>,
    model_operation: watch::Sender<ModelOperationState>,
    model_cancelled: AtomicBool,
    /// Set on the first non-empty generation (gate 6).
    self_test_passed: Arc<AtomicBool>,
    /// A cooperative cancel flag the generation callback observes.
    generation_cancelled: Arc<AtomicBool>,
    /// Messages of the last generation, for tool-result continuation.
    last_messages: RwLock<Vec<InferenceMessage>>,
}

struct LoadedSession {
    session: Session,
    spec: LocalModelSpec,
}

impl MlxcEngine {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let (model_operation, _) = watch::channel(ModelOperationState::NotInstalled);
        Self {
            cache_dir: cache_dir.into(),
            session: Arc::new(Mutex::new(None)),
            diagnostics: RwLock::new(None),
            model_operation,
            model_cancelled: AtomicBool::new(false),
            self_test_passed: Arc::new(AtomicBool::new(false)),
            generation_cancelled: Arc::new(AtomicBool::new(false)),
            last_messages: RwLock::new(Vec::new()),
        }
    }

    pub async fn diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics.read().await.clone()
    }

    fn model_dir(&self, set: &MlxArtifactSet) -> PathBuf {
        self.cache_dir.join(set.model_id)
    }
}

fn mlxc_peak_bytes_for_id(model_id: &str) -> Option<u64> {
    mlx_artifact_set_for_id(model_id).map(|set| set.estimated_peak_bytes)
}

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
    load_error: Option<&RunError>,
    session_loaded: bool,
    self_test_passed: bool,
) -> Vec<RuntimeGateReport> {
    let mut gates = vec![
        RuntimeGateReport {
            gate: 1,
            name: "rust_library_loaded".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("gen_ui_inference selected the in-process mlx-c provider".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 2,
            name: "model_files_downloaded_or_cached".into(),
            status: RuntimeGateStatus::Passed,
            evidence: Some("every pinned MLX file resolved in the app data cache".into()),
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
            evidence: Some("mlx-c runs on the Metal GPU with unified memory".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 5,
            name: "mlx_session_loaded".into(),
            status: RuntimeGateStatus::Pending,
            evidence: Some("mlx-c session not loaded yet".into()),
            error_code: None,
        },
        RuntimeGateReport {
            gate: 6,
            name: "backend_self_test".into(),
            status: RuntimeGateStatus::Skipped,
            evidence: Some("requires a first non-empty generation".into()),
            error_code: None,
        },
    ];
    if let Some(error) = load_error {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Failed;
            gate.error_code = Some(error.code.clone());
        }
    } else if session_loaded {
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 5) {
            gate.status = RuntimeGateStatus::Passed;
            gate.evidence = Some("mlx-c loaded the model weights and tokenizer".into());
        }
        if let Some(gate) = gates.iter_mut().find(|gate| gate.gate == 6) {
            if self_test_passed {
                gate.status = RuntimeGateStatus::Passed;
                gate.evidence = Some("mlx-c generation returned a non-empty result".into());
            } else {
                gate.status = RuntimeGateStatus::Pending;
                gate.evidence = Some("session loaded; waiting for first generation".into());
            }
        }
    }
    gates
}

/// Map the app's `SampleParams` onto mlex's `SamplingConfig`. Fields mlex does
/// not model (min_p, presence_penalty) are dropped — documented degradation,
/// not a silent divergence.
fn sampling_from(params: &SampleParams) -> SamplingConfig {
    SamplingConfig {
        temperature: params.temperature,
        top_p: params.top_p,
        top_k: (params.top_k > 0).then_some(params.top_k),
        ..SamplingConfig::default()
    }
}

/// Convert gen_ui inference messages into mlex chat messages, preserving roles.
fn chat_messages_from(messages: &[InferenceMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .map(|message| match message.role {
            InferenceRole::System => ChatMessage::system(message.content.clone()),
            InferenceRole::User => ChatMessage::user(message.content.clone()),
            InferenceRole::Assistant => ChatMessage::assistant(message.content.clone()),
            InferenceRole::Tool => ChatMessage::tool_result(
                message.tool_call_id.clone().unwrap_or_default(),
                message.content.clone(),
            ),
        })
        .collect()
}

/// Convert gen_ui tool schemas into mlex tools.
fn tools_from(request: &InferenceRequest) -> Vec<Tool> {
    request
        .tools
        .iter()
        .map(|tool| Tool::new(&tool.name, &tool.description, tool.parameters.clone()))
        .collect()
}

#[async_trait]
impl InferenceProvider for MlxcEngine {
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()> {
        if self
            .session
            .lock()
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

        let set = match resolve_set(spec)? {
            Some(set) => {
                if let Some(downgraded) =
                    memory::downgrade_for_memory(spec, MLXC_FALLBACKS, mlxc_peak_bytes_for_id)
                {
                    tracing::warn!(
                        target: "__APP_NAME___mlxc",
                        requested = %spec.model,
                        selected = %downgraded.model,
                        "requested MLX model exceeds this device's memory budget; using a smaller model"
                    );
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
                            "MLX model {} needs ~{} bytes resident but this machine's safe budget is {} bytes",
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
        let messages = request.effective_messages();
        if messages
            .iter()
            .all(|message| message.content.trim().is_empty())
        {
            return Err(CoreError::Terminal(
                "mlx-c inference: request contains no message content".into(),
            ));
        }
        *self.last_messages.write().await = messages.clone();
        let chat = chat_messages_from(&messages);
        let tools = tools_from(request);
        let sampling = sampling_from(&request.generation_settings);
        let max_tokens = request.generation_settings.max_tokens as usize;
        Ok(self.stream_generation(chat, tools, sampling, max_tokens))
    }

    async fn continue_with_tool_results(
        &self,
        results: &[InferenceToolResult],
    ) -> CoreResult<futures::stream::BoxStream<'static, StreamEvent>> {
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
        let chat = chat_messages_from(&messages);
        Ok(self.stream_generation(chat, Vec::new(), SamplingConfig::default(), 1024))
    }

    async fn unload(&self) -> CoreResult<()> {
        *self.session.lock() = None;
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
        self.generation_cancelled.store(true, Ordering::Release);
        let _ = self.model_operation.send(ModelOperationState::Cancelling);
    }

    async fn runtime_diagnostics(&self) -> Option<RuntimeDiagnostics> {
        self.diagnostics().await
    }
}

impl MlxcEngine {
    async fn load_from_dir(&self, dir: &Path, spec: &LocalModelSpec) -> CoreResult<()> {
        let _ = self.model_operation.send(ModelOperationState::Loading);
        let dir_owned = dir.to_owned();
        let load = gen_ui_runtime::spawn_blocking(move || Session::load(&dir_owned))
            .await
            .map_err(|error| CoreError::Terminal(format!("mlx-c load task: {error}")))?;

        let diagnostics = RuntimeDiagnostics {
            model_id: spec.model.clone(),
            backend: "mlx-c-metal".into(),
            device_name: None,
            offloaded_layers: None,
            fallback_reason: None,
            gate_report: gate_report(None, false, false),
        };

        match load {
            Ok(session) => {
                let mut diagnostics = diagnostics;
                diagnostics.gate_report =
                    gate_report(None, true, self.self_test_passed.load(Ordering::Acquire));
                *self.diagnostics.write().await = Some(diagnostics.clone());
                *self.session.lock() = Some(LoadedSession {
                    session,
                    spec: spec.clone(),
                });
                let _ = self
                    .model_operation
                    .send(ModelOperationState::Ready { diagnostics });
                Ok(())
            }
            Err(error) => {
                let run_error = RunError {
                    code: "mlx_session_load_failed".into(),
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
                Err(CoreError::Terminal(error.to_string()))
            }
        }
    }

    /// Run one generation on a blocking thread, streaming tokens as StreamEvents.
    /// mlex classifies each token (text / reasoning / tool-call span), so we map
    /// those directly rather than re-parsing tags. Tool-call spans are collected
    /// and, at the end, parsed with mlex's Hermes parser into the canonical
    /// tool-call event trio.
    fn stream_generation(
        &self,
        chat: Vec<ChatMessage>,
        tools: Vec<Tool>,
        sampling: SamplingConfig,
        max_tokens: usize,
    ) -> futures::stream::BoxStream<'static, StreamEvent> {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let session = Arc::clone(&self.session);
        let self_test = Arc::clone(&self.self_test_passed);
        let cancel = Arc::clone(&self.generation_cancelled);
        cancel.store(false, Ordering::Release);

        gen_ui_runtime::spawn_blocking(move || {
            let _ = sender.send(StreamEvent::MessageStart);
            let guard = session.lock();
            let Some(loaded) = guard.as_ref() else {
                let _ = sender.send(StreamEvent::Error {
                    message: "mlx-c session is not loaded".into(),
                });
                let _ = sender.send(StreamEvent::Done);
                return;
            };

            let tools_opt = (!tools.is_empty()).then_some(tools.as_slice());
            let prompt_ids = match loaded
                .session
                .encode_chat_with_media_tools(&chat, tools_opt)
            {
                Ok((ids, _media)) => ids,
                Err(error) => {
                    let _ = sender.send(StreamEvent::Error {
                        message: format!("mlx-c prompt encode: {error}"),
                    });
                    let _ = sender.send(StreamEvent::Done);
                    return;
                }
            };

            let options = GenerateOptions {
                max_tokens,
                sampling,
                ..GenerateOptions::default()
            };

            let mut tool_span = String::new();
            let mut produced = false;
            let generate = loaded.session.generate(&prompt_ids, options, |token| {
                if cancel.load(Ordering::Acquire) {
                    return false; // stop generation
                }
                match token.kind {
                    TokenKind::Text => {
                        if !token.text.is_empty() {
                            produced = true;
                            let _ = sender.send(StreamEvent::TextDelta {
                                index: 0,
                                delta: token.text,
                            });
                        }
                    }
                    TokenKind::Reasoning => {
                        if !token.text.is_empty() {
                            produced = true;
                            let _ = sender.send(StreamEvent::ThinkingDelta {
                                index: 0,
                                delta: token.text,
                            });
                        }
                    }
                    TokenKind::ToolCall => {
                        tool_span.push_str(&token.text);
                    }
                }
                true
            });

            if let Err(error) = generate {
                let _ = sender.send(StreamEvent::Error {
                    message: format!("mlx-c generation: {error}"),
                });
                let _ = sender.send(StreamEvent::Done);
                return;
            }

            // Emit any collected tool-call span as canonical tool events.
            if !tool_span.is_empty() {
                for (index, call) in parse_tool_calls(&tool_span, ToolCallFormat::Hermes)
                    .into_iter()
                    .enumerate()
                {
                    produced = true;
                    let id = format!("mlxc-tool-{index}");
                    let _ = sender.send(StreamEvent::ToolCallStarted {
                        id: id.clone(),
                        name: call.name,
                    });
                    let _ = sender.send(StreamEvent::ToolCallDelta {
                        id: id.clone(),
                        delta: call.arguments.to_string(),
                    });
                    let _ = sender.send(StreamEvent::ToolCallComplete { id });
                }
            }

            if cancel.load(Ordering::Acquire) {
                let _ = sender.send(StreamEvent::Cancelled {
                    message: "mlx-c generation cancelled".into(),
                });
            }
            if produced {
                self_test.store(true, Ordering::Release);
            }
            let _ = sender.send(StreamEvent::Done);
        });
        tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).boxed()
    }
}
