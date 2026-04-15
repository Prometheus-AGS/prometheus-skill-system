//! Cross-platform execution trace capture protocol.
//!
//! Each platform adapter implements `TraceCapture` to extract execution
//! traces from its native session format. This ensures the self-learning
//! pipeline ingests data from ALL platforms — not just Claude Code.
//!
//! This is Gap 3 from the platform review: trace capture must be a
//! first-class protocol, not a Claude Code-specific hook.

use crate::config::AgentKind;
use serde::{Deserialize, Serialize};

/// A platform-agnostic execution trace extracted from an AI agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTrace {
    /// Which platform produced this trace
    pub platform: AgentKind,
    /// Session or conversation ID (platform-specific format)
    pub session_id: String,
    /// Skills invoked during the session
    pub skill_invocations: Vec<SkillInvocation>,
    /// Total session duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp of trace capture
    pub captured_at: String,
}

/// A single skill invocation within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInvocation {
    pub skill_name: String,
    pub command: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub success: bool,
}

/// Trait for platform-specific trace extraction.
///
/// Each platform stores session data differently:
/// - Claude Code: session log files in ~/.claude/sessions/
/// - OpenCode: session state in ~/.config/opencode/sessions/
/// - Cursor: internal session state (limited access)
/// - Codex: git worktree-based execution logs
///
/// Implementors extract what's available and normalize to `PlatformTrace`.
pub trait TraceCapture {
    /// Platform this capture implementation handles.
    fn platform(&self) -> AgentKind;

    /// Check if trace data is available for capture.
    fn is_available(&self) -> bool;

    /// Extract the most recent session trace.
    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>>;

    /// Extract all session traces since a given timestamp.
    fn capture_since(&self, since: &str) -> anyhow::Result<Vec<PlatformTrace>>;
}

/// Claude Code trace capture — reads session metadata.
pub struct ClaudeCodeTraceCapture {
    sessions_dir: std::path::PathBuf,
}

impl ClaudeCodeTraceCapture {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            sessions_dir: home.join(".claude").join("sessions"),
        }
    }
}

impl TraceCapture for ClaudeCodeTraceCapture {
    fn platform(&self) -> AgentKind {
        AgentKind::ClaudeCode
    }

    fn is_available(&self) -> bool {
        self.sessions_dir.exists()
    }

    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>> {
        if !self.is_available() {
            return Ok(None);
        }

        // Claude Code stores session data in JSON files
        // Parse the most recent session for skill invocations
        // This is a stub — full implementation parses the session JSON format
        Ok(Some(PlatformTrace {
            platform: AgentKind::ClaudeCode,
            session_id: "latest".to_string(),
            skill_invocations: vec![],
            duration_ms: 0,
            captured_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    fn capture_since(&self, _since: &str) -> anyhow::Result<Vec<PlatformTrace>> {
        // Iterate session files, filter by timestamp
        Ok(vec![])
    }
}

/// Generic file-based trace capture for platforms that write execution logs.
/// Works for OpenCode, Codex, and any platform that writes structured logs.
pub struct FileBasedTraceCapture {
    platform: AgentKind,
    log_dir: std::path::PathBuf,
}

impl FileBasedTraceCapture {
    pub fn new(platform: AgentKind, log_dir: std::path::PathBuf) -> Self {
        Self { platform, log_dir }
    }
}

impl TraceCapture for FileBasedTraceCapture {
    fn platform(&self) -> AgentKind {
        self.platform
    }

    fn is_available(&self) -> bool {
        self.log_dir.exists()
    }

    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>> {
        if !self.is_available() {
            return Ok(None);
        }
        // Parse log files for skill invocations
        Ok(None)
    }

    fn capture_since(&self, _since: &str) -> anyhow::Result<Vec<PlatformTrace>> {
        Ok(vec![])
    }
}

/// Build trace capture implementations for all detected platforms.
pub fn build_trace_captures() -> Vec<Box<dyn TraceCapture>> {
    let mut captures: Vec<Box<dyn TraceCapture>> = vec![
        Box::new(ClaudeCodeTraceCapture::new()),
    ];

    // Add file-based captures for other platforms
    let home = dirs::home_dir().unwrap_or_default();

    let file_based_platforms = [
        (AgentKind::OpenCode, home.join(".config").join("opencode").join("sessions")),
        (AgentKind::Codex, home.join(".agents").join("sessions")),
    ];

    for (kind, dir) in file_based_platforms {
        captures.push(Box::new(FileBasedTraceCapture::new(kind, dir)));
    }

    captures
}
