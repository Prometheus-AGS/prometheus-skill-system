// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
//! DSPy OpenAI optimizer subset for the Prometheus Skill System.

pub mod adapters;
pub mod error;
pub mod evaluations;
pub mod models;
pub mod modules;
pub mod optimizers;
pub mod primitives;
pub mod retrievers;
pub mod signatures;

pub use adapters::{Adapter, ChatAdapter};
pub use error::{DspyError, Result};
pub use evaluations::{Dataset, Evaluate, Metric};
#[cfg(feature = "openai")]
pub use models::OpenAIModel;
pub use models::{LanguageModel, ModelRequest, ModelResponse, Provider};
pub use modules::{Module, Predict};
pub use optimizers::BootstrapFewShot;
pub use primitives::{Example, Prediction, Record, Value};
pub use signatures::Signature;
