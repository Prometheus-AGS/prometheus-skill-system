//! Integration test for `--daemon-job` (change-rah-004).
//!
//! One test function, three phases, run sequentially because they set
//! process-wide environment (`PATH`, `RESEARCH_OUTPUT_DIR`, ...) that the
//! spawned daemon inherits:
//!
//! 1. A fake `claude` (tests/fixtures/fake-claude.sh) is the only binary on
//!    PATH. A job is started through the production REST path against a real
//!    HTTP server; the real `prometheus-research` binary runs as
//!    `--daemon-job`, resolves the fake harness, and the fake harness runs the
//!    real deep-research driver with the fixture stage runner. The test waits
//!    for `complete` and asserts on the checkpoint, the SSE stream, the hook
//!    markers, the KBD-hook contract, and `research_export`.
//! 2. With an empty PATH the same start path yields `blocked` with the reason,
//!    and `/health` still answers.
//! 3. A required field removed from the completed package's manifest makes
//!    `research_export` refuse, naming the field.
//!
//! Everything asserted is a file the daemon or driver wrote, a checkpoint the
//! server served, or bytes that came over the SSE socket.

use prometheus_research::{
    a2ui::registry::ComponentRegistry,
    agui::AguiEvent,
    http_server::{
        health, rest,
        sse::{self, EventBroadcast},
    },
    job::checkpoint,
    mcp_server::{tools::ResearchExportParams, validate_package, ResearchMcpServer},
};

use axum::{
    routing::{delete, get, post},
    Router,
};
use futures::StreamExt as _;
use rmcp::handler::server::wrapper::Parameters;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

