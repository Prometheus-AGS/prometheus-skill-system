use thiserror::Error;

pub type Result<T> = std::result::Result<T, ContractError>;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("unsupported schema version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("canonical JSON serialization failed: {0}")]
    Canonicalization(#[from] serde_json::Error),
    #[error("invalid digest: {0}")]
    InvalidDigest(String),
    #[error("invalid public key encoding: {0}")]
    InvalidPublicKey(String),
    #[error("invalid signature encoding: {0}")]
    InvalidSignature(String),
    #[error("signature algorithm does not match the supplied key")]
    AlgorithmMismatch,
    #[error("key id mismatch: expected {expected}, got {actual}")]
    KeyIdMismatch { expected: String, actual: String },
    #[error("signature verification failed")]
    SignatureVerification,
    #[error("receipt invariant failed: {0}")]
    ReceiptInvariant(String),
    #[error("unsafe artifact path: {0}")]
    UnsafeArtifactPath(String),
    #[error("artifact I/O failed for {path}: {source}")]
    ArtifactIo {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("receipt log verification failed: {0}")]
    ReceiptLog(String),
}
