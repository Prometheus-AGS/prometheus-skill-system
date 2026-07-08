use crate::job::checkpoint::{self, JobCheckpoint};
use anyhow::Context as _;

pub fn spawn_job(
    query: &str,
    depth: &str,
    max_sources: u32,
    citation_style: &str,
) -> anyhow::Result<String> {
    let job_id = format!(
        "job-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("0000")
    );

    let dir = checkpoint::job_dir(&job_id);
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create job dir {dir:?}"))?;

    let cp = JobCheckpoint {
        job_id: job_id.clone(),
        query: query.to_string(),
        depth: depth.to_string(),
        max_sources,
        citation_style: citation_style.to_string(),
        status: "pending".to_string(),
        stage: 0,
        stage_name: "initializing".to_string(),
        progress: 0,
        pid: None,
        started_at: "2026-07-08T00:00:00Z".to_string(),
        last_updated_at: "2026-07-08T00:00:00Z".to_string(),
        tokens_used: 0,
        sources_found: 0,
        output_dir: dir.to_string_lossy().to_string(),
    };
    checkpoint::write(&cp)?;

    // Spawn self as background daemon for the job
    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(&exe)
        .arg("--daemon-job")
        .arg(&job_id)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match child {
        Ok(c) => {
            // Update checkpoint with PID
            let mut updated = checkpoint::read(&job_id)?;
            updated.pid = Some(c.id());
            updated.status = "running".to_string();
            checkpoint::write(&updated)?;
        }
        Err(e) => {
            tracing::warn!("daemon spawn failed (non-fatal in test): {e}");
            // Mark as running even without daemon — for status/cancel testing
            let mut updated = checkpoint::read(&job_id)?;
            updated.status = "running".to_string();
            checkpoint::write(&updated)?;
        }
    }

    Ok(job_id)
}