async fn spawn_test_server() -> (String, EventBroadcast) {
    let (tx, _rx) = broadcast::channel::<AguiEvent>(1024);
    let state = rest::AppState {
        broadcast: tx.clone(),
        registry: ComponentRegistry::new(),
        // Nothing listens here; emit_to_surface_bridge logs and moves on.
        surface_bridge_url: "http://127.0.0.1:1".into(),
    };
    let app = Router::new()
        .route("/health", get(health::health_handler))
        .route("/api/v1/jobs", post(rest::create_job))
        .route("/api/v1/jobs/{id}", get(rest::get_job))
        .route("/api/v1/jobs/{id}", delete(rest::delete_job))
        .route("/api/v1/jobs/{id}/events", get(sse::sse_handler))
        .route("/api/v1/jobs/{id}/events", post(sse::ingest_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://127.0.0.1:{port}"), tx)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

async fn create_job(base: &str, query: &str) -> String {
    let resp = client()
        .post(format!("{base}/api/v1/jobs"))
        .json(&serde_json::json!({ "query": query, "depth": "deep", "citation_style": "apa" }))
        .send()
        .await
        .expect("POST /api/v1/jobs");
    assert_eq!(resp.status(), 201, "create_job status");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["job_id"].as_str().expect("job_id").to_string()
}

async fn wait_for_terminal(base: &str, job_id: &str, timeout: Duration) -> serde_json::Value {
    let start = Instant::now();
    loop {
        let resp = client()
            .get(format!("{base}/api/v1/jobs/{job_id}"))
            .send()
            .await
            .expect("GET job");
        let body: serde_json::Value = resp.json().await.unwrap();
        let status = body["status"].as_str().unwrap_or("");
        if matches!(status, "complete" | "blocked" | "failed" | "cancelled") {
            return body;
        }
        if start.elapsed() > timeout {
            let dir = checkpoint::job_dir(job_id);
            let daemon_log = std::fs::read_to_string(dir.join("daemon.log")).unwrap_or_default();
            let harness_log =
                std::fs::read_to_string(dir.join("harness.log")).unwrap_or_default();
            panic!(
                "job {job_id} did not reach a terminal state in {timeout:?}; last status {status}\n\
                 --- daemon.log ---\n{daemon_log}\n--- harness.log ---\n{harness_log}"
            );
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn daemon_job_runs_the_driver_through_a_headless_harness() {
    let repo = repo_root();
    let deep = repo.join("skills/research/deep-research");
    let driver = deep.join("scripts/run-research.sh");
    let runner = deep.join("tests/fixtures/stage-runner.sh");
    let judge = deep.join("tests/fixtures/judge.sh");
    for p in [&driver, &runner, &judge] {
        assert!(p.is_file(), "fixture missing: {}", p.display());
    }

    let root = std::env::temp_dir().join(format!(
        "prometheus-research-jobexec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let fakebin = root.join("bin");
    std::fs::create_dir_all(&fakebin).unwrap();
    let fake = fakebin.join("claude");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-claude.sh"),
        &fake,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let hook_log = root.join("hooks.log");
    let fake_log = root.join("fake-claude.log");
    let original_path = std::env::var("PATH").unwrap_or_default();

    // The environment the daemon, the harness, and the driver inherit.
    std::env::set_var("RESEARCH_OUTPUT_DIR", &root);
    std::env::set_var(
        "RESEARCH_DAEMON_EXE",
        env!("CARGO_BIN_EXE_prometheus-research"),
    );
    std::env::set_var("RESEARCH_DRIVER", &driver);
    std::env::set_var("RESEARCH_STAGE_RUNNER", format!("bash {}", runner.display()));
    std::env::set_var("RESEARCH_JUDGE_CMD", format!("bash {}", judge.display()));
    std::env::set_var("KBD_PRODUCER_MODEL", "fixture/producer");
    std::env::set_var("RESEARCH_HOOK_LOG", &hook_log);
    std::env::set_var("FIXTURE_SUBQ", "4");
    std::env::set_var("FIXTURE_UNRESOLVED", "1");
    std::env::set_var("FAKE_CLAUDE_PATH", &original_path);
    std::env::set_var("FAKE_CLAUDE_LOG", &fake_log);
    std::env::remove_var("KBD_HOOKS_DISABLED");

    let (base, tx) = spawn_test_server().await;
    std::env::set_var("RESEARCH_EVENT_SINK", &base);
    std::env::set_var("RESEARCH_EVENT_TOKEN", "test-token-for-job-execution");

    // The ingest sink refuses a caller without the server's token, and a
    // mismatched job id, before anything reaches the broadcast.
    let forged = client()
        .post(format!("{base}/api/v1/jobs/forged/events"))
        .json(&serde_json::json!({"type":"agent.message","job_id":"forged","message":"x","level":"info","timestamp":"t"}))
        .send()
        .await
        .unwrap();
    assert_eq!(forged.status(), 401, "ingest without the token is refused");
    let mismatched = client()
        .post(format!("{base}/api/v1/jobs/forged/events"))
        .header("x-research-event-token", "test-token-for-job-execution")
        .json(&serde_json::json!({"type":"agent.message","job_id":"other","message":"x","level":"info","timestamp":"t"}))
        .send()
        .await
        .unwrap();
    assert_eq!(mismatched.status(), 400, "ingest with a mismatched job id is refused");

    // ---- phase 1: fake harness on PATH → complete --------------------------
    std::env::set_var("PATH", &fakebin);

    // Collect every broadcast event before the job exists, so nothing is missed.
    let events: Arc<Mutex<Vec<AguiEvent>>> = Arc::new(Mutex::new(Vec::new()));
    {
        let mut rx = tx.subscribe();
        let events = events.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ev) => events.lock().unwrap().push(ev),
                    // A slow machine may lag the 1024-slot channel; that is not
                    // end-of-stream, and dropping the rest would fake a daemon bug.
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    let query = "daemon execution fixture query about vector databases";
    let job_id = create_job(&base, query).await;

    // The SSE transport itself: read the stream until the completing event.
    let sse_text = {
        let base = base.clone();
        let job_id = job_id.clone();
        tokio::spawn(async move {
            let resp = client()
                .get(format!("{base}/api/v1/jobs/{job_id}/events"))
                .send()
                .await
                .expect("SSE request");
            assert_eq!(resp.status(), 200);
            let mut stream = resp.bytes_stream();
            let mut collected = String::new();
            let deadline = Instant::now() + Duration::from_secs(180);
            while Instant::now() < deadline {
                match tokio::time::timeout(Duration::from_secs(5), stream.next()).await {
                    Ok(Some(Ok(bytes))) => {
                        collected.push_str(&String::from_utf8_lossy(&bytes));
                        if collected.contains("\"status\":\"complete\"")
                            || collected.contains("\"status\":\"blocked\"")
                            || collected.contains("\"status\":\"failed\"")
                        {
                            break;
                        }
                    }
                    Ok(Some(Err(_))) | Ok(None) => break,
                    Err(_) => continue,
                }
            }
            collected
        })
    };

    let final_cp = wait_for_terminal(&base, &job_id, Duration::from_secs(180)).await;
    let sse_text = sse_text.await.unwrap();
    // Let the last broadcast land before reading the collection.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let daemon_log = std::fs::read_to_string(checkpoint::job_dir(&job_id).join("daemon.log"))
        .unwrap_or_default();
    assert_eq!(
        final_cp["status"], "complete",
        "job did not complete: {final_cp}\n--- daemon.log ---\n{daemon_log}"
    );
    assert_eq!(final_cp["harness"], "claude");
    assert!(final_cp["pid"].as_u64().is_some(), "daemon recorded its pid");
    assert!(final_cp["harness_pid"].as_u64().is_some(), "daemon recorded the harness pid");
    assert_eq!(final_cp["stage"], 10);
    assert_eq!(final_cp["progress"], 100);
    assert_eq!(final_cp["exit_code"], 0);
    let package_dir = PathBuf::from(final_cp["package_dir"].as_str().expect("package_dir"));
    assert!(
        package_dir.starts_with(&root) && package_dir.join("manifest.json").is_file(),
        "package under the output root with a manifest: {}",
        package_dir.display()
    );
    let pkg_cp: serde_json::Value =
        serde_json::from_str(&read(&package_dir.join("checkpoint.json"))).unwrap();
    assert_eq!(pkg_cp["job_id"], job_id, "driver recorded the daemon's job id");
    assert_eq!(pkg_cp["status"], "complete");
    // The job was created with the entry points' lowercase default; the daemon
    // canonicalised it to the schema's vocabulary before the driver saw it.
    assert_eq!(pkg_cp["citation_style"], "APA", "citation style canonicalised for the driver");
    assert_eq!(pkg_cp["depth"], "deep");
    assert_eq!(pkg_cp["stages_completed"].as_array().map(|a| a.len()), Some(10));

    // The harness received the contract.
    let fake = read(&fake_log);
    assert!(fake.contains("invocation: -p"), "fake claude was invoked with -p:\n{fake}");
    assert!(fake.contains("KBD_HOOKS_DISABLED=1"), "KBD_HOOKS_DISABLED reached the harness:\n{fake}");
    assert!(fake.contains(&format!("RESEARCH_JOB_ID={job_id}")));
    assert!(fake.contains(&format!("RESEARCH_OUTPUT_DIR={}", root.display())));
    assert!(fake.contains(query), "prompt carries the query");
    assert!(fake.contains("- depth: deep"), "prompt carries the depth");
    assert!(fake.contains(&format!("- output root: {}", root.display())), "prompt carries the output root");
    assert!(fake.contains("--job-id "), "prompt names the driver command with --job-id");
    let prompt_file = checkpoint::job_dir(&job_id).join("prompt.md");
    assert!(prompt_file.is_file(), "prompt.md written beside the checkpoint");
    assert!(
        !root.join("kbd-hook.marker").exists(),
        "no KBD hook marker: the harness saw KBD_HOOKS_DISABLED=1"
    );

    // The four deep-research hooks fired through the driver.
    let hooks = read(&hook_log);
    for marker in ["pre-research", "post-stage 01", "post-stage 10", "on-contradiction 06", "post-export"] {
        assert!(hooks.contains(marker), "hook marker {marker:?} missing:\n{hooks}");
    }

    // SSE carried agent.status over the socket, and the broadcast carried at
    // least one agent.status per executed stage (1..=10).
    assert!(
        sse_text.contains("event: agent.status") && sse_text.contains("\"status\":\"complete\""),
        "SSE stream lacks agent.status / complete:\n{sse_text}"
    );
    let stages_seen: std::collections::BTreeSet<u32> = events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AguiEvent::AgentStatus { job_id: j, stage, .. } if j == &job_id => Some(*stage),
            _ => None,
        })
        .collect();
    for stage in 1..=10u32 {
        assert!(stages_seen.contains(&stage), "no agent.status for stage {stage}; saw {stages_seen:?}");
    }
    let errors: Vec<String> = events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            AguiEvent::AgentError { job_id: j, message, .. } if j == &job_id => Some(message.clone()),
            _ => None,
        })
        .collect();
    assert!(errors.is_empty(), "no agent.error on a clean run, got {errors:?}");

    // research_export returns the validated package.
    let server = ResearchMcpServer::new();
    let exported: serde_json::Value = serde_json::from_str(
        &server
            .research_export(Parameters(ResearchExportParams {
                job_id: job_id.clone(),
                format: "markdown".into(),
            }))
            .await,
    )
    .unwrap();
    assert!(exported.get("error").is_none(), "export refused a valid package: {exported}");
    assert_eq!(exported["package_dir"], package_dir.display().to_string());
    assert_eq!(exported["output_path"], package_dir.join("report.md").display().to_string());
    assert!(
        matches!(exported["verification_status"].as_str(), Some("verified" | "partial" | "unverified")),
        "verification_status from the manifest: {exported}"
    );
    assert!(
        matches!(exported["verification_verdict"].as_str(), Some("PASS" | "PASS WITH NOTES" | "BLOCKED")),
        "verification_verdict from the manifest: {exported}"
    );

    // ---- phase 1b: a hostile depth never reaches the prompt ----------------
    let hostile = client()
        .post(format!("{base}/api/v1/jobs"))
        .json(&serde_json::json!({ "query": "injection attempt fixture query", "depth": "deep; touch /tmp/rah004-injected", "citation_style": "apa" }))
        .send()
        .await
        .unwrap();
    assert_eq!(hostile.status(), 201);
    let hostile_id = hostile.json::<serde_json::Value>().await.unwrap()["job_id"].as_str().unwrap().to_string();
    let hostile_cp = wait_for_terminal(&base, &hostile_id, Duration::from_secs(60)).await;
    assert_eq!(hostile_cp["status"], "blocked", "hostile depth: {hostile_cp}");
    assert!(hostile_cp["error"].as_str().unwrap_or("").contains("depth"), "reason names the field: {hostile_cp}");
    assert!(!checkpoint::job_dir(&hostile_id).join("prompt.md").exists(), "no prompt was written for a rejected job");
    assert!(!Path::new("/tmp/rah004-injected").exists(), "nothing was executed");
    let fake_after = read(&fake_log);
    assert_eq!(fake_after.matches("invocation: -p").count(), 1, "the harness was not invoked for the rejected job");

    // ---- phase 2: empty PATH → blocked, daemon keeps serving ---------------
    std::env::set_var("PATH", "");
    let blocked_id = create_job(&base, "no harness fixture query").await;
    let blocked = wait_for_terminal(&base, &blocked_id, Duration::from_secs(60)).await;
    assert_eq!(blocked["status"], "blocked", "empty PATH: {blocked}");
    let reason = blocked["error"].as_str().unwrap_or("");
    assert!(reason.contains("no harness binary on PATH"), "reason names the cause: {reason}");
    assert!(blocked.get("harness").is_none() || blocked["harness"].is_null());
    let health: serde_json::Value = client()
        .get(format!("{base}/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["status"], "ok", "/health after a blocked job");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let blocked_error_events = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, AguiEvent::AgentError { job_id: j, .. } if j == &blocked_id))
        .count();
    assert_eq!(blocked_error_events, 1, "one agent.error for the blocked job");
    let blocked_daemon_log =
        std::fs::read_to_string(checkpoint::job_dir(&blocked_id).join("daemon.log")).unwrap_or_default();
    assert!(!blocked_daemon_log.contains("panicked"), "daemon must not panic:\n{blocked_daemon_log}");
    std::env::set_var("PATH", &original_path);

    // ---- phase 3: an invalid manifest is refused, naming the field --------
    let manifest_path = package_dir.join("manifest.json");
    let mut manifest: serde_json::Value = serde_json::from_str(&read(&manifest_path)).unwrap();
    let saved = manifest.clone();
    manifest.as_object_mut().unwrap().remove("query");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    let err = validate_package(&package_dir).expect_err("manifest without query must be refused");
    assert!(format!("{err:#}").contains("query"), "error names the missing field: {err:#}");
    let refused: serde_json::Value = serde_json::from_str(
        &server
            .research_export(Parameters(ResearchExportParams {
                job_id: job_id.clone(),
                format: "json".into(),
            }))
            .await,
    )
    .unwrap();
    assert!(
        refused["error"].as_str().unwrap_or("").contains("query"),
        "research_export names the field: {refused}"
    );
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&saved).unwrap()).unwrap();
    assert!(validate_package(&package_dir).is_ok(), "restored manifest validates again");

    let _ = std::fs::remove_dir_all(&root);
}
