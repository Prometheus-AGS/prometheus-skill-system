//! Durable, transport-independent execution service.
//!
//! REST, MCP, FFI, and CLI adapters call this layer. It depends on the core
//! port rather than a concrete executor, so surface parity is testable and
//! Tier W never acquires a Tier P dependency.

#![forbid(unsafe_code)]

mod ledger;

pub use ledger::{
    AcceptRunResult, ReconciliationReport, RunLedger, RunLedgerError, RunRecord, SpawnStatus,
    TerminalCommitResult, TerminalReceiptRecord,
};

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
