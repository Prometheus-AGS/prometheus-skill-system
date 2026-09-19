use crate::job::checkpoint::{self, JobCheckpoint};
use anyhow::Context as _;

/// Start a job: write its checkpoint, then re-execute this binary as
/// `--daemon-job <job_id>` (see `job::daemon`), which runs the harness.
///
/// The daemon executable is this process's own binary; `RESEARCH_DAEMON_EXE`
/// overrides it so an integration test (whose `current_exe` is the test
/// harness) can point at the real `prometheus-research` binary. The child
/// inherits the environment, so `RESEARCH_OUTPUT_DIR`, `RESEARCH_EVENT_SINK`,
/// `PATH`, and the deep-research variables all flow through unchanged.
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

    let started_at = checkpoint::now_rfc3339();
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
        started_at: started_at.clone(),
        last_updated_at: started_at,
        tokens_used: 0,
        sources_found: 0,
        output_dir: dir.to_string_lossy().to_string(),
        package_dir: None,
        package_id: None,
        harness: None,
        harness_pid: None,
        exit_code: None,
        error: None,
        // Depth selects director cycles; a job carries its budgets from the
        // moment it is created so a reader never has to guess what bounded it.
        budgets: Some(checkpoint::Budgets::for_depth(depth)),
    };
    checkpoint::write(&cp)?;

    // Everything after the checkpoint exists is one failure domain: a job whose
    // daemon never started must read `failed` with the reason, never a
    // `pending` that nothing will ever advance.
    let launch = || -> anyhow::Result<u32> {
        let exe = match std::env::var_os("RESEARCH_DAEMON_EXE") {
            Some(p) if !p.is_empty() => std::path::PathBuf::from(p),
            _ => std::env::current_exe().context("cannot resolve the daemon executable")?,
        };
        // The daemon's own log is kept beside the checkpoint; the harness writes
        // harness.log in the same directory.
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("daemon.log"))
            .with_context(|| format!("cannot open daemon.log in {}", dir.display()))?;
        let log_err = log.try_clone().context("cannot clone the daemon.log handle")?;
        let child = std::process::Command::new(&exe)
            .arg("--daemon-job")
            .arg(&job_id)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::from(log))
            .stderr(std::process::Stdio::from(log_err))
            .spawn()
            .with_context(|| format!("daemon spawn failed ({})", exe.display()))?;
        Ok(child.id())
    };

    match launch() {
        Ok(pid) => {
            // Nothing is written here on success. The daemon child records its
            // own pid and every later status; a parent write after the spawn
            // would race the child and could overwrite a terminal status the
            // child already reached (a job blocked on a missing harness is
            // blocked within milliseconds). The job stays `pending` until the
            // daemon takes it over.
            tracing::debug!("daemon for {job_id} spawned as pid {pid}");
        }
        Err(e) => {
            let mut updated = checkpoint::read(&job_id)?;
            updated.status = "failed".to_string();
            updated.error = Some(format!("{e:#}"));
            updated.last_updated_at = checkpoint::now_rfc3339();
            checkpoint::write(&updated)?;
            tracing::warn!("daemon launch failed for {job_id}: {e:#}");
        }
    }

    Ok(job_id)
}
