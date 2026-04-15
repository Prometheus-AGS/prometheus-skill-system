//! Multi-platform AI agent adapter library.
//!
//! Detects installed AI coding agents (Claude Code, OpenCode, Cursor, etc.)
//! and provides unified skill installation, listing, and removal across all platforms.
//! Includes a cross-platform trace capture protocol for the self-learning pipeline.

pub mod config;
pub mod detect;
pub mod platforms;
pub mod trace_protocol;

pub use config::{AgentConfig, AgentKind};
pub use detect::detect_installed_agents;
pub use trace_protocol::{PlatformTrace, TraceCapture};
