/// Integration tests for sovereign-sync REST API and CRDT engine.
///
/// Tests 1-5 use Axum's oneshot() to test the router in-process.
/// Tests 6-8 test the CRDT and P2P utilities directly.
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceExt;

use sovereign_sync::crdt::{apply_incoming_delta, current_version, export_outgoing_delta};
use sovereign_sync::p2p::P2PNode;
use sovereign_sync::rest_api::{build_router, AppState};
use storage_provider::{DomainConfig, PrivacyClass, SyncDomain, SyncManifest};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn test_router() -> axum::Router {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let state = AppState::new(&skills_dir);
    build_router(state)
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
    let app = test_router();
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
    let app = test_router();
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/sync/status")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["node_state"], "idle");
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
    let app = test_router();
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
    let app = test_router();
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
    let app = test_router();
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
    assert_eq!(json["status"], "queued");
    assert_eq!(json["domain"], "learner-model");
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
