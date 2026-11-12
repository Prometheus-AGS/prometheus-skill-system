use thiserror::Error;

#[derive(Debug, Error)]
pub enum DspyError {
    #[error("Adapter failed to parse response: {0}")]
    AdapterParse(String),

    #[error("Adapter failed to format request: {0}")]
    AdapterFormat(String),

    #[error("Model call failed: {0}")]
    ModelCall(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Output field not found: {0}")]
    OutputNotFound(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Retrieval error: {0}")]
    Retrieval(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, DspyError>;
