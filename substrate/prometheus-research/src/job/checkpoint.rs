use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobCheckpoint {
    pub job_id: String,
    pub query: String,
    pub depth: String,
    pub max_sources: u32,
    pub citation_style: String,
    pub status: String,
    pub stage: u32,
    pub stage_name: String,
    pub progress: u32,
    pub pid: Option<u32>,
    pub started_at: String,
    pub last_updated_at: String,
    pub tokens_used: u64,
    pub sources_found: u32,
    pub output_dir: String,
    /// The research package directory the driver created for this job
    /// (`<output_root>/<slug>-<yyyymmdd>-<4hex>/`), once the daemon has found it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    /// Harness binary the daemon resolved for this job (`claude` or `codex`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Pid of the harness process (distinct from `pid`, the `--daemon-job` process).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_pid: Option<u32>,
    /// Harness exit code once it has exited; `None` while running or on a signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Why the job is `blocked` or `failed`; cleared on `complete`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Cycle and wall-clock budgets, and whether one forced completion.
    ///
    /// Optional and defaulted so checkpoints written before change-drt-002
    /// still deserialize: a job from an older binary simply has no budgets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budgets: Option<Budgets>,
}

/// Budgets bounding a job, its director, and each thread.
///
/// Starting values are adopted from Onyx and verified at source (analysis
/// D-07). They are defaults, not findings: the open question on the change
/// records that per-depth director cycles are not yet cost-validated, and the
/// first real run should report actual spend before they are fixed.
///
/// | Field | Value | Onyx source |
/// |---|---|---|
/// | `thread_cycles` | 8 | `research_agent.py` cycle cap |
/// | `thread_force_report_minutes` | 12 | `research_agent.py:91` |
/// | `thread_timeout_minutes` | 30 | `research_agent.py:88` |
/// | `job_force_report_minutes` | 30 | `dr_loop.py:85` |
///
/// Director cycles vary by depth (shallow 2, deep 4, exhaustive 8); Onyx's own
/// orchestrator caps are 8 and 4 (`dr_loop.py:97,100`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budgets {
    /// Search/reflect cycles one thread may spend.
    pub thread_cycles: u32,
    /// When a thread must stop searching and write up what it has.
    pub thread_force_report_minutes: u64,
    /// Hard per-thread wall clock; the scheduler kills the worker at this point.
    pub thread_timeout_minutes: u64,
    /// Director cycles for this job's depth.
    pub director_cycles: u32,
    /// When the job must stop dispatching and assemble what it has.
    pub job_force_report_minutes: u64,
    /// Set true when any budget forced completion, so the report can say so.
    #[serde(default)]
    pub budget_hit: bool,
}

impl Default for Budgets {
    fn default() -> Self {
        Self {
            thread_cycles: 8,
            thread_force_report_minutes: 12,
            thread_timeout_minutes: 30,
            director_cycles: 4,
            job_force_report_minutes: 30,
            budget_hit: false,
        }
    }
}

impl Budgets {
    /// Budgets for a research depth.
    ///
    /// Only director cycles vary: a deeper run gets more orchestration passes,
    /// not longer individual threads, because thread length is bounded by how
    /// much a worker can hold accurately rather than by how deep the job is.
    #[must_use]
    pub fn for_depth(depth: &str) -> Self {
        let director_cycles = match depth {
            "shallow" => 2,
            "exhaustive" => 8,
            // "deep" and anything unrecognised: the middle setting, so an
            // unknown depth is bounded rather than unbounded.
            _ => 4,
        };
        Self {
            director_cycles,
            ..Self::default()
        }
    }
}

/// Root under which every research package and daemon job directory lives.
/// `RESEARCH_OUTPUT_DIR` overrides it; the default is `~/.prometheus/research/`,
/// the same convention as `~/.prometheus/learn` and `~/.prometheus/kbd`.
pub fn output_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("RESEARCH_OUTPUT_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".prometheus")
        .join("research")
}

/// The legacy root the daemon and hooks used before change-rah-002. Nothing
/// reads it any more; `status` names it while it still exists so an operator
/// can delete it by hand (analysis D-14: no migration).
pub fn legacy_output_root() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".research-jobs") // legacy root; never read, only named in status output
}

pub fn job_dir(job_id: &str) -> PathBuf {
    output_root().join(job_id)
}

pub fn checkpoint_path(job_id: &str) -> PathBuf {
    job_dir(job_id).join("checkpoint.json")
}

pub fn read(job_id: &str) -> anyhow::Result<JobCheckpoint> {
    let path = checkpoint_path(job_id);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("checkpoint not found for job {job_id}"))?;
    let checkpoint: JobCheckpoint = serde_json::from_str(&content)
        .with_context(|| format!("invalid checkpoint JSON for job {job_id}"))?;
    Ok(checkpoint)
}

pub fn write(checkpoint: &JobCheckpoint) -> anyhow::Result<()> {
    let dir = job_dir(&checkpoint.job_id);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("checkpoint.json");
    let content = serde_json::to_string_pretty(checkpoint)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn update_status(job_id: &str, status: &str) -> anyhow::Result<()> {
    let mut checkpoint = read(job_id)?;
    checkpoint.status = status.to_string();
    checkpoint.last_updated_at = now_rfc3339();
    write(&checkpoint)
}

/// Current wall-clock time as an RFC 3339 UTC timestamp with second precision,
/// e.g. `2026-09-04T17:30:12Z`. Every checkpoint timestamp in this crate comes
/// from here so the format is uniform and parseable with
/// `chrono::DateTime::parse_from_rfc3339`.
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_checkpoint(job_id: &str) -> JobCheckpoint {
        JobCheckpoint {
            job_id: job_id.to_string(),
            query: "test query".into(),
            depth: "shallow".into(),
            max_sources: 5,
            citation_style: "apa".into(),
            status: "running".into(),
            stage: 1,
            stage_name: "planner".into(),
            progress: 0,
            pid: None,
            started_at: now_rfc3339(),
            last_updated_at: now_rfc3339(),
            tokens_used: 0,
            sources_found: 0,
            output_dir: format!("~/.prometheus/research/{job_id}/"),
            package_dir: None,
            package_id: None,
            harness: None,
            harness_pid: None,
            exit_code: None,
            error: None,
        }
    }

    #[test]
    fn checkpoint_round_trip() {
        let job_id = format!("test-rtt-{}", std::process::id());
        let cp = make_test_checkpoint(&job_id);
        write(&cp).unwrap();
        let read_back = read(&job_id).unwrap();
        assert_eq!(read_back.query, "test query");
        assert_eq!(read_back.status, "running");
        let _ = std::fs::remove_dir_all(job_dir(&job_id));
    }

    #[test]
    fn update_status_changes_status_field() {
        let job_id = format!("test-upd-{}", std::process::id());
        let cp = make_test_checkpoint(&job_id);
        write(&cp).unwrap();
        update_status(&job_id, "cancelled").unwrap();
        let read_back = read(&job_id).unwrap();
        assert_eq!(read_back.status, "cancelled");
        let _ = std::fs::remove_dir_all(job_dir(&job_id));
    }

    #[test]
    fn read_returns_error_for_missing_job() {
        let result = read("non-existent-job-00000000");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("non-existent-job-00000000"));
    }
}
