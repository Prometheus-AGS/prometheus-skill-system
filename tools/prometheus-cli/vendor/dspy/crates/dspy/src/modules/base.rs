use crate::error::Result;
use crate::primitives::{Chunk, Example, Prediction};
use async_trait::async_trait;
use futures::stream::BoxStream;

#[async_trait]
pub trait Module: Send + Sync {
    async fn forward(&self, input: &Example) -> Result<Prediction>;
    async fn stream(&self, _input: &Example) -> Result<BoxStream<'static, Result<Chunk>>> {
        Err(crate::error::DspyError::General(
            "Streaming not implemented for this module".into(),
        ))
    }
}
