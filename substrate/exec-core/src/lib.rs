//! Transport-independent execution orchestration.
//!
//! The core depends inward on `prometheus-exec-contracts` and exposes ports
//! implemented by execution tiers. It contains no transport, OS sandbox, KBD,
//! or Sovereign Sync dependency.

#![forbid(unsafe_code)]

mod artifact;
mod job;
mod policy;
mod port;
mod receipt;

pub use job::{ExecutionJob, JobValidationError, ValidatedExecutionJob};
pub use policy::{BaselinePolicy, PolicyEvaluator, PolicyOutcome, PolicyReason};
pub use port::{BackendExecution, ExecutionPort, ProducedArtifact};
pub use receipt::{Ed25519ReceiptSigner, ReceiptAssembler, ReceiptAssemblyError, ReceiptSigner};

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub use artifact::{ArtifactStore, CasError, GcReport, StoredArtifact};
