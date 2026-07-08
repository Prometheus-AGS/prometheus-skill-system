use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

pub fn job_dir(job_id: &str) -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".research-jobs")
        .join(job_id)
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
    checkpoint.last_updated_at = chrono_now();
    write(&checkpoint)
}

fn chrono_now() -> String {
    // Simple ISO-8601 timestamp without chrono dependency
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("1970-01-01T00:00:{:02}Z", d.as_secs() % 60))
        .unwrap_or_else(|_| "unknown".to_string())
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
            started_at: "2026-07-08T00:00:00Z".into(),
            last_updated_at: "2026-07-08T00:00:00Z".into(),
            tokens_used: 0,
            sources_found: 0,
            output_dir: format!("~/.research-jobs/{job_id}/"),
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
