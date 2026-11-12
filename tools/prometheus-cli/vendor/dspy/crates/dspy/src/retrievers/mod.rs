// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub long_text: String,
    pub score: f64,
    pub pid: Option<i64>, // Passage ID if available
                          // Additional metadata can be handled via a generic or HashMap if needed,
                          // keeping it simple for now as per Python DSPy standard fields.
}

#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve top-k passages for a query.
    async fn retrieve(&self, query: &str, k: usize) -> Result<Vec<SearchResult>>;
}
