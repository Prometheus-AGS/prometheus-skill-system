use chrono::Datelike as _;
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

/// A job started through the production path carries real timestamps: both
/// `started_at` and `last_updated_at` parse as RFC 3339 and fall within 60 s
/// of this test's own clock. This is the regression gate for the fabricated
/// `1970-01-01T00:00:SSZ` and hardcoded `2026-07-08T00:00:00Z` values that
/// every checkpoint carried before change-rah-001.
#[test]
fn spawned_job_timestamps_are_current_rfc3339() {
    let prefix = unique_job_prefix("ts");
    let before = chrono::Utc::now();
    let job_id = spawn_job(&format!("query for {prefix}"), "shallow", 3, "apa").unwrap();
    let after = chrono::Utc::now();

    let cp = checkpoint::read(&job_id).unwrap();
    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));

    for (field, value) in [
        ("started_at", &cp.started_at),
        ("last_updated_at", &cp.last_updated_at),
    ] {
        let parsed = chrono::DateTime::parse_from_rfc3339(value)
            .unwrap_or_else(|e| panic!("{field} {value:?} is not RFC 3339: {e}"))
            .with_timezone(&chrono::Utc);
        assert!(
            parsed.year() > 1970,
            "{field} {value:?} is the fabricated epoch-relative timestamp"
        );
        let skew = (parsed - before)
            .num_seconds()
            .abs()
            .max((after - parsed).num_seconds().abs());
        assert!(
            skew <= 60,
            "{field} {value:?} is {skew}s away from the test clock window {before}..{after}"
        );
    }
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
        started_at: checkpoint::now_rfc3339(),
        last_updated_at: checkpoint::now_rfc3339(),
        tokens_used: 1234,
        sources_found: 7,
        output_dir: format!("~/.prometheus/research/{job_id}/"),
        ..Default::default()
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
        started_at: checkpoint::now_rfc3339(),
        last_updated_at: checkpoint::now_rfc3339(),
        tokens_used: 0,
        sources_found: 0,
        output_dir: format!("~/.prometheus/research/{job_id}/"),
        ..Default::default()
    };
    checkpoint::write(&cp).unwrap();

    cancel_job(&job_id).unwrap();

    let after = checkpoint::read(&job_id).unwrap();
    assert_eq!(after.status, "cancelled");

    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}
