//! Native execution adapter.
//!
//! Implementations must select a supported OS sandbox before process spawn.
//! This crate never provides an unsandboxed fallback that can emit an
//! attested receipt.

#![forbid(unsafe_code)]

mod linux;

pub use linux::{
    BwrapConfig, BwrapPlan, LandlockClassification, LandlockCompatibility, LandlockProbe,
    LandlockRulesetStatus, LinuxSandboxError, LinuxSandboxSelection,
};

#[cfg(target_os = "linux")]
mod landlock_status;

#[cfg(target_os = "linux")]
pub use landlock_status::classify_landlock_restriction;

#[cfg(target_os = "macos")]
mod seatbelt;

#[cfg(target_os = "macos")]
pub use seatbelt::{SeatbeltConfig, SeatbeltError, SeatbeltExecutor, SeatbeltProfile};

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
