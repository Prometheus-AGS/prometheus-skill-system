//! Transport-independent execution orchestration.
//!
//! The core depends inward on `prometheus-exec-contracts` and exposes ports
//! implemented by execution tiers. It contains no transport, OS sandbox, KBD,
//! or Sovereign Sync dependency.

#![forbid(unsafe_code)]

mod artifact;
mod grant;
mod job;
mod policy;
mod port;
mod receipt;
mod receipt_log;

pub use job::{ExecutionJob, JobValidationError, ValidatedExecutionJob};
pub use policy::{
    BaselinePolicy, CedarPolicyError, CedarTighteningPolicy, PolicyEvaluator, PolicyOutcome,
    PolicyReason, DEFAULT_EXECUTION_POLICY,
};
pub use port::{BackendExecution, ExecutionPort, ProducedArtifact};
pub use receipt::{Ed25519ReceiptSigner, ReceiptAssembler, ReceiptAssemblyError, ReceiptSigner};
pub use receipt_log::{AppendReceiptResult, ReceiptLog, ReceiptLogError};

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub use artifact::{
    ArtifactBudgetProfile, ArtifactStore, CasError, GcReport, StoredArtifact,
    DESKTOP_ARTIFACT_BUDGET_BYTES, MOBILE_ARTIFACT_BUDGET_BYTES,
};
pub use grant::{
    verify_interactive_grant, GrantValidationError, InteractiveGrantIssuer,
    InteractiveGrantStatement, InteractiveGrantToken, SshGrantManifest, SshGrantVerifier,
    ValidatedGrant, GRANT_NAMESPACE,
};
