use thiserror::Error;
use uuid::Uuid;

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
    #[error("dispatch hash conflict for {dispatch_id}: existing {existing}, supplied {supplied}")]
    DispatchHashConflict {
        dispatch_id: Uuid,
        existing: prometheus_exec_contracts::Digest,
        supplied: prometheus_exec_contracts::Digest,
    },
    #[error("request {request_id} was already accepted as dispatch {existing_dispatch_id}")]
    RequestReplay {
        request_id: Uuid,
        existing_dispatch_id: Uuid,
    },
    #[error("dispatch was not found: {0}")]
    DispatchNotFound(Uuid),
    #[error("invalid dispatch state transition: {0}")]
    InvalidTransition(String),
    #[error("remote queue I/O failed at {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("remote queue segment is corrupt: {0}")]
    CorruptSegment(String),
}

pub type Result<T> = std::result::Result<T, RemoteError>;
