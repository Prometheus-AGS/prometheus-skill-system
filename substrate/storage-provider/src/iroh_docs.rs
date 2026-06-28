use crate::traits::{CrdtEngine, Result, StorageError, StorageProvider};
use async_trait::async_trait;

pub struct IrohDocsAdapter;

impl IrohDocsAdapter {
    pub fn new() -> Self {
        IrohDocsAdapter
    }
}

impl Default for IrohDocsAdapter {
    fn default() -> Self {
        IrohDocsAdapter::new()
    }
}

#[async_trait]
impl StorageProvider for IrohDocsAdapter {
    async fn read(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    async fn write(&self, _key: &str, _value: Vec<u8>) -> Result<()> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    async fn list_keys(&self, _prefix: &str) -> Result<Vec<String>> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    fn backend_name(&self) -> &'static str {
        "iroh-docs"
    }
}

impl CrdtEngine for IrohDocsAdapter {
    fn new_doc(&self) -> Vec<u8> {
        // Stub: returns empty bytes; real implementation requires iroh-docs integration
        vec![]
    }

    fn merge(&self, _local: &[u8], _remote_delta: &[u8]) -> Result<Vec<u8>> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    fn apply_json(&self, _doc: &[u8], _patch: serde_json::Value) -> Result<(Vec<u8>, Vec<u8>)> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    fn to_json(&self, _doc: &[u8]) -> Result<serde_json::Value> {
        Err(StorageError::Unavailable(
            "iroh-docs not yet implemented".into(),
        ))
    }

    fn engine_name(&self) -> &'static str {
        "iroh-docs"
    }
}
