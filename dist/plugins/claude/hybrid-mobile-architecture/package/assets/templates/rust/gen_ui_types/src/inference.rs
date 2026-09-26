// TJ-ARCH-MOB-001 compliant
//! InferenceProvider — the local-inference engine seam. One trait, per-lane
//! implementations selected at build time in gen_ui_inference (see
//! `versions.toml` `[inference]`): LiteRT-LM on Android, llama.cpp on desktop
//! and iOS while that lane is being re-certified, WebLLM on web, and optional mistral.rs. UI layers
//! and gen_ui_agent depend on this trait only — never on an engine crate —
//! so swapping or adding an engine never ripples past gen_ui_inference.
//!
//! The default is deliberately shared across native surfaces: the reference app
//! must have one reproducible, checksummed local model path that works first run.
use crate::error::CoreResult;
use crate::events::StreamEvent;
use crate::lifecycle::{ModelOperationState, RuntimeDiagnostics};
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use serde::{Deserialize, Serialize};

/// Identifies a locally loadable model (catalog entry or user-supplied path).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelSpec {
    /// Catalog id (e.g. "qwen2.5-3b-instruct-q4") or an absolute file path.
    pub model: String,
    /// Engine-specific context length; None = engine default.
    pub context_len: Option<u32>,
}

/// Local runtime lane selected by the platform shell. This is explicit so
/// callers and diagnostics can distinguish model families from runtime
/// backends; Android's production lane is LiteRT-LM even when it runs on CPU.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeKind {
    LlamaCpp,
    LiteRtLm,
    Mistral,
    WebLlm,
    /// MLX-Swift on iOS: Metal unified-memory inference behind the same
    /// `InferenceProvider` seam; the Swift side binds generation only.
    Mlx,
}

/// How a model marks its extended-reasoning ("thinking") tokens in the raw
/// text stream, so the engine can split reasoning from content and emit
/// `StreamEvent::ThinkingDelta` instead of leaking it into the assistant reply.
///
/// This is a per-model property (declared in the model preset), not a runtime
/// backend property: the same MLX/llama lane serves Qwen3 (`<think>`), a future
/// harmony model (`<|channel|>analysis`), or a model with no visible reasoning.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningFormat {
    /// No reasoning markup; every token is assistant content.
    #[default]
    None,
    /// Qwen3-style `<think>…</think>` blocks (also DeepSeek-R1 distills).
    Qwen3Think,
    /// OpenAI harmony channels (`<|channel|>analysis<|message|>…`). Reserved for
    /// a future harmony-format local model; the segmenter recognizes the tag set
    /// but no wired model uses it yet.
    Harmony,
}

/// Sampling parameters shared by every engine lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SampleParams {
    pub temperature: f32,
    pub top_p: f32,
    #[serde(default = "default_top_k")]
    pub top_k: i32,
    #[serde(default)]
    pub min_p: f32,
    #[serde(default)]
    pub presence_penalty: f32,
    pub max_tokens: u32,
}

impl Default for SampleParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.8,
            top_k: 20,
            min_p: 0.0,
            presence_penalty: 0.0,
            max_tokens: 1024,
        }
    }
}

const fn default_top_k() -> i32 {
    20
}

/// Provider-neutral chat roles. Keeping roles structured until the engine
/// applies the model's embedded template prevents the role-flattening bugs that
/// previously caused incoherent and wrong-language completions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferenceRole {
    System,
    User,
    Assistant,
    Tool,
}

impl InferenceRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferenceMessage {
    pub role: InferenceRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InferenceTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<InferenceToolAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferenceToolResult {
    pub tool_name: String,
    pub output_json: String,
    pub is_error: bool,
}

/// Provider-neutral subset of MCP `ToolAnnotations`. These are advisory hints,
/// not proof of safety; execution policy still decides whether a local model may
/// call a tool automatically.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferenceToolAnnotations {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// A bounded image supplied to a local multimodal model. The bytes stay in the
/// Rust inference boundary; Flutter only forwards the selected file path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageInput {
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPlanOrigin {
    pub version: String,
    pub estimated_tokens: u32,
    pub estimator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GrammarConstraint {
    pub kind: String,
    pub data: String,
}

/// Canonical provider request. All roles, tools, attachments, sampling,
/// grammar, and context-plan provenance cross the engine boundary together.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceRequest {
    /// Legacy single-turn text retained for wire compatibility. New callers
    /// populate `messages`; engines use this only when `messages` is empty.
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub messages: Vec<InferenceMessage>,
    #[serde(default)]
    pub tools: Vec<InferenceTool>,
    #[serde(default)]
    pub images: Vec<ImageInput>,
    #[serde(default)]
    pub generation_settings: SampleParams,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar: Option<GrammarConstraint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_plan: Option<PromptPlanOrigin>,
}

impl InferenceRequest {
    pub fn effective_messages(&self) -> Vec<InferenceMessage> {
        if self.messages.is_empty() {
            vec![InferenceMessage {
                role: InferenceRole::User,
                content: self.prompt.clone(),
                name: None,
                tool_call_id: None,
            }]
        } else {
            self.messages.clone()
        }
    }
}

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Load (or mmap/attach) the model. Idempotent per spec; heavy work must
    /// run off the async runtime inside the implementation (spawn_blocking).
    async fn load(&self, spec: &LocalModelSpec) -> CoreResult<()>;

