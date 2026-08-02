/// Integration tests for sovereign-sync REST API and CRDT engine.
///
/// Tests 1-5 use Axum's oneshot() to test the router in-process.
/// Tests 6-8 test the CRDT and P2P utilities directly.
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tower::ServiceExt;

use sovereign_sync::crdt::{apply_incoming_delta, current_version, export_outgoing_delta};
use sovereign_sync::p2p::P2PNode;
use sovereign_sync::rest_api::{build_router, AppState};
use storage_provider::{DomainConfig, PrivacyClass, SyncDomain, SyncManifest};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn test_router() -> (axum::Router, TempDir) {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let project_root = fixture.path().join("project");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&project_root).unwrap();
    let state = AppState::try_new_at(&skills_dir, &project_root, &data_root, None)
        .await
        .unwrap();
    (build_router(state), fixture)
}

fn default_manifest() -> SyncManifest {
    let mut manifest = SyncManifest::new();
    manifest.register(
        SyncDomain::new("learner-model"),
        DomainConfig::new(PrivacyClass::Trusted, "learner/"),
    );
    manifest.register(
        SyncDomain::new("surreal-memory"),
        DomainConfig::new(PrivacyClass::Local, "memory/"),
    );
    manifest
}

// ---------------------------------------------------------------------------
// 1. Health endpoint returns 200 + service name
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_endpoint_returns_200() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["service"], "sovereign-sync");
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn loopback_listener_can_be_acquired_before_application_state_exists() {
    let listener = sovereign_sync::rest_api::bind_loopback(0).await.unwrap();
    let address = listener.local_addr().unwrap();
    assert!(address.ip().is_loopback());
    let connection = tokio::net::TcpStream::connect(address).await.unwrap();
    drop(connection);
    drop(listener);
}

#[tokio::test]
async fn startup_router_serves_health_and_gates_stateful_routes_until_install() {
    let (app, gate) = sovereign_sync::rest_api::build_startup_router();
    let health = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(health).await.unwrap().status(),
        StatusCode::OK
    );

    let ready = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    let ready = app.clone().oneshot(ready).await.unwrap();
    assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = axum::body::to_bytes(ready.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["startup"]["stage"], "listener_bound");
    assert_eq!(json["startup"]["skillIndex"], "pending");

    gate.set_project_progress(18, 12, 1).await;
    gate.fail("local authority initialization failed").await;
    let ready = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    let ready = app.clone().oneshot(ready).await.unwrap();
    assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = axum::body::to_bytes(ready.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "failed");
    assert_eq!(json["startup"]["projectTotal"], 18);
    assert_eq!(json["startup"]["openedProjects"], 12);
    assert_eq!(json["startup"]["failedProjects"], 1);
    assert_eq!(
        json["startup"]["terminalError"],
        "local authority initialization failed"
    );

    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let project_root = fixture.path().join("project");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&project_root).unwrap();
    let state = AppState::try_new_at(&skills_dir, &project_root, &data_root, None)
        .await
        .unwrap();
    gate.install(state).await;

    let ready = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.oneshot(ready).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dedicated_http_runtime_keeps_liveness_available_in_diagnostic_mode() {
    let mut service = sovereign_sync::rest_api::HttpService::start(0)
        .await
        .unwrap();
    let port = service.address().port();
    service
        .gate()
        .fail("local authority initialization failed")
        .await;

    let report = sovereign_sync::health_check::detect_daemon_health(port).await;
    assert_eq!(
        report.status,
        sovereign_sync::health_check::DaemonHealthKind::Healthy
    );

    let response = tokio::time::timeout(std::time::Duration::from_millis(500), async move {
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port)).await?;
        let request = b"GET /ready HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = stream;
        stream.write_all(request).await?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        Ok::<_, std::io::Error>(response)
    })
    .await
    .expect("diagnostic readiness exceeded 500 ms")
    .unwrap();
    let response = String::from_utf8(response).unwrap();
    assert!(response.starts_with("HTTP/1.1 503"));
    assert!(response.contains("\"stage\":\"failed\""));

    service.shutdown().await.unwrap();
}

