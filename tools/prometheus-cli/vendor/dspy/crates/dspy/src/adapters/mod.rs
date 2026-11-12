// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
use crate::error::Result;
use crate::models::{ModelRequest, ModelResponse};
use crate::primitives::{Example, Prediction};
use crate::signatures::Signature;

pub mod chat;

pub use chat::ChatAdapter;

/// Drives the interaction between a Signature and a Language Model.
pub trait Adapter: Send + Sync {
    /// format the input example into a model request (e.g. prompt or messages)
    fn format(
        &self,
        signature: &Signature,
        demos: &[Example],
        input: &Example,
    ) -> Result<ModelRequest>;

    /// parse the model response into a prediction
    fn parse(&self, signature: &Signature, response: &ModelResponse) -> Result<Prediction>;
}
