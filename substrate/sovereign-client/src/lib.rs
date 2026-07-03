//! Sovereign-sync client SDK.
//!
//! Connects to a running `sovereign-sync --mode server` or `--mode daemon`
//! instance and provides typed access to the REST API and AG-UI SSE stream.
mod client;
mod error;
mod types;

pub use client::SovereignClient;
pub use error::ClientError;
pub use types::{AgUiEvent, PeerInfo, SkillResult, SyncStatus};