#[tokio::test]
async fn ready_endpoint_replays_the_journal_asynchronously() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("GET")
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ready");
    assert_eq!(json["projectCount"], 1);
    assert_eq!(json["projects"][0]["revision"], 0);
}

#[tokio::test]
async fn registry_routes_two_projects_without_a_focus_environment_variable() {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let first = fixture.path().join("first");
    let second = fixture.path().join("second");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let state = AppState::try_new_at(&skills_dir, &first, &data_root, None)
        .await
        .unwrap();
    let second_runtime = kbd_runtime::Runtime::open_canonical_at(&second, &data_root).unwrap();
    let second_id = second_runtime
        .project_manifest(false)
        .unwrap()
        .unwrap()
        .project_id;
    let app = build_router(state);

    let register = Request::builder()
        .method("POST")
        .uri("/api/v1/kbd/projects/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::json!({"path":second}).to_string()))
        .unwrap();
    let response = app.clone().oneshot(register).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let projects = Request::builder()
        .method("GET")
        .uri("/api/v1/kbd/projects")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(projects).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 16_384)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["projects"].as_array().unwrap().len(), 2);

    let replicas = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/kbd/projects/{second_id}/replicas"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(replicas).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let submodules = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/kbd/projects/{second_id}/submodules"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(submodules).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 16_384)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["projectId"], second_id);
    assert!(json["pins"].as_object().unwrap().is_empty());

    let ready = Request::builder()
        .method("GET")
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(ready).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 16_384)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["projectCount"], 2);
}

#[tokio::test]
async fn adoption_route_is_a_non_mutating_dry_run_by_default() {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let target = fixture.path().join("target");
    let source = fixture.path().join("source");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::create_dir_all(&source).unwrap();

    let state = AppState::try_new_at(&skills_dir, &target, &data_root, None)
        .await
        .unwrap();
    let target_id = kbd_runtime::Runtime::open_canonical_at(&target, &data_root)
        .unwrap()
        .project_manifest(false)
        .unwrap()
        .unwrap()
        .project_id;
    let source_runtime = kbd_runtime::Runtime::open_canonical_at(&source, &data_root).unwrap();
    let source_manifest = source_runtime.project_manifest(false).unwrap().unwrap();
    let app = build_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/kbd/projects/adopt")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "path": source,
                "intoProjectId": target_id
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["dryRun"], true);
    assert_eq!(
        json["outcome"]["formerProjectId"],
        source_manifest.project_id
    );
    assert_eq!(json["outcome"]["intoProjectId"], target_id);
    assert_eq!(json["outcome"]["sourceEventCount"], 0);
    assert!(!json["outcome"]["warnings"].as_array().unwrap().is_empty());
    let current_manifest: kbd_runtime::ProjectManifest =
        serde_json::from_slice(&std::fs::read(source.join(".prometheus/project.json")).unwrap())
            .unwrap();
    assert_eq!(
        current_manifest, source_manifest,
        "dry-run adoption must not rewrite the declared project UUID"
    );
}

// ---------------------------------------------------------------------------
// 2. Sync status returns idle state and domains map
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sync_status_returns_idle() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/sync/status")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // This test fixture builds an `AppState` with no P2P node (server-mode
    // style), so `node_state` honestly reports that rather than a fake
    // "idle" — real domain-adapter wiring replaced the old hardcoded stub.
    assert_eq!(json["node_state"], "no-p2p");
    assert!(
        json["domains"].is_object(),
        "domains object should be present"
    );
}

// ---------------------------------------------------------------------------
// 3. Sync peers returns empty list initially
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sync_peers_returns_empty_list() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/sync/peers")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["peers"].is_array());
}

// ---------------------------------------------------------------------------
// 4. Skills search returns results array with count
// ---------------------------------------------------------------------------

#[tokio::test]
async fn skills_search_returns_results_array() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/skills/search?q=learn")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["results"].is_array(), "results should be an array");
    assert!(json["count"].is_number(), "count should be present");
}

