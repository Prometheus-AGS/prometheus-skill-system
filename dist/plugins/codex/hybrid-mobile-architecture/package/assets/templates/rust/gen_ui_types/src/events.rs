// TJ-ARCH-MOB-001 compliant
//! Raw stream + protocol event enums. Pure data — transformation logic is in
//! gen_ui_protocol (which depends on this crate).
use crate::lifecycle::{RunError, RunPhase, RunStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreamCitation {
    pub source: String,
    pub quote: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart,
    TextDelta {
        index: u32,
        delta: String,
    },
    ThinkingDelta {
        index: u32,
        delta: String,
    },
    ToolCallStarted {
        id: String,
        name: String,
    },
    ToolCallDelta {
        id: String,
        delta: String,
    },
    ToolCallComplete {
        id: String,
    },
    ToolCallReady {
        id: String,
        name: String,
        input_json: String,
    },
    ToolResult {
        tool_use_id: String,
        output_json: String,
        is_error: bool,
    },
    SkillActivation {
        name: String,
        status: String,
    },
    Citation {
        source: String,
        quote: String,
    },
    Citations {
        citations: Vec<StreamCitation>,
    },
    Memory {
        operation: String,
        key: String,
        value: Option<String>,
    },
    Custom {
        source: String,
        event_name: String,
        payload: serde_json::Value,
    },
    Error {
        message: String,
    },
    Cancelled {
        message: String,
    },
    Done,
}

/// A2UI event surface (subset shown; full 27-variant set filled in gen_ui_protocol
/// consumers). Kept as an open enum contract here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum A2uiEvent {
    RunStarted {
        run_id: String,
    },
    RunPhaseChanged {
        run_id: String,
        phase: RunPhase,
        status: RunStatus,
        message: String,
    },
    Block {
        run_id: String,
        block: crate::content_block::ContentBlock,
    },
    RunFinished {
        run_id: String,
    },
    RunCancelled {
        run_id: String,
        phase: RunPhase,
        message: String,
    },
    RunError {
        run_id: String,
        error: RunError,
    },
    ContextTelemetry {
        run_id: String,
        payload: serde_json::Value,
    },
}

impl A2uiEvent {
    pub fn run_id(&self) -> &str {
        match self {
            Self::RunStarted { run_id }
            | Self::RunPhaseChanged { run_id, .. }
            | Self::Block { run_id, .. }
            | Self::RunFinished { run_id }
            | Self::RunCancelled { run_id, .. }
            | Self::RunError { run_id, .. }
            | Self::ContextTelemetry { run_id, .. } => run_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AguiEvent {
    RunStarted {
        thread_id: String,
        run_id: String,
    },
    TextMessageContent {
        delta: String,
    },
    ReasoningMessageContent {
        delta: String,
    },
    ToolCallStart {
        id: String,
        name: String,
    },
    ToolCallArgs {
        id: String,
        delta: String,
    },
    ToolCallEnd {
        id: String,
    },
    ToolCallResult {
        id: String,
        output_json: String,
        is_error: bool,
    },
    Custom {
        name: String,
        value: serde_json::Value,
    },
    StateSnapshot {
        snapshot_json: String,
    },
    RunFinished {
        run_id: String,
    },
    RunCancelled {
        run_id: String,
        message: String,
    },
    RunError {
        run_id: String,
        error_json: String,
    },
}
