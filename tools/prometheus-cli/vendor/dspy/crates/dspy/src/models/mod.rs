// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "openai")]
pub use openai::OpenAIModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

use crate::primitives::{Chunk, Usage};
use futures::stream::BoxStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse>;
    async fn stream(&self, request: ModelRequest) -> Result<BoxStream<'static, Result<Chunk>>>;
}

pub enum Provider {
    OpenAI,
    // Anthropic,
    // ...
}
