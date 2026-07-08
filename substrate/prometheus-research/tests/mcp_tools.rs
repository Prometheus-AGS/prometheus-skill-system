/// Integration tests for MCP tool logic.
///
/// Tests call the underlying job/checkpoint functions that the MCP tool
/// handlers delegate to — the same code path exercised when the MCP server
/// dispatches a research_start / research_status / research_cancel call.
use prometheus_research::job::{cancel::cancel_job, checkpoint, spawn::spawn_job};

fn unique_prefix(label: &str) -> String {
    format!(
        "mcp-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

/// research_start logic: spawn_job returns a non-empty job_id.
#[test]
fn research_start_returns_job_id() {
    let job_id = spawn_job("test query for mcp start", "shallow", 5, "apa").unwrap();
    assert!(!job_id.is_empty(), "job_id must be non-empty");
    assert!(
        job_id.starts_with("job-"),
        "job_id should start with 'job-', got: {job_id}"
    );
    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}

/// research_status logic: checkpoint fields are readable after a job is created.
#[test]
fn research_status_returns_stage_fields() {
    let job_id = unique_prefix("sts");
    let cp = checkpoint::JobCheckpoint {
        job_id: job_id.clone(),
        query: "status test query".into(),
        depth: "moderate".into(),
        max_sources: 10,
        citation_style: "mla".into(),
        status: "running".into(),
        stage: 5,
        stage_name: "citation_builder".into(),
        progress: 75,
        pid: None,
        started_at: "2026-07-08T00:00:00Z".into(),
        last_updated_at: "2026-07-08T00:00:00Z".into(),
        tokens_used: 8192,
        sources_found: 12,
        output_dir: format!("~/.research-jobs/{job_id}/"),
    };
    checkpoint::write(&cp).unwrap();

    let read_back = checkpoint::read(&job_id).unwrap();
    assert_eq!(read_back.status, "running");
    assert_eq!(read_back.stage, 5);
    assert_eq!(read_back.stage_name, "citation_builder");
    assert_eq!(read_back.progress, 75);
    assert_eq!(read_back.tokens_used, 8192);
    assert_eq!(read_back.sources_found, 12);

    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}

/// research_cancel logic: cancel_job sets status to "cancelled".
#[test]
fn research_cancel_returns_cancelled_true() {
    let job_id = unique_prefix("cancel");
    let cp = checkpoint::JobCheckpoint {
        job_id: job_id.clone(),
        query: "cancel me".into(),
        depth: "shallow".into(),
        max_sources: 3,
        citation_style: "apa".into(),
        status: "running".into(),
        stage: 2,
        stage_name: "source_discovery".into(),
        progress: 20,
        pid: None,
        started_at: "2026-07-08T00:00:00Z".into(),
        last_updated_at: "2026-07-08T00:00:00Z".into(),
        tokens_used: 512,
        sources_found: 2,
        output_dir: format!("~/.research-jobs/{job_id}/"),
    };
    checkpoint::write(&cp).unwrap();

    let result = cancel_job(&job_id);
    assert!(result.is_ok(), "cancel_job failed: {:?}", result.err());

    let after = checkpoint::read(&job_id).unwrap();
    assert_eq!(after.status, "cancelled");

    let _ = std::fs::remove_dir_all(checkpoint::job_dir(&job_id));
}
