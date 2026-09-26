// TJ-ARCH-MOB-001 compliant
//! Deterministic, engine-neutral context assembly. Platform crates provide
//! history/retrieval/tool adapters; this crate owns capability resolution,
//! budgets, placement, and history strategy behavior.
#![forbid(unsafe_code)]

use async_trait::async_trait;
use gen_ui_types::inference::{InferenceMessage, InferenceRole, InferenceTool};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CONTEXT_PLAN_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceClass {
    MobileHigh,
    MobileMid,
    Desktop,
    Web,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThermalState {
    Nominal,
    Warm,
    Hot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    pub id: String,
    pub class: DeviceClass,
    pub ram_mb: u32,
    pub battery_aware: bool,
    pub max_consecutive_inferences: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuantTier {
    F16,
    Q8,
    Q5,
    Q4,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttentionKind {
    Dense,
    HybridGdn { full_attention_layers: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingModes {
    None,
    Toggleable,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub id: String,
    pub family: String,
    pub parameters_b: f32,
    pub quant: QuantTier,
    pub nominal_context: u32,
    pub trusted_context: u32,
    pub extended_context: Option<u32>,
    pub attention: AttentionKind,
    pub supports_vision: bool,
    pub thinking_modes: ThinkingModes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    LlamaCpp,
    LiteRtLm,
    MistralRs,
    WebLlm,
    CloudApi,
    Mlx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EstimatorKind {
    ExactTokenizer,
    CharsHeuristic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProfile {
    pub engine: EngineKind,
    pub grammar_constrained: bool,
    pub streaming: bool,
    pub multimodal: bool,
    pub estimator: EstimatorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionEnvelope {
    pub consecutive_inferences: u32,
    pub session_infer_ms: u64,
    pub thermal_state: ThermalState,
    pub last_ttft_ms: Option<u64>,
    pub last_total_tokens: Option<u32>,
}

impl Default for SessionEnvelope {
    fn default() -> Self {
        Self {
            consecutive_inferences: 0,
            session_infer_ms: 0,
            thermal_state: ThermalState::Nominal,
            last_ttft_ms: None,
            last_total_tokens: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SliceKind {
    System,
    Tools,
    History,
    Memory,
    Knowledge,
    Query,
    OutputReserve,
    ThinkingReserve,
    SafetyMargin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BudgetPlan {
    pub id: String,
    pub context_ceiling: u32,
    pub system_prompt_max: u32,
    pub tool_schema_max: u32,
    pub history_max: u32,
    pub memory_max: u32,
    pub knowledge_max: u32,
    pub query_max: u32,
    pub output_reserve: u32,
    pub thinking_reserve: u32,
    pub safety_margin_pct: u8,
    pub max_tool_shortlist: u8,
    pub max_memory_items: u8,
    pub max_kb_retrievals: u8,
    pub chunk_target_tokens: u16,
    pub max_autonomous_steps: u8,
}

impl BudgetPlan {
    pub fn validate(&self) -> Result<(), ContextError> {
        let margin = self.context_ceiling * u32::from(self.safety_margin_pct) / 100;
        let committed = self.system_prompt_max
            + self.tool_schema_max
            + self.history_max
            + self.memory_max
            + self.knowledge_max
            + self.query_max
            + self.output_reserve
            + self.thinking_reserve
            + margin;
        if committed > self.context_ceiling {
            return Err(ContextError::InvalidBudget {
                committed,
                ceiling: self.context_ceiling,
            });
        }
        let retrieval = self.memory_max + self.knowledge_max;
        if retrieval > 0 && self.memory_max * 10 != retrieval * 3 {
            return Err(ContextError::InvalidRetrievalSplit);
        }
        Ok(())
    }

    pub fn cap(&self, kind: SliceKind) -> u32 {
        match kind {
            SliceKind::System => self.system_prompt_max,
            SliceKind::Tools => self.tool_schema_max,
            SliceKind::History => self.history_max,
            SliceKind::Memory => self.memory_max,
            SliceKind::Knowledge => self.knowledge_max,
            SliceKind::Query => self.query_max,
            SliceKind::OutputReserve => self.output_reserve,
            SliceKind::ThinkingReserve => self.thinking_reserve,
            SliceKind::SafetyMargin => {
                self.context_ceiling * u32::from(self.safety_margin_pct) / 100
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveCapabilities {
    pub device: DeviceProfile,
    pub model: ModelProfile,
    pub runtime: RuntimeProfile,
    pub plan: BudgetPlan,
    pub degraded: bool,
    pub escalation_advice: Option<String>,
}

pub fn resolve_capabilities(
    device: DeviceProfile,
    model: ModelProfile,
    runtime: RuntimeProfile,
    envelope: &SessionEnvelope,
    extended_context_enabled: bool,
) -> Result<EffectiveCapabilities, ContextError> {
    let mut plan = presets::for_model(&model, &runtime, extended_context_enabled);
    let degraded = envelope.thermal_state == ThermalState::Hot
        || envelope.consecutive_inferences >= device.max_consecutive_inferences;
    if degraded {
        plan.output_reserve = plan.output_reserve.saturating_mul(3) / 4;
        let retrieval_total = plan.memory_max + plan.knowledge_max;
        let reduced = (retrieval_total.saturating_mul(2) / 3) / 10 * 10;
        plan.memory_max = reduced * 3 / 10;
        plan.knowledge_max = reduced * 7 / 10;
        plan.max_kb_retrievals = plan.max_kb_retrievals.saturating_sub(1).max(1);
    }
    plan.validate()?;
    Ok(EffectiveCapabilities {
        device,
        model,
        runtime,
        plan,
        degraded,
        escalation_advice: degraded.then(|| {
            "The local session is thermally constrained; shorten the run or use cloud inference."
                .into()
        }),
    })
}

pub mod presets {
    use super::*;

    pub fn for_model(model: &ModelProfile, runtime: &RuntimeProfile, extended: bool) -> BudgetPlan {
        let ceiling = if extended {
            model.extended_context.unwrap_or(model.trusted_context)
        } else {
            model.trusted_context
        };
        let heuristic_margin = matches!(runtime.estimator, EstimatorKind::CharsHeuristic);
        if model.id == "qwen3-4b-instruct-2507-q5-k-m" {
            BudgetPlan {
                id: "mobile-4b-android-v1".into(),
                context_ceiling: ceiling,
                system_prompt_max: 500,
                tool_schema_max: 200,
                history_max: 1_200,
                memory_max: 600,
                knowledge_max: 1_400,
                query_max: 500,
                output_reserve: 768,
                thinking_reserve: 0,
                safety_margin_pct: if heuristic_margin { 15 } else { 10 },
                max_tool_shortlist: 4,
                max_memory_items: 3,
                max_kb_retrievals: 3,
                chunk_target_tokens: 250,
                max_autonomous_steps: 3,
            }
        } else if model.id == "qwen3.5-9b-q4-k-m" {
            BudgetPlan {
                id: "mobile-9b-ios-v1".into(),
                context_ceiling: ceiling,
                system_prompt_max: 600,
                tool_schema_max: 300,
                history_max: 1_600,
                memory_max: 750,
                knowledge_max: 1_750,
                query_max: 300,
                output_reserve: 1_024,
                thinking_reserve: 768,
                safety_margin_pct: if heuristic_margin { 15 } else { 10 },
                max_tool_shortlist: 6,
                max_memory_items: 4,
                max_kb_retrievals: 5,
                chunk_target_tokens: 320,
                max_autonomous_steps: 4,
            }
        } else {
            BudgetPlan {
                id: "general-v1".into(),
                context_ceiling: ceiling,
                system_prompt_max: 600,
                tool_schema_max: 300,
                history_max: 1_800,
                memory_max: 600,
                knowledge_max: 1_400,
                query_max: 500,
                output_reserve: 1_024,
                thinking_reserve: 0,
                safety_margin_pct: if heuristic_margin { 15 } else { 10 },
                max_tool_shortlist: 8,
                max_memory_items: 6,
                max_kb_retrievals: 8,
                chunk_target_tokens: 400,
                max_autonomous_steps: 6,
            }
        }
    }

    pub fn android_qwen3() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        (
            DeviceProfile {
                id: "galaxy-s25".into(),
                class: DeviceClass::MobileHigh,
                ram_mb: 12_288,
                battery_aware: true,
                max_consecutive_inferences: 12,
            },
            ModelProfile {
                id: "qwen3-4b-instruct-2507-q5-k-m".into(),
                family: "qwen3".into(),
                parameters_b: 4.0,
                quant: QuantTier::Q5,
                nominal_context: 262_144,
                trusted_context: 8_192,
                extended_context: Some(16_384),
                attention: AttentionKind::Dense,
                supports_vision: false,
                thinking_modes: ThinkingModes::None,
            },
            RuntimeProfile {
                engine: EngineKind::LlamaCpp,
                grammar_constrained: true,
                streaming: true,
                multimodal: false,
                estimator: EstimatorKind::ExactTokenizer,
            },
        )
    }

    pub fn android_litert_gemma4_e2b() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        (
            DeviceProfile {
                id: "modern-android-arm64".into(),
                class: DeviceClass::MobileHigh,
                ram_mb: 12_288,
                battery_aware: true,
                max_consecutive_inferences: 12,
            },
            ModelProfile {
                id: "gemma-4-e2b-it-litertlm".into(),
                family: "gemma4".into(),
                parameters_b: 2.0,
                quant: QuantTier::Q4,
                nominal_context: 32_768,
                trusted_context: 8_192,
                extended_context: Some(16_384),
                attention: AttentionKind::Dense,
                supports_vision: false,
                thinking_modes: ThinkingModes::None,
            },
            RuntimeProfile {
                engine: EngineKind::LiteRtLm,
                grammar_constrained: true,
                streaming: true,
                multimodal: false,
                estimator: EstimatorKind::CharsHeuristic,
            },
        )
    }

    pub fn android_litert_gemma4_e2b_sm8750() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        let (mut device, mut model, runtime) = android_litert_gemma4_e2b();
        device.id = "qualcomm-sm8750".into();
        model.id = "gemma-4-e2b-it-litertlm-sm8750".into();
        (device, model, runtime)
    }

    pub fn ios_qwen35() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        (
            DeviceProfile {
                id: "iphone-17-pro-max".into(),
                class: DeviceClass::MobileHigh,
                ram_mb: 12_288,
                battery_aware: true,
                max_consecutive_inferences: 8,
            },
            ModelProfile {
                id: "qwen3.5-9b-q4-k-m".into(),
                family: "qwen3.5".into(),
                parameters_b: 9.0,
                quant: QuantTier::Q4,
                nominal_context: 262_144,
                trusted_context: 8_192,
                extended_context: Some(16_384),
                attention: AttentionKind::HybridGdn {
                    full_attention_layers: 8,
                },
                supports_vision: true,
                thinking_modes: ThinkingModes::Toggleable,
            },
            RuntimeProfile {
                engine: EngineKind::LlamaCpp,
                grammar_constrained: true,
                streaming: true,
                multimodal: true,
                estimator: EstimatorKind::ExactTokenizer,
            },
        )
    }

    /// iOS default: Qwen3.5-4B 4-bit under MLX-Swift. Profiled for the 8 GB
    /// baseline device so the default works on every supported iPhone; the
    /// 12 GB Pro tier opts into the 9B instead. Text lane today — MLXVLM's
    /// Swift registry has no Qwen3.5 vision processor yet, so vision is off
    /// and images route through the Xberg extraction path, mirroring the
    /// Android text-only lane.
    pub fn ios_qwen35_4b_mlx() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        (
            DeviceProfile {
                id: "iphone-17".into(),
                class: DeviceClass::MobileHigh,
                ram_mb: 8_192,
                battery_aware: true,
                max_consecutive_inferences: 8,
            },
            ModelProfile {
                id: "qwen3.5-4b-mlx-4bit".into(),
                family: "qwen3.5".into(),
                parameters_b: 4.0,
                quant: QuantTier::Q4,
                nominal_context: 262_144,
                trusted_context: 8_192,
                extended_context: Some(16_384),
                attention: AttentionKind::HybridGdn {
                    full_attention_layers: 8,
                },
                supports_vision: false,
                thinking_modes: ThinkingModes::Toggleable,
            },
            RuntimeProfile {
                engine: EngineKind::Mlx,
                // No llguidance hookup in the MLX bridge; constrained output
                // relies on prompt-level schema guidance until one exists.
                grammar_constrained: false,
                streaming: true,
                multimodal: false,
                estimator: EstimatorKind::ExactTokenizer,
            },
        )
    }

    /// 12 GB-device opt-in tier: Qwen3.5-9B 4-bit under MLX-Swift (the Qwen3.5
    /// line has no 8B). Admission is decided by the memory preflight against
    /// the catalog's estimated peak residency, never assumed.
    pub fn ios_qwen35_9b_mlx() -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        let (mut device, mut model, runtime) = ios_qwen35_4b_mlx();
        device.id = "iphone-17-pro-max".into();
        device.ram_mb = 12_288;
        model.id = "qwen3.5-9b-mlx-4bit".into();
        model.parameters_b = 9.0;
        (device, model, runtime)
    }

    pub fn cloud(model_id: &str) -> (DeviceProfile, ModelProfile, RuntimeProfile) {
        (
            DeviceProfile {
                id: "cloud-managed".into(),
                class: DeviceClass::Server,
                ram_mb: 0,
                battery_aware: false,
                max_consecutive_inferences: u32::MAX,
            },
            ModelProfile {
                id: model_id.into(),
                family: "cloud".into(),
                parameters_b: 0.0,
                quant: QuantTier::Cloud,
                nominal_context: 32_768,
                trusted_context: 32_768,
                extended_context: None,
                attention: AttentionKind::Dense,
                supports_vision: true,
                thinking_modes: ThinkingModes::Toggleable,
            },
            RuntimeProfile {
                engine: EngineKind::CloudApi,
                grammar_constrained: true,
                streaming: true,
                multimodal: true,
                estimator: EstimatorKind::CharsHeuristic,
            },
        )
    }
}

pub trait TokenEstimator: Send + Sync {
    fn estimate(&self, text: &str) -> u32;
    fn kind(&self) -> EstimatorKind;
}

pub struct CharsHeuristic;

impl TokenEstimator for CharsHeuristic {
    fn estimate(&self, text: &str) -> u32 {
        text.chars().count().div_ceil(4) as u32
    }

    fn kind(&self) -> EstimatorKind {
        EstimatorKind::CharsHeuristic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticKind {
    User,
    AssistantAnswer,
    ToolResult,
    Thinking,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub id: String,
    pub role: InferenceRole,
    pub text: String,
    pub semantic_kind: SemanticKind,
    pub tool_group_id: Option<String>,
}

impl Turn {
    pub fn eligible_for_context(&self) -> bool {
        matches!(
            self.semantic_kind,
            SemanticKind::User | SemanticKind::AssistantAnswer | SemanticKind::ToolResult
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    SlidingWindow,
    TruncateMiddle,
    Summarize,
    Hierarchical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StrategyConfig {
    pub kind: StrategyKind,
    pub keep_first: usize,
    pub keep_last: usize,
    pub summary_max_tokens: u32,
    pub summarize_threshold: usize,
    pub short_term_turns: usize,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            kind: StrategyKind::SlidingWindow,
            keep_first: 2,
            keep_last: 4,
            summary_max_tokens: 500,
            summarize_threshold: 6,
            short_term_turns: 5,
        }
    }
}

#[async_trait]
pub trait Summarizer: Send + Sync {
    async fn summarize(&self, turns: &[Turn], max_tokens: u32) -> Result<String, ContextError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RankedItem {
    pub id: String,
    pub text: String,
    pub score_micros: i64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolCandidate {
    pub tool: InferenceTool,
    pub score: f32,
}

#[async_trait]
pub trait ContextSources: Send + Sync {
    async fn history(
        &self,
        conversation_id: &str,
        branch_id: &str,
    ) -> Result<Vec<Turn>, ContextError>;
    async fn memories(&self, query: &str, limit: usize) -> Result<Vec<RankedItem>, ContextError>;
    async fn knowledge(
        &self,
        conversation_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RankedItem>, ContextError>;
    async fn tools(&self, query: &str) -> Result<Vec<ToolCandidate>, ContextError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssembleRequest {
    pub conversation_id: String,
    pub branch_id: String,
    pub assistant_id: String,
    pub system_prompt: String,
    pub user_text: String,
    pub thinking_requested: bool,
    pub strategy: StrategyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SliceTelemetry {
    pub kind: SliceKind,
    pub budget: u32,
    pub spent: u32,
    pub items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextTelemetryEvent {
    pub plan_version: String,
    pub strategy: StrategyKind,
    pub tokens_before: u32,
    pub tokens_after: u32,
    pub tokens_saved: u32,
    pub messages_removed: u32,
    pub slices: Vec<SliceTelemetry>,
    pub estimator: EstimatorKind,
    pub degraded: bool,
    pub alteration_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPlan {
    pub version: String,
    pub messages: Vec<InferenceMessage>,
    pub estimated_tokens: u32,
    pub estimator: EstimatorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssembledContext {
    pub plan: BudgetPlan,
    pub prompt: PromptPlan,
    pub tools: Vec<InferenceTool>,
    pub telemetry: ContextTelemetryEvent,
    pub escalation: Option<String>,
}

#[derive(Debug)]
struct BudgetLedger<'a> {
    plan: &'a BudgetPlan,
    spent: BTreeMap<SliceKind, u32>,
    items: BTreeMap<SliceKind, u32>,
}

impl<'a> BudgetLedger<'a> {
    fn new(plan: &'a BudgetPlan) -> Self {
        Self {
            plan,
            spent: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }

    fn admit(&mut self, kind: SliceKind, tokens: u32) -> bool {
        let spent = self.spent.entry(kind).or_default();
        if spent.saturating_add(tokens) > self.plan.cap(kind) {
            return false;
        }
        *spent += tokens;
        *self.items.entry(kind).or_default() += 1;
        true
    }

    fn telemetry(&self) -> Vec<SliceTelemetry> {
        [
            SliceKind::System,
            SliceKind::Tools,
            SliceKind::History,
            SliceKind::Memory,
            SliceKind::Knowledge,
            SliceKind::Query,
            SliceKind::OutputReserve,
            SliceKind::ThinkingReserve,
            SliceKind::SafetyMargin,
        ]
        .into_iter()
        .map(|kind| SliceTelemetry {
            kind,
            budget: self.plan.cap(kind),
            spent: self.spent.get(&kind).copied().unwrap_or_default(),
            items: self.items.get(&kind).copied().unwrap_or_default(),
        })
        .collect()
    }
}

pub struct ContextBuilder<'a> {
    pub capabilities: &'a EffectiveCapabilities,
    pub estimator: &'a dyn TokenEstimator,
    pub sources: &'a dyn ContextSources,
    pub summarizer: Option<&'a dyn Summarizer>,
}

impl ContextBuilder<'_> {
    pub async fn assemble(
        &self,
        request: &AssembleRequest,
    ) -> Result<AssembledContext, ContextError> {
        let mut plan = self.capabilities.plan.clone();
        if !request.thinking_requested
            || matches!(self.capabilities.model.thinking_modes, ThinkingModes::None)
        {
            plan.thinking_reserve = 0;
        }
        plan.validate()?;
        let mut ledger = BudgetLedger::new(&plan);
        let mut messages = Vec::new();

        admit_message(
            &mut ledger,
            self.estimator,
            SliceKind::System,
            InferenceMessage {
                role: InferenceRole::System,
                content: request.system_prompt.clone(),
                name: None,
                tool_call_id: None,
            },
            &mut messages,
        )?;

        let mut tool_candidates = self.sources.tools(&request.user_text).await?;
        tool_candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.tool.name.cmp(&b.tool.name))
        });
        let mut tools = Vec::new();
        for candidate in tool_candidates
            .into_iter()
            .take(usize::from(plan.max_tool_shortlist))
        {
            let rendered = serde_json::to_string(&candidate.tool)
                .map_err(|error| ContextError::Serialization(error.to_string()))?;
            if ledger.admit(SliceKind::Tools, self.estimator.estimate(&rendered)) {
                tools.push(candidate.tool);
            }
        }

        let raw_history = self
            .sources
            .history(&request.conversation_id, &request.branch_id)
            .await?;
        let eligible = raw_history
            .iter()
            .filter(|turn| turn.eligible_for_context())
            .cloned()
            .collect::<Vec<_>>();
        let tokens_before = eligible
            .iter()
            .map(|turn| self.estimator.estimate(&turn.text))
            .sum();
        let (history, removed, degraded_strategy) = self
            .apply_strategy(eligible, &request.strategy, plan.history_max)
            .await?;
        for turn in history {
            let tokens = self.estimator.estimate(&turn.text);
            if ledger.admit(SliceKind::History, tokens) {
                messages.push(InferenceMessage {
                    role: turn.role,
                    content: turn.text,
                    name: None,
                    tool_call_id: None,
                });
            }
        }

        let memories = self
            .sources
            .memories(&request.user_text, usize::from(plan.max_memory_items) * 3)
            .await?;
        let knowledge = self
            .sources
            .knowledge(
                &request.conversation_id,
                &request.user_text,
                usize::from(plan.max_kb_retrievals) * 3,
            )
            .await?;
        admit_ranked_section(
            &mut ledger,
            self.estimator,
            SliceKind::Memory,
            "Relevant personal memories",
            memories,
            usize::from(plan.max_memory_items),
            &mut messages,
        );
        admit_ranked_section(
            &mut ledger,
            self.estimator,
            SliceKind::Knowledge,
            "Relevant conversation documents",
            knowledge,
            usize::from(plan.max_kb_retrievals),
            &mut messages,
        );

        admit_message(
            &mut ledger,
            self.estimator,
            SliceKind::Query,
            InferenceMessage {
                role: InferenceRole::User,
                content: request.user_text.clone(),
                name: None,
                tool_call_id: None,
            },
            &mut messages,
        )?;
        let estimated_tokens = messages
            .iter()
            .map(|message| self.estimator.estimate(&message.content))
            .sum::<u32>()
            + tools
                .iter()
                .map(|tool| {
                    serde_json::to_string(tool)
                        .map(|text| self.estimator.estimate(&text))
                        .unwrap_or_default()
                })
                .sum::<u32>();
        let tokens_after = ledger
            .spent
            .get(&SliceKind::History)
            .copied()
            .unwrap_or_default();
        Ok(AssembledContext {
            plan: plan.clone(),
            prompt: PromptPlan {
                version: CONTEXT_PLAN_VERSION.into(),
                messages,
                estimated_tokens,
                estimator: self.estimator.kind(),
            },
            tools,
            telemetry: ContextTelemetryEvent {
                plan_version: CONTEXT_PLAN_VERSION.into(),
                strategy: request.strategy.kind.clone(),
                tokens_before,
                tokens_after,
                tokens_saved: tokens_before.saturating_sub(tokens_after),
                messages_removed: removed,
                slices: ledger.telemetry(),
                estimator: self.estimator.kind(),
                degraded: self.capabilities.degraded || degraded_strategy,
                alteration_summary: if removed == 0 {
                    "History fit without alteration".into()
                } else {
                    format!("Removed or summarized {removed} history messages")
                },
            },
            escalation: self.capabilities.escalation_advice.clone(),
        })
    }

    async fn apply_strategy(
        &self,
        turns: Vec<Turn>,
        config: &StrategyConfig,
        budget: u32,
    ) -> Result<(Vec<Turn>, u32, bool), ContextError> {
        match config.kind {
            StrategyKind::SlidingWindow => {
                let selected = sliding_window(turns.clone(), budget, self.estimator);
                let removed = turns.len().saturating_sub(selected.len()) as u32;
                Ok((selected, removed, false))
            }
            StrategyKind::TruncateMiddle => {
                let selected = truncate_middle(turns.clone(), config);
                let selected = sliding_window(selected, budget, self.estimator);
                let removed = turns.len().saturating_sub(selected.len()) as u32;
                Ok((selected, removed, false))
            }
            StrategyKind::Summarize | StrategyKind::Hierarchical => {
                if turns.len() < config.summarize_threshold {
                    let selected = sliding_window(turns.clone(), budget, self.estimator);
                    let removed = turns.len().saturating_sub(selected.len()) as u32;
                    return Ok((selected, removed, false));
                }
                let keep = if config.kind == StrategyKind::Hierarchical {
                    config.short_term_turns
                } else {
                    config.keep_last
                }
                .min(turns.len());
                let split = turns.len() - keep;
                let Some(summarizer) = self.summarizer else {
                    let selected = sliding_window(turns.clone(), budget, self.estimator);
                    let removed = turns.len().saturating_sub(selected.len()) as u32;
                    return Ok((selected, removed, true));
                };
                match summarizer
                    .summarize(&turns[..split], config.summary_max_tokens)
                    .await
                {
                    Ok(summary) => {
                        let mut selected = vec![Turn {
                            id: "context-summary".into(),
                            role: InferenceRole::System,
                            text: format!("Conversation summary:\n{summary}"),
                            semantic_kind: SemanticKind::AssistantAnswer,
                            tool_group_id: None,
                        }];
                        selected.extend_from_slice(&turns[split..]);
                        let selected = sliding_window(selected, budget, self.estimator);
                        Ok((selected, split as u32, false))
                    }
                    Err(_) => {
                        let selected = sliding_window(turns.clone(), budget, self.estimator);
                        let removed = turns.len().saturating_sub(selected.len()) as u32;
                        Ok((selected, removed, true))
                    }
                }
            }
        }
    }
}

fn admit_message(
    ledger: &mut BudgetLedger<'_>,
    estimator: &dyn TokenEstimator,
    kind: SliceKind,
    message: InferenceMessage,
    messages: &mut Vec<InferenceMessage>,
) -> Result<(), ContextError> {
    let tokens = estimator.estimate(&message.content);
    if !ledger.admit(kind, tokens) {
        return Err(ContextError::RequiredSliceOverflow { kind, tokens });
    }
    messages.push(message);
    Ok(())
}

fn sliding_window(turns: Vec<Turn>, budget: u32, estimator: &dyn TokenEstimator) -> Vec<Turn> {
    let mut spent = 0;
    let mut selected = Vec::new();
    for turn in turns.into_iter().rev() {
        let tokens = estimator.estimate(&turn.text);
        if spent + tokens > budget {
            continue;
        }
        spent += tokens;
        selected.push(turn);
    }
    selected.reverse();
    selected
}

fn truncate_middle(turns: Vec<Turn>, config: &StrategyConfig) -> Vec<Turn> {
    if turns.len() <= config.keep_first + config.keep_last {
        return turns;
    }
    let omitted = turns.len() - config.keep_first - config.keep_last;
    let mut selected = turns[..config.keep_first].to_vec();
    selected.push(Turn {
        id: "context-omission".into(),
        role: InferenceRole::System,
        text: format!("[… {omitted} earlier messages omitted …]"),
        semantic_kind: SemanticKind::AssistantAnswer,
        tool_group_id: None,
    });
    selected.extend_from_slice(&turns[turns.len() - config.keep_last..]);
    selected
}

fn placement_order(mut items: Vec<RankedItem>) -> Vec<RankedItem> {
    items.sort_by(|a, b| {
        b.score_micros
            .cmp(&a.score_micros)
            .then_with(|| a.id.cmp(&b.id))
    });
    if items.len() < 2 {
        return items;
    }
    let best = items.remove(0);
    let second = items.remove(0);
    let mut placed = vec![second];
    placed.extend(items);
    placed.push(best);
    placed
}

fn admit_ranked_section(
    ledger: &mut BudgetLedger<'_>,
    estimator: &dyn TokenEstimator,
    kind: SliceKind,
    heading: &str,
    items: Vec<RankedItem>,
    limit: usize,
    messages: &mut Vec<InferenceMessage>,
) {
    let mut admitted = Vec::new();
    for item in placement_order(items).into_iter().take(limit) {
        let rendered = format!("[{}] {}", item.source, item.text);
        if ledger.admit(kind, estimator.estimate(&rendered)) {
            admitted.push(rendered);
        }
    }
    if !admitted.is_empty() {
        messages.push(InferenceMessage {
            role: InferenceRole::System,
            content: format!("{heading}:\n{}", admitted.join("\n\n")),
            name: None,
            tool_call_id: None,
        });
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContextError {
    #[error("context slices commit {committed} tokens above ceiling {ceiling}")]
    InvalidBudget { committed: u32, ceiling: u32 },
    #[error("memory and knowledge budgets must retain the fixed 30/70 split")]
    InvalidRetrievalSplit,
    #[error("required {kind:?} content needs {tokens} tokens and does not fit")]
    RequiredSliceOverflow { kind: SliceKind, tokens: u32 },
    #[error("context serialization failed: {0}")]
    Serialization(String),
    #[error("context source failed: {0}")]
    Source(String),
    #[error("summarization failed: {0}")]
    Summarization(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sources {
        history: Vec<Turn>,
    }

    #[async_trait]
    impl ContextSources for Sources {
        async fn history(&self, _: &str, _: &str) -> Result<Vec<Turn>, ContextError> {
            Ok(self.history.clone())
        }

        async fn memories(&self, _: &str, _: usize) -> Result<Vec<RankedItem>, ContextError> {
            Ok(vec![
                RankedItem {
                    id: "m1".into(),
                    text: "best".into(),
                    score_micros: 100,
                    source: "memory-1".into(),
                },
                RankedItem {
                    id: "m2".into(),
                    text: "second".into(),
                    score_micros: 90,
                    source: "memory-2".into(),
                },
            ])
        }

        async fn knowledge(
            &self,
            _: &str,
            _: &str,
            _: usize,
        ) -> Result<Vec<RankedItem>, ContextError> {
            Ok(Vec::new())
        }

        async fn tools(&self, _: &str) -> Result<Vec<ToolCandidate>, ContextError> {
            Ok(Vec::new())
        }
    }

    fn capabilities() -> EffectiveCapabilities {
        let (device, model, runtime) = presets::android_litert_gemma4_e2b();
        resolve_capabilities(device, model, runtime, &SessionEnvelope::default(), false)
            .expect("preset resolves")
    }

    #[test]
    fn android_litert_profile_is_dense_text_only_and_non_thinking() {
        let value = capabilities();
        assert_eq!(value.runtime.engine, EngineKind::LiteRtLm);
        assert_eq!(value.model.id, "gemma-4-e2b-it-litertlm");
        assert_eq!(value.model.attention, AttentionKind::Dense);
        assert!(!value.model.supports_vision);
        assert_eq!(value.model.thinking_modes, ThinkingModes::None);
        assert_eq!(value.plan.thinking_reserve, 0);
    }

    #[test]
    fn plan_rejects_retrieval_split_drift() {
        let mut plan = capabilities().plan;
        plan.memory_max += 1;
        assert_eq!(plan.validate(), Err(ContextError::InvalidRetrievalSplit));
    }

    #[test]
    fn reasoning_and_errors_are_never_eligible_for_future_context() {
        for kind in [
            SemanticKind::Thinking,
            SemanticKind::Error,
            SemanticKind::Cancelled,
        ] {
            let turn = Turn {
                id: "x".into(),
                role: InferenceRole::Assistant,
                text: "private".into(),
                semantic_kind: kind,
                tool_group_id: None,
            };
            assert!(!turn.eligible_for_context());
        }
    }

    #[test]
    fn placement_puts_second_first_and_best_last() {
        let placed = placement_order(vec![
            RankedItem {
                id: "1".into(),
                text: "best".into(),
                score_micros: 100,
                source: "a".into(),
            },
            RankedItem {
                id: "2".into(),
                text: "second".into(),
                score_micros: 90,
                source: "b".into(),
            },
            RankedItem {
                id: "3".into(),
                text: "third".into(),
                score_micros: 80,
                source: "c".into(),
            },
        ]);
        assert_eq!(
            placed
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["2", "3", "1"]
        );
    }

    #[tokio::test]
    async fn assembly_is_deterministic_and_excludes_reasoning() {
        let history = vec![
            Turn {
                id: "u".into(),
                role: InferenceRole::User,
                text: "hello".into(),
                semantic_kind: SemanticKind::User,
                tool_group_id: None,
            },
            Turn {
                id: "r".into(),
                role: InferenceRole::Assistant,
                text: "secret reasoning".into(),
                semantic_kind: SemanticKind::Thinking,
                tool_group_id: None,
            },
            Turn {
                id: "a".into(),
                role: InferenceRole::Assistant,
                text: "answer".into(),
                semantic_kind: SemanticKind::AssistantAnswer,
                tool_group_id: None,
            },
        ];
        let sources = Sources { history };
        let capabilities = capabilities();
        let builder = ContextBuilder {
            capabilities: &capabilities,
            estimator: &CharsHeuristic,
            sources: &sources,
            summarizer: None,
        };
        let request = AssembleRequest {
            conversation_id: "c".into(),
            branch_id: "main".into(),
            assistant_id: "default".into(),
            system_prompt: "You are helpful.".into(),
            user_text: "question".into(),
            thinking_requested: true,
            strategy: StrategyConfig::default(),
        };
        let first = builder.assemble(&request).await.expect("assembles");
        let second = builder.assemble(&request).await.expect("assembles twice");
        assert_eq!(first, second);
        let rendered = serde_json::to_string(&first.prompt).expect("serializes");
        assert!(!rendered.contains("secret reasoning"));
        assert_eq!(first.plan.thinking_reserve, 0);
    }
}
