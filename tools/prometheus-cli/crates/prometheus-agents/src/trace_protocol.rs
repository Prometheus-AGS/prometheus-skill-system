//! Cross-platform execution trace capture protocol.
//!
//! Each platform adapter implements `TraceCapture` to extract execution
//! traces from its native session format. The self-learning pipeline
//! ingests data from ALL platforms — not just Claude Code.

use crate::config::AgentKind;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// A platform-agnostic execution trace extracted from an AI agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTrace {
    pub platform: AgentKind,
    pub session_id: String,
    pub skill_invocations: Vec<SkillInvocation>,
    pub tool_calls_total: usize,
    pub duration_ms: u64,
    pub captured_at: String,
    pub project_path: Option<String>,
}

/// A single skill invocation within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInvocation {
    pub skill_name: String,
    pub args: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub success: bool,
}

/// Trait for platform-specific trace extraction.
pub trait TraceCapture {
    fn platform(&self) -> AgentKind;
    fn is_available(&self) -> bool;
    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>>;
    fn capture_since(&self, since: &str) -> anyhow::Result<Vec<PlatformTrace>>;
}

// ─── Claude Code Implementation ─────────────────────────────────────────────

/// Claude Code trace capture — parses session JSONL files.
///
/// Session data format (verified):
/// - Location: ~/.claude/projects/{project-path-slug}/{session-id}.jsonl
/// - Entry types: user, assistant, attachment, queue-operation, system, last-prompt
/// - Tool use: inside `assistant` entries → `message.content[]` → `{type: "tool_use", name: "...", input: {...}}`
/// - Skill invocations: tool_use entries with `name: "Skill"` and `input.skill` field
pub struct ClaudeCodeTraceCapture {
    projects_dir: PathBuf,
}

impl ClaudeCodeTraceCapture {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            projects_dir: home.join(".claude").join("projects"),
        }
    }

    /// Find the most recent session JSONL file across all projects.
    fn find_latest_session(&self) -> anyhow::Result<Option<(PathBuf, String)>> {
        if !self.projects_dir.exists() {
            return Ok(None);
        }

        let mut latest: Option<(PathBuf, std::time::SystemTime)> = None;

        for project_entry in std::fs::read_dir(&self.projects_dir)? {
            let project_entry = project_entry?;
            if !project_entry.file_type()?.is_dir() { continue; }

            for file_entry in std::fs::read_dir(project_entry.path())? {
                let file_entry = file_entry?;
                let path = file_entry.path();
                if path.extension().is_some_and(|e| e == "jsonl") {
                    if let Ok(meta) = file_entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if latest.as_ref().is_none_or(|(_, t)| modified > *t) {
                                latest = Some((path, modified));
                            }
                        }
                    }
                }
            }
        }

        Ok(latest.map(|(path, _)| {
            let session_id = path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (path, session_id)
        }))
    }

    /// Parse a session JSONL file into a PlatformTrace.
    fn parse_session(&self, path: &Path, session_id: &str) -> anyhow::Result<PlatformTrace> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        let mut skill_invocations = Vec::new();
        let mut tool_calls_total = 0usize;
        let mut project_path: Option<String> = None;
        let mut first_timestamp: Option<String> = None;
        let mut last_timestamp: Option<String> = None;

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() { continue; }

            let entry: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let timestamp = entry.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");

            if !timestamp.is_empty() {
                if first_timestamp.is_none() {
                    first_timestamp = Some(timestamp.to_string());
                }
                last_timestamp = Some(timestamp.to_string());
            }

            // Extract project path from attachment entries
            if entry_type == "attachment" {
                if let Some(cwd) = entry.get("cwd").and_then(|c| c.as_str()) {
                    project_path = Some(cwd.to_string());
                }
            }

            // Parse tool_use blocks from assistant messages
            if entry_type == "assistant" {
                let content = entry
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array());

                if let Some(blocks) = content {
                    for block in blocks {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            tool_calls_total += 1;

                            let tool_name = block.get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("");

                            // Track Skill invocations specifically
                            if tool_name == "Skill" {
                                let input = block.get("input").cloned()
                                    .unwrap_or(serde_json::Value::Object(Default::default()));
                                let skill_name = input.get("skill")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let args = input.get("args")
                                    .and_then(|a| a.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                skill_invocations.push(SkillInvocation {
                                    skill_name,
                                    args,
                                    tool_calls: vec![], // populated by correlating tool_results
                                    timestamp: timestamp.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(PlatformTrace {
            platform: AgentKind::ClaudeCode,
            session_id: session_id.to_string(),
            skill_invocations,
            tool_calls_total,
            duration_ms: 0, // would need to parse timestamps to compute
            captured_at: chrono::Utc::now().to_rfc3339(),
            project_path,
        })
    }
}

impl TraceCapture for ClaudeCodeTraceCapture {
    fn platform(&self) -> AgentKind {
        AgentKind::ClaudeCode
    }

    fn is_available(&self) -> bool {
        self.projects_dir.exists()
    }

    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>> {
        let Some((path, session_id)) = self.find_latest_session()? else {
            return Ok(None);
        };

        let trace = self.parse_session(&path, &session_id)?;
        Ok(Some(trace))
    }

    fn capture_since(&self, _since: &str) -> anyhow::Result<Vec<PlatformTrace>> {
        // Iterate all session files, filter by modification time
        // For now, return latest only
        let mut traces = Vec::new();
        if let Some(trace) = self.capture_latest()? {
            traces.push(trace);
        }
        Ok(traces)
    }
}

// ─── Generic file-based capture ─────────────────────────────────────────────

/// File-based trace capture for platforms that write structured execution logs.
pub struct FileBasedTraceCapture {
    platform: AgentKind,
    log_dir: PathBuf,
}

impl FileBasedTraceCapture {
    pub fn new(platform: AgentKind, log_dir: PathBuf) -> Self {
        Self { platform, log_dir }
    }
}

impl TraceCapture for FileBasedTraceCapture {
    fn platform(&self) -> AgentKind { self.platform }
    fn is_available(&self) -> bool { self.log_dir.exists() }
    fn capture_latest(&self) -> anyhow::Result<Option<PlatformTrace>> { Ok(None) }
    fn capture_since(&self, _since: &str) -> anyhow::Result<Vec<PlatformTrace>> { Ok(vec![]) }
}

/// Build trace capture implementations for all detected platforms.
pub fn build_trace_captures() -> Vec<Box<dyn TraceCapture>> {
    let mut captures: Vec<Box<dyn TraceCapture>> = vec![
        Box::new(ClaudeCodeTraceCapture::new()),
    ];

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
