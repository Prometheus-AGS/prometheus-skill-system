// TJ-ARCH-MOB-001 compliant
//! Provider-neutral run and model lifecycle contracts. These are operational
//! state, not synthetic assistant prose, so every surface can render and
//! persist failures/cancellation without contaminating answer content.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Preparing,
    Thinking,
    Generating,
    Complete,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    Queued,
    ContextAssembly,
    DocumentExtraction,
    Ocr,
    EmbeddingPreparation,
    MemoryRetrieval,
    KnowledgeRetrieval,
    ToolSelection,
    Summarization,
    ModelDownload,
    ModelVerification,
    ModelLoading,
    FallbackSelection,
    Thinking,
    Generation,
    ToolExecution,
    Persistence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunError {
    pub code: String,
    pub phase: RunPhase,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,
}

impl RunError {
    pub fn protocol(message: impl Into<String>) -> Self {
        Self {
            code: "protocol_error".into(),
            phase: RunPhase::Generation,
            message: message.into(),
            retryable: false,
            diagnostics: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    pub model_id: String,
    pub backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offloaded_layers: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gate_report: Vec<RuntimeGateReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeGateStatus {
    Pending,
    Passed,
    Failed,
    Skipped,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeGateReport {
    /// Stable gate id used by native verification scripts and UI diagnostics.
    pub gate: u8,
    pub name: String,
    pub status: RuntimeGateStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ModelOperationState {
    NotInstalled,
    Downloading {
        bytes_received: u64,
        total_bytes: u64,
        bytes_per_second: u64,
        resumable: bool,
    },
    Verifying,
    Loading,
    Ready {
        diagnostics: RuntimeDiagnostics,
    },
    Cancelling,
    Cancelled,
    Failed {
        error: RunError,
    },
}

impl Default for ModelOperationState {
    fn default() -> Self {
        Self::NotInstalled
    }
}
