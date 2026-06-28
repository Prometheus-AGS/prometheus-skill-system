use async_trait::async_trait;
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("CRDT merge error: {0}")]
    Crdt(String),
    #[error("Backend unavailable: {0}")]
    Unavailable(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Abstraction over any key-value storage backend.
/// Keys are string paths; values are raw bytes (typically CRDT documents).
#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn read(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn write(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    fn backend_name(&self) -> &'static str;
}

/// Abstraction over a CRDT engine (currently automerge-rs; could be Loro).
pub trait CrdtEngine: Send + Sync {
    /// Create a new empty document.
    fn new_doc(&self) -> Vec<u8>;
    /// Merge a remote delta into a local document. Returns merged bytes.
    fn merge(&self, local: &[u8], remote_delta: &[u8]) -> Result<Vec<u8>>;
    /// Apply a JSON patch to a document. Returns new bytes + change delta.
    fn apply_json(&self, doc: &[u8], patch: serde_json::Value) -> Result<(Vec<u8>, Vec<u8>)>;
    /// Extract the current state of a document as JSON.
    fn to_json(&self, doc: &[u8]) -> Result<serde_json::Value>;
    fn engine_name(&self) -> &'static str;
}
