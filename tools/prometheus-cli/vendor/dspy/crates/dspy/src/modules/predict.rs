use super::base::Module;
use crate::adapters::{Adapter, ChatAdapter};
use crate::error::Result;
use crate::models::LanguageModel;
use crate::primitives::{Example, Prediction};
use crate::retrievers::Retriever;
use crate::signatures::Signature;
use async_trait::async_trait;
use std::sync::Arc;

/// The atomic unit of DSPy: a Predict module.
/// It wraps a Signature, a LanguageModel, and an Adapter.
#[derive(Clone)]
pub struct Predict {
    pub signature: Signature,
    pub model: Arc<dyn LanguageModel>,
    pub adapter: Arc<dyn Adapter>,
    pub demos: Vec<Example>,
    pub demo_retriever: Option<Arc<dyn Retriever>>,
    pub demo_limit: usize,
}

impl Predict {
    pub fn new(signature: Signature, model: Arc<dyn LanguageModel>) -> Self {
        Self {
            signature,
            model,
            adapter: Arc::new(ChatAdapter::new()),
            demos: Vec::new(),
            demo_retriever: None,
            demo_limit: 0,
        }
    }

    pub fn with_adapter(mut self, adapter: Arc<dyn Adapter>) -> Self {
        self.adapter = adapter;
        self
    }

    pub fn with_demos(mut self, demos: Vec<Example>) -> Self {
        self.demos = demos;
        self
    }

    pub fn with_demo_retriever(mut self, retriever: Arc<dyn Retriever>, k: usize) -> Self {
        self.demo_retriever = Some(retriever);
        self.demo_limit = k;
        self
    }
}

#[async_trait]
impl Module for Predict {
    async fn forward(&self, input: &Example) -> Result<Prediction> {
        let demos = if let Some(retriever) = &self.demo_retriever {
            // Retrieve dynamic demos
            // Using the input's string representation as query for now
            // Ideally we extract a specific field based on signature
            let query = input.to_string();
            let retrieved = retriever.retrieve(&query, self.demo_limit).await?;
            // Convert chunks to examples (assuming chunks are compatible or we just use raw text)
            // This is a simplification. Ideally retrieved Chunks map to Examples.
            // For parity, we need a way to store Example in Retriever or reconstruct it.
            // DSPy python often retrieves examples directly.
            // Our Retriever returns `Chunk`.
            // We'll map Chunk -> Example for now.
            retrieved
                .into_iter()
                .map(|c| Example::new_from_text(c.long_text))
                .collect()
        } else {
            self.demos.clone()
        };

        let request = self.adapter.format(&self.signature, &demos, input)?;
        let response = self.model.complete(request).await?;
        self.adapter.parse(&self.signature, &response)
    }

    async fn stream(
        &self,
        input: &Example,
    ) -> Result<futures::stream::BoxStream<'static, Result<crate::primitives::Chunk>>> {
        let demos = if let Some(retriever) = &self.demo_retriever {
            let query = input.to_string();
            let retrieved = retriever.retrieve(&query, self.demo_limit).await?;
            retrieved
                .into_iter()
                .map(|c| Example::new_from_text(c.long_text))
                .collect()
        } else {
            self.demos.clone()
        };

        let request = self.adapter.format(&self.signature, &demos, input)?;
        self.model.stream(request).await
    }
}
