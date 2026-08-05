use thiserror::Error;

#[derive(Debug, Error)]
pub enum RemoteError {
    #[error("remote contract failed: {0}")]
    Contract(String),
    #[error("execution contract failed: {0}")]
    ExecutionContract(#[from] prometheus_exec_contracts::ContractError),
    #[error("remote JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("remote signature is invalid: {0}")]
    Signature(String),
    #[error("endpoint is not enrolled: {0}")]
    UnknownEndpoint(String),
    #[error("endpoint signing key does not match enrollment for {0}")]
    SignerMismatch(String),
    #[error("remote dispatch has expired")]
    Expired,
}

pub type Result<T> = std::result::Result<T, RemoteError>;
