//! Durable, transport-independent execution service.
//!
//! REST, MCP, FFI, and CLI adapters call this layer. It depends on the core
//! port rather than a concrete executor, so surface parity is testable and
//! Tier W never acquires a Tier P dependency.

#![forbid(unsafe_code)]

mod event_log;
mod http;
mod ledger;
mod service;

pub use event_log::{AppendEventResult, RunEvent, RunEventData, RunEventLog, RunEventLogError};
pub use http::{
    build_api_router, ApiErrorDetail, ApiErrorEnvelope, ReadinessSnapshot, ReadinessStatus,
    RunResponse, SidecarState, SubsystemReadiness,
};
#[cfg(unix)]
pub use http::{peer_is_same_user, UdsSidecar, UdsSidecarError};
pub use ledger::{
    AcceptRunResult, ReconciliationReport, RunLedger, RunLedgerError, RunRecord, SpawnStatus,
    TerminalCommitResult, TerminalReceiptRecord,
};
pub use service::{ExecutionService, ExecutionServiceError, SubmitRunResult};

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