// ---------------------------------------------------------------------------
// 5. Sync push queues a domain and echoes its name
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sync_push_queues_domain() {
    let (app, _fixture) = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/sync/push")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"domain":"learner-model"}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Real push pipeline now runs (domain registration, adapter export, CRDT
    // merge) instead of the old hardcoded "queued" stub. No P2P node in this
    // test fixture, so it merges locally rather than broadcasting.
    assert_eq!(json["status"], "applied-locally-only");
    assert_eq!(json["domain"], "learner-model");
    assert!(json["snapshotBytes"].as_u64().unwrap_or(0) > 0);
}

// ---------------------------------------------------------------------------
// KBD project routes
//
// `/api/v1/kbd/projects/{project_id}/...` had no coverage at all before these
// tests, so the 404 branches and the command envelope/path check were free to
// regress silently. They are also the regression net for making the control
// plane multi-project: today one `AppState` serves exactly one project, and
// `kbd_status` compares the path id against that single runtime rather than
// using it as a lookup key.
// ---------------------------------------------------------------------------

/// Build a router over its own project + data root, returning the project id
/// the runtime minted for it. Two calls yield two independent projects.
async fn test_project() -> (axum::Router, String, TempDir) {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let project_root = fixture.path().join("project");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&project_root).unwrap();
    let state = AppState::try_new_at(&skills_dir, &project_root, &data_root, None)
        .await
        .unwrap();
    let manifest = kbd_runtime::Runtime::open_canonical_at(&project_root, &data_root)
        .unwrap()
        .project_manifest(false)
        .unwrap()
        .expect("try_new_at establishes the project manifest");
    (build_router(state), manifest.project_id, fixture)
}

fn kbd_status_request(project_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/api/v1/kbd/projects/{project_id}/status"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn kbd_status_reports_uninitialized_runtime_distinctly() {
    // A fresh project authenticates but has no committed events. This must stay
    // distinguishable from "unknown project" — collapsing the two is what made
    // an empty runtime look like an unreachable daemon.
    let (app, project_id, _fixture) = test_project().await;
    let resp = app.oneshot(kbd_status_request(&project_id)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "kbd runtime is not initialized");
}