    /// Generate from the canonical typed request. Implementations must preserve
    /// roles and apply the model's own chat template; there is intentionally no
    /// raw-string compatibility method.
    async fn generate_request(
        &self,
        request: &InferenceRequest,
    ) -> CoreResult<BoxStream<'static, StreamEvent>>;

    /// Continue a manual tool-calling conversation after the host has executed
    /// one or more tool calls. Engines without a stateful conversation API can
    /// keep the default error; callers must treat that as a visible protocol
    /// limitation rather than silently dropping tool results.
    async fn continue_with_tool_results(
        &self,
        _results: &[InferenceToolResult],
    ) -> CoreResult<BoxStream<'static, StreamEvent>> {
        Err(crate::error::CoreError::Terminal(
            "local inference provider does not support tool-result continuation".into(),
        ))
    }

    /// Release model memory. Safe to call when nothing is loaded.
    async fn unload(&self) -> CoreResult<()>;

    /// Current operational state for download/verification/load UI. Backends
    /// that do not manage downloadable artifacts remain `not_installed` until
    /// their platform adapter provides a richer implementation.
    fn model_operation_state(&self) -> ModelOperationState {
        ModelOperationState::NotInstalled
    }

    /// Subscribe to model-operation changes. The first item is always the
    /// current state so a late UI subscriber never misses preparation status.
    fn watch_model_operation(&self) -> BoxStream<'static, ModelOperationState> {
        futures::stream::once(futures::future::ready(self.model_operation_state())).boxed()
    }

    fn cancel_model_operation(&self) {}

    async fn runtime_diagnostics(&self) -> Option<RuntimeDiagnostics> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_request_round_trips_without_images() {
        let request = InferenceRequest {
            prompt: "Describe the image".into(),
            messages: Vec::new(),
            tools: Vec::new(),
            images: Vec::new(),
            generation_settings: SampleParams::default(),
            grammar: None,
            originating_plan: None,
        };
        let encoded = serde_json::to_string(&request).expect("request serializes");
        let decoded: InferenceRequest = serde_json::from_str(&encoded).expect("request parses");
        assert_eq!(decoded, request);
    }

    #[test]
    fn image_input_preserves_bytes_and_metadata() {
        let request = InferenceRequest {
            prompt: "What is shown?".into(),
            messages: Vec::new(),
            tools: Vec::new(),
            images: vec![ImageInput {
                file_name: "sample.png".into(),
                mime_type: "image/png".into(),
                bytes: vec![137, 80, 78, 71],
            }],
            generation_settings: SampleParams::default(),
            grammar: None,
            originating_plan: None,
        };
        let decoded: InferenceRequest =
            serde_json::from_value(serde_json::to_value(&request).expect("request serializes"))
                .expect("request parses");
        assert_eq!(decoded.images[0].mime_type, "image/png");
        assert_eq!(decoded.images[0].bytes, vec![137, 80, 78, 71]);
    }

    #[test]
    fn structured_messages_preserve_roles_and_tools() {
        let request = InferenceRequest {
            prompt: String::new(),
            messages: vec![InferenceMessage {
                role: InferenceRole::Tool,
                content: "sunny".into(),
                name: Some("weather".into()),
                tool_call_id: Some("call-1".into()),
            }],
            tools: vec![InferenceTool {
                name: "weather".into(),
                description: "Get weather".into(),
                parameters: serde_json::json!({"type": "object"}),
                annotations: Some(InferenceToolAnnotations {
                    title: Some("Weather".into()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: None,
                    open_world_hint: Some(true),
                }),
            }],
            images: Vec::new(),
            generation_settings: SampleParams::default(),
            grammar: Some(GrammarConstraint {
                kind: "lark".into(),
                data: "root: object".into(),
            }),
            originating_plan: Some(PromptPlanOrigin {
                version: "1".into(),
                estimated_tokens: 12,
                estimator: "chars_heuristic".into(),
            }),
        };
        let decoded: InferenceRequest =
            serde_json::from_str(&serde_json::to_string(&request).expect("request serializes"))
                .expect("request parses");
        assert_eq!(decoded, request);
    }
}
