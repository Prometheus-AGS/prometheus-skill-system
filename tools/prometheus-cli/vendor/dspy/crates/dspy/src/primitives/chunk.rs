use super::usage::Usage;
use super::value::Value;
use serde::{Deserialize, Serialize};

/// A chunk of a streamed response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Chunk {
    #[serde(rename = "content")]
    Content(String),
    #[serde(rename = "reasoning")]
    Reasoning(String),
    #[serde(rename = "citation")]
    Citation {
        source: String,
        text: String,
        metadata: Value,
    },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "tool_result")]
    ToolResult { id: String, result: String },
    #[serde(rename = "usage")]
    Usage(Usage),
    #[serde(rename = "finish")]
    Finish(FinishReason),
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "status")]
    Status {
        phase: String,
        message: String,
        state: Option<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Other(String),
}
