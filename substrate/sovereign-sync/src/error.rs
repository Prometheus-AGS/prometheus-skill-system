use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Privacy violation: domain '{0}' is LocalOnly and cannot be synced")]
    PrivacyViolation(String),
    #[error("CRDT error: {0}")]
    Crdt(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Config error: {0}")]
    Config(#[from] anyhow::Error),
}