#[tokio::test]
async fn kbd_status_rejects_an_unrelated_project_id() {
    let (app, _project_id, _fixture) = test_project().await;
    let unrelated = "00000000-0000-4000-8000-000000000000";
    let resp = app.oneshot(kbd_status_request(unrelated)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn kbd_command_rejects_an_envelope_whose_project_disagrees_with_the_path() {
    // Guards rest_api.rs's envelope/path equality check. Without it a caller
    // could commit a signed event into a project other than the one addressed.
    let (app, project_id, _fixture) = test_project().await;
    let envelope = serde_json::json!({
        "command": {
            "schemaVersion": "2",
            "projectId": "00000000-0000-4000-8000-000000000000",
            "runId": "run-mismatch",
            "commandId": "11111111-2222-4333-8444-555555555555",
            "frontier": {},
            "actor": {
                "kind": "harness",
                "id": "operator",
                "device": "test-device",
                "harness": "claude-code",
                "session": "test-session"
            },
            "command": { "type": "cancel", "payload": { "reason": "path mismatch" } }
        },
        "signerKeyId": "ed25519:not-checked-before-path-validation",
        "signature": "invalid"
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/kbd/projects/{project_id}/commands"))
        .header("content-type", "application/json")
        .body(Body::from(envelope.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn kbd_command_requires_device_signature_and_claim_surface_reports_commit() {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let fixture = tempfile::tempdir().unwrap();
    let project_root = fixture.path().join("project");
    let data_root = fixture.path().join("data");
    std::fs::create_dir_all(&project_root).unwrap();
    let runtime = kbd_runtime::Runtime::open_canonical_at(&project_root, &data_root).unwrap();
    let project_id = runtime.project_manifest(false).unwrap().unwrap().project_id;
    let initialized = runtime
        .initialize(
            project_id.clone(),
            "run-a",
            kbd_runtime::Actor::operator("operator", "test"),
        )
        .unwrap();
    let state = AppState::try_new_at(&skills_dir, &project_root, &data_root, None)
        .await
        .unwrap();
    let app = build_router(state);
    let command = kbd_runtime::CommandEnvelope {
        schema_version: "2".into(),
        project_id: project_id.clone(),
        run_id: initialized.run_id.clone(),
        command_id: "claim-command-a".into(),
        frontier: Some(initialized.frontier.clone()),
        expected_revision: 0,
        actor: kbd_runtime::Actor {
            kind: kbd_runtime::ActorKind::Harness,
            id: "holder-a".into(),
            device: "device-a".into(),
            harness: "test".into(),
            session: "session-a".into(),
        },
        command: kbd_runtime::CommandKind::ClaimAcquire {
            scope: "phase:recovery".into(),
            mode: kbd_runtime::ClaimMode::Exclusive,
            ttl_seconds: 300,
            holder_id: "holder-a".into(),
        },
    };

    let unsigned = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/kbd/projects/{project_id}/commands"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&command).unwrap()))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(unsigned).await.unwrap().status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let signed =
        kbd_runtime::SignedCommandEnvelope::sign(command, &runtime.device_signer().unwrap())
            .unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/kbd/projects/{project_id}/claims/acquire"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&signed).unwrap()))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(request).await.unwrap().status(),
        StatusCode::OK
    );

    let audit = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/kbd/projects/{project_id}/audit"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(audit).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/x-ndjson");
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let events = std::str::from_utf8(&body)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<kbd_runtime::Event>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert_eq!(kbd_runtime::replay_events(&events).unwrap().claims.len(), 1);

    let claims = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/kbd/projects/{project_id}/claims"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(claims).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["claims"].as_object().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// 6. CRDT: export snapshot then apply delta roundtrip
// ---------------------------------------------------------------------------

#[test]
fn crdt_export_snapshot_and_apply_roundtrip() {
    let manifest = default_manifest();
    let domain = SyncDomain::new("learner-model");

    let mut docs: HashMap<SyncDomain, loro::LoroDoc> = HashMap::new();

    // Insert a value into the source doc (Loro 1.13 API — no txn needed for maps)
    let src_doc = loro::LoroDoc::new();
    {
        let map = src_doc.get_map("skills");
        map.insert("test-skill", "A test skill").unwrap();
    }
    docs.insert(domain.clone(), src_doc);

    // Export snapshot
    let bytes = export_outgoing_delta(&manifest, &domain, None, &docs).unwrap();
    assert!(!bytes.is_empty(), "snapshot should not be empty");

    // Apply to fresh destination docs
    let mut dst_docs: HashMap<SyncDomain, loro::LoroDoc> = HashMap::new();
    apply_incoming_delta(&manifest, &domain, &bytes, &mut dst_docs).unwrap();

    // Version vector should be present after import
    let vv = current_version(&domain, &dst_docs);
    assert!(
        vv.is_some(),
        "version vector should be present after import"
    );
}

// ---------------------------------------------------------------------------
// 7. CRDT: SurrealMemory (LocalOnly) is rejected by apply and export
// ---------------------------------------------------------------------------

#[test]
fn crdt_rejects_local_only_domain() {
    let manifest = default_manifest();
    let local_domain = SyncDomain::new("surreal-memory");
    let dummy_bytes = vec![0u8; 16];
    let mut docs = HashMap::new();

    let apply_result = apply_incoming_delta(&manifest, &local_domain, &dummy_bytes, &mut docs);
    assert!(
        apply_result.is_err(),
        "LocalOnly domain must be rejected on apply"
    );

    let export_result = export_outgoing_delta(&manifest, &local_domain, None, &docs);
    assert!(
        export_result.is_err(),
        "LocalOnly domain must be rejected on export"
    );
}

// ---------------------------------------------------------------------------
// 8. P2P: topic derivation is deterministic and operator-specific
// ---------------------------------------------------------------------------

#[test]
fn p2p_topic_derivation_deterministic_and_unique() {
    let operator_a = [1u8; 32];
    let operator_b = [2u8; 32];

    let topic_a1 = P2PNode::derive_topic(&operator_a);
    let topic_a2 = P2PNode::derive_topic(&operator_a);
    let topic_b = P2PNode::derive_topic(&operator_b);

    assert_eq!(topic_a1, topic_a2, "same operator_id must yield same topic");
    assert_ne!(
        topic_a1, topic_b,
        "different operator_ids must yield different topics"
    );
}
