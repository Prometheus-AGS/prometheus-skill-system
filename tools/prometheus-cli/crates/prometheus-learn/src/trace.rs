//! Execution trace capture and storage.
//!
//! Every skill execution produces a trace containing: skill name, inputs,
//! outputs, tool calls, errors, duration, and final score. Traces are the
//! raw material for the knowledge compilation and optimization pipelines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A captured execution trace from a skill invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: String,
    pub skill_name: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,

    /// What the skill was asked to do
    pub input_summary: String,

    /// What the skill produced
    pub output_summary: String,

    /// Tool calls made during execution
    pub tool_calls: Vec<ToolCall>,

    /// Errors encountered (empty if successful)
    pub errors: Vec<String>,

    /// Classification: success, partial, failure
    pub classification: TraceClassification,

    /// Score from automated evaluation (0.0 - 1.0)
    pub score: Option<f64>,

    /// LLM reflection on failures (populated during evaluation)
    pub reflection: Option<String>,

    /// Project context
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub input_summary: String,
    pub output_summary: String,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceClassification {
    Success,
    Partial,
    Failure,
}

impl ExecutionTrace {
    /// Convert trace to markdown for knowledge compilation.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Execution Trace: {}\n\n", self.skill_name));
        md.push_str(&format!("**Date**: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        md.push_str(&format!("**Duration**: {}ms\n", self.duration_ms));
        md.push_str(&format!("**Classification**: {:?}\n", self.classification));
        if let Some(score) = self.score {
            md.push_str(&format!("**Score**: {:.2}\n", score));
        }
        md.push_str(&format!("\n## Input\n\n{}\n", self.input_summary));
        md.push_str(&format!("\n## Output\n\n{}\n", self.output_summary));

        if !self.tool_calls.is_empty() {
            md.push_str("\n## Tool Calls\n\n");
            for tc in &self.tool_calls {
                let status = if tc.success { "ok" } else { "FAIL" };
                md.push_str(&format!("- **{}** [{}] ({}ms): {}\n",
                    tc.tool_name, status, tc.duration_ms, tc.input_summary));
            }
        }

        if !self.errors.is_empty() {
            md.push_str("\n## Errors\n\n");
            for err in &self.errors {
                md.push_str(&format!("- {}\n", err));
            }
        }

        if let Some(ref reflection) = self.reflection {
            md.push_str(&format!("\n## Reflection\n\n{}\n", reflection));
        }

        md
    }
}

/// Filesystem-backed trace storage.
pub struct TraceStore {
    base_dir: PathBuf,
}

impl TraceStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self { base_dir: base_dir.into() }
    }

    /// Default location: .prometheus/traces/
    pub fn default_for_project(project_root: &Path) -> Self {
        Self::new(project_root.join(".prometheus").join("traces"))
    }

    /// Store an execution trace.
    pub fn store(&self, trace: &ExecutionTrace) -> anyhow::Result<PathBuf> {
        let skill_dir = self.base_dir.join(&trace.skill_name);
        std::fs::create_dir_all(&skill_dir)?;

        let filename = format!("{}.json", trace.timestamp.format("%Y%m%d-%H%M%S"));
        let path = skill_dir.join(&filename);
        let json = serde_json::to_string_pretty(trace)?;
        std::fs::write(&path, json)?;

        Ok(path)
    }

    /// Load all traces for a specific skill.
    pub fn load_for_skill(&self, skill_name: &str) -> anyhow::Result<Vec<ExecutionTrace>> {
        let skill_dir = self.base_dir.join(skill_name);
        if !skill_dir.exists() {
            return Ok(vec![]);
        }

        let mut traces = Vec::new();
        for entry in std::fs::read_dir(&skill_dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "json") {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(trace) = serde_json::from_str::<ExecutionTrace>(&content) {
                    traces.push(trace);
                }
            }
        }

        traces.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(traces)
    }

    /// Count total traces across all skills.
    pub fn count_all(&self) -> anyhow::Result<usize> {
        if !self.base_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                for file in std::fs::read_dir(entry.path())? {
                    let file = file?;
                    if file.path().extension().is_some_and(|e| e == "json") {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}
