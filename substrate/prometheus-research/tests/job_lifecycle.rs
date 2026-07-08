use prometheus_research::job::{cancel::cancel_job, checkpoint, spawn::spawn_job};

fn unique_job_prefix(label: &str) -> String {
    format!(
        "test-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

/// After spawn_job, the checkpoint file must exist on disk.
#[test]
fn creates_checkpoint_on_start() {
    let prefix = unique_job_prefix("ccos");
    let job_id = spawn_job(&format!("query for {prefix}"), "shallow", 3, "apa").unwrap();
    let path = checkpoint::checkpoint_path(&job_id);
    assert!(path.exists(), "checkpoint not found at {path:?}");
    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}

/// Writing a checkpoint then reading it back via checkpoint::read returns the same fields.
#[test]
fn status_reads_checkpoint() {
    let job_id = unique_job_prefix("src");
    let cp = checkpoint::JobCheckpoint {
        job_id: job_id.clone(),
        query: "integration test query".into(),
        depth: "moderate".into(),
        max_sources: 10,
        citation_style: "apa".into(),
        status: "running".into(),
        stage: 3,
        stage_name: "synthesis".into(),
        progress: 42,
        pid: None,
        started_at: "2026-07-08T00:00:00Z".into(),
        last_updated_at: "2026-07-08T00:00:00Z".into(),
        tokens_used: 1234,
        sources_found: 7,
        output_dir: format!("~/.research-jobs/{job_id}/"),
    };
    checkpoint::write(&cp).unwrap();

    let read_back = checkpoint::read(&job_id).unwrap();
    assert_eq!(read_back.stage, 3);
    assert_eq!(read_back.stage_name, "synthesis");
    assert_eq!(read_back.progress, 42);
    assert_eq!(read_back.status, "running");
    assert_eq!(read_back.tokens_used, 1234);

    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}

/// After cancel_job, the checkpoint status field becomes "cancelled".
#[test]
fn cancel_updates_checkpoint_to_cancelled() {
    let job_id = unique_job_prefix("cucc");
    let cp = checkpoint::JobCheckpoint {
        job_id: job_id.clone(),
        query: "to be cancelled".into(),
        depth: "shallow".into(),
        max_sources: 5,
        citation_style: "apa".into(),
        status: "running".into(),
        stage: 1,
        stage_name: "planner".into(),
        progress: 10,
        pid: None,
        started_at: "2026-07-08T00:00:00Z".into(),
        last_updated_at: "2026-07-08T00:00:00Z".into(),
        tokens_used: 0,
        sources_found: 0,
        output_dir: format!("~/.research-jobs/{job_id}/"),
    };
    checkpoint::write(&cp).unwrap();

    cancel_job(&job_id).unwrap();

    let after = checkpoint::read(&job_id).unwrap();
    assert_eq!(after.status, "cancelled");

    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}
