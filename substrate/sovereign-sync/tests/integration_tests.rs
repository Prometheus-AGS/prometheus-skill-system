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
    let resp = app
        .oneshot(kbd_status_request(&project_id))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "kbd runtime is not initialized");
}

#[tokio::test]
async fn kbd_status_rejects_an_unrelated_project_id() {
    let (app, _project_id, _fixture) = test_project().await;
    let unrelated = "00000000-0000-4000-8000-000000000000";
    let resp = app
        .oneshot(kbd_status_request(unrelated))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}



#[tokio::test]
async fn kbd_command_rejects_an_envelope_whose_project_disagrees_with_the_path() {
    // Guards rest_api.rs's envelope/path equality check. Without it a caller
    // could commit a signed event into a project other than the one addressed.
    let (app, project_id, _fixture) = test_project().await;
    let envelope = serde_json::json!({
        "schemaVersion": "1",
        "projectId": "00000000-0000-4000-8000-000000000000",
        "runId": "run-mismatch",
        "commandId": "11111111-2222-4333-8444-555555555555",
        "expectedRevision": 0,
        "actor": {
            "kind": "harness",
            "id": "operator",
            "device": "test-device",
            "harness": "claude-code",
            "session": "test-session"
        },
        "command": { "type": "lease_heartbeat" }
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
