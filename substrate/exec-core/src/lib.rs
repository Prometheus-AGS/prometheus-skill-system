//! Transport-independent execution orchestration.
//!
//! The core depends inward on `prometheus-exec-contracts` and exposes ports
//! implemented by execution tiers. It contains no transport, OS sandbox, KBD,
//! or Sovereign Sync dependency.

#![forbid(unsafe_code)]

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
