use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use chrono::{TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, sign_receipt_ed25519, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass,
    ExecutingDevice, ExecutionBackend, ExecutionExit, ExecutionLimits, ExecutionOutputs,
    ExecutionProvenance, ExecutionReceipt, ExecutionTier, RequestedTier, ResourceUsage, RunState,
    RuntimeKind, SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
use prometheus_exec_service::{
    build_api_router, peer_is_same_user, ApiErrorEnvelope, ExecutionService, ReadinessSnapshot,
    RunEventData, RunRecord, SidecarState, UdsSidecar, UdsSidecarError,
};
use tempfile::{tempdir, TempDir};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tower::ServiceExt as _;
use uuid::Uuid;

fn request(request_id: Uuid, code: &[u8]) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 17, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(code),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![],
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: vec![],
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    }
}

fn signing_material() -> (SigningKey, VerificationKey) {
    let signing = SigningKey::from_bytes(&[53_u8; 32]);
    let verification = VerificationKey::ed25519(signing.verifying_key().to_bytes());
    (signing, verification)
}

fn signed_receipt(record: &RunRecord, key: &SigningKey) -> ExecutionReceipt {
    let started_at = Utc.with_ymd_and_hms(2026, 8, 4, 17, 0, 1).unwrap();
    let finished_at = Utc.with_ymd_and_hms(2026, 8, 4, 17, 0, 2).unwrap();
    let mut receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: record.run_id,
        request_hash: record.request_hash.clone(),
        state: RunState::Succeeded,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        code_hash: record.request.code.hash.clone(),
        input_set_hash: hash_bytes(b"inputs"),
        env_hash: hash_bytes(b"environment"),
        toolchain_hash: Some(hash_bytes(b"python3")),
        sandbox_profile_hash: hash_bytes(b"seatbelt"),
        backend: ExecutionBackend::Seatbelt,
        exit: ExecutionExit {
            status: 0,
            signal_or_trap: None,
        },
        outputs: ExecutionOutputs {
            stdout: hash_bytes(b"stdout"),
            stderr: hash_bytes(b"stderr"),
            artifacts: vec![],
        },
        usage: ResourceUsage::default(),
        started_at,
        finished_at,
        executing_device: ExecutingDevice {
            key_id: String::new(),
            sig_alg: SignatureAlgorithm::Ed25519,
            platform: "macos-aarch64".into(),
        },
        grants: vec![],
        signature: None,
    };
    sign_receipt_ed25519(&mut receipt, key).unwrap();
    receipt
}

async fn ready_state() -> (
    SidecarState,
    Arc<ExecutionService>,
    Arc<ArtifactStore>,
    TempDir,
) {
    let directory = tempdir().unwrap();
    let service = Arc::new(ExecutionService::open(directory.path().join("service")).unwrap());
    let artifacts = Arc::new(
        ArtifactStore::open(directory.path().join("artifacts"), 16 * 1024 * 1024).unwrap(),
    );
    let state = SidecarState::new();
    state.install(service.clone(), artifacts.clone()).await;
    (state, service, artifacts, directory)
}

fn json_request(method: &str, uri: &str, value: &impl serde::Serialize) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(value).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn health_is_static_while_ready_and_mutating_routes_report_initialization() {
    let state = SidecarState::new();
    let router = build_api_router(state);
    let health = router
        .clone()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let ready = router
        .clone()
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    let snapshot: ReadinessSnapshot =
        serde_json::from_slice(&to_bytes(ready.into_body(), 16 * 1024).await.unwrap()).unwrap();
    assert!(!snapshot.ready);
    assert_eq!(snapshot.subsystems.len(), 2);

    let response = router
        .oneshot(json_request(
            "POST",
            "/api/v2/exec/runs",
            &request(Uuid::new_v4(), b"print('wait')"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let envelope: ApiErrorEnvelope =
        serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap()).unwrap();
    assert_eq!(envelope.error.code, "service_initializing");
}

#[tokio::test]
async fn run_routes_are_replay_safe_and_return_consistent_errors() {
    let (state, _, _, _directory) = ready_state().await;
    let router = build_api_router(state);
    let request_id = Uuid::new_v4();
    let original = request(request_id, b"print('original')");

    let accepted = router
        .clone()
        .oneshot(json_request("POST", "/api/v2/exec/runs", &original))
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::ACCEPTED);
    let accepted: serde_json::Value =
        serde_json::from_slice(&to_bytes(accepted.into_body(), 64 * 1024).await.unwrap()).unwrap();
    let run_id = accepted["runId"].as_str().unwrap();
    assert_eq!(accepted["replayed"], false);

    let replay = router
        .clone()
        .oneshot(json_request("POST", "/api/v2/exec/runs", &original))
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::OK);
    let replay: serde_json::Value =
        serde_json::from_slice(&to_bytes(replay.into_body(), 64 * 1024).await.unwrap()).unwrap();
    assert_eq!(replay["runId"], run_id);
    assert_eq!(replay["replayed"], true);

    let conflict = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v2/exec/runs",
            &request(request_id, b"print('different')"),
        ))
        .await
        .unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conflict: ApiErrorEnvelope =
        serde_json::from_slice(&to_bytes(conflict.into_body(), 16 * 1024).await.unwrap()).unwrap();
    assert_eq!(conflict.error.code, "request_hash_conflict");

    let found = router
        .clone()
        .oneshot(
            Request::get(format!("/api/v2/exec/runs/{run_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(found.status(), StatusCode::OK);
    let invalid = router
        .clone()
        .oneshot(
            Request::get("/api/v2/exec/runs/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let missing = router
        .oneshot(
            Request::get(format!("/api/v2/exec/runs/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sse_resume_receipt_and_artifact_routes_return_durable_evidence() {
    let (state, service, artifacts, _directory) = ready_state().await;
    let router = build_api_router(state);
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('evidence')");
    let hash = request.request_hash().unwrap();
    let submitted = service.submit(request.clone()).unwrap();
    service.mark_spawned(request.request_id, &hash).unwrap();
    service
        .append_runtime_event(
            submitted.record.run_id,
            "stdout.1",
            Utc.with_ymd_and_hms(2026, 8, 4, 17, 0, 1).unwrap(),
            RunEventData::Stdout {
                chunk: "evidence\n".into(),
            },
        )
        .unwrap();
    let receipt = signed_receipt(&submitted.record, &signing);
    service
        .commit_terminal(request.request_id, &hash, receipt.clone(), &verification)
        .unwrap();

    let events = router
        .clone()
        .oneshot(
            Request::get(format!(
                "/api/v2/exec/runs/{}/events?after=2",
                submitted.record.run_id
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(events.status(), StatusCode::OK);
    assert_eq!(events.headers()[header::CONTENT_TYPE], "text/event-stream");
    let body = String::from_utf8(
        to_bytes(events.into_body(), 128 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("id: 3"));
    assert!(body.contains("event: stdout"));
    assert!(body.contains("id: 4"));
    assert!(body.contains("event: completed"));
    assert!(!body.contains("id: 1"));
    assert!(!body.contains("id: 2"));

    let receipt_response = router
        .clone()
        .oneshot(
            Request::get(format!("/api/v2/exec/receipts/{}", submitted.record.run_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(receipt_response.status(), StatusCode::OK);
    let returned: ExecutionReceipt = serde_json::from_slice(
        &to_bytes(receipt_response.into_body(), 128 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(returned, receipt);

    let stored = artifacts.put(b"portable evidence").unwrap();
    let artifact_response = router
        .oneshot(
            Request::get(format!("/api/v2/exec/artifacts/{}", stored.hash))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(artifact_response.status(), StatusCode::OK);
    assert_eq!(
        &to_bytes(artifact_response.into_body(), 1024).await.unwrap()[..],
        b"portable evidence"
    );
}

#[tokio::test]
async fn sse_connection_streams_events_appended_after_the_client_connects() {
    let (state, service, _, _directory) = ready_state().await;
    let router = build_api_router(state);
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('live')");
    let hash = request.request_hash().unwrap();
    let submitted = service.submit(request.clone()).unwrap();
    service.mark_spawned(request.request_id, &hash).unwrap();
    let response = router
        .oneshot(
            Request::get(format!(
                "/api/v2/exec/runs/{}/events?after=2",
                submitted.record.run_id
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_reader = tokio::spawn(async move { to_bytes(response.into_body(), 128 * 1024).await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    service
        .append_runtime_event(
            submitted.record.run_id,
            "stdout.live",
            Utc::now(),
            RunEventData::Stdout {
                chunk: "arrived after connect\n".into(),
            },
        )
        .unwrap();
    service
        .commit_terminal(
            request.request_id,
            &hash,
            signed_receipt(&submitted.record, &signing),
            &verification,
        )
        .unwrap();
    let body = String::from_utf8(
        tokio::time::timeout(Duration::from_secs(2), body_reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("event: stdout"));
    assert!(body.contains("arrived after connect"));
    assert!(body.contains("event: completed"));
}

#[cfg(unix)]
async fn uds_request(path: &std::path::Path, target: &str) -> Vec<u8> {
    let mut stream = tokio::net::UnixStream::connect(path).await.unwrap();
    stream
        .write_all(
            format!("GET {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), stream.read_to_end(&mut response))
        .await
        .unwrap()
        .unwrap();
    response
}

#[cfg(unix)]
#[tokio::test]
async fn uds_binds_health_first_with_private_mode_and_real_peer_credentials() {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = tempdir().unwrap();
    let socket = directory.path().join("runtime/prometheus-exec.sock");
    let bind_started = Instant::now();
    let sidecar = UdsSidecar::start(&socket).await.unwrap();
    let bind_elapsed = bind_started.elapsed();
    assert!(
        bind_elapsed < Duration::from_secs(1),
        "health-first UDS bind exceeded one second"
    );
    eprintln!("health_first_bind_us={}", bind_elapsed.as_micros());
    let mode = std::fs::symlink_metadata(&socket)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(
        std::fs::metadata(socket.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert!(peer_is_same_user(
        nix::unistd::geteuid().as_raw(),
        Some(nix::unistd::geteuid().as_raw())
    ));
    assert!(!peer_is_same_user(
        nix::unistd::geteuid().as_raw(),
        Some(nix::unistd::geteuid().as_raw().wrapping_add(1))
    ));

    let started = Instant::now();
    let health = uds_request(&socket, "/health").await;
    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(health.starts_with(b"HTTP/1.1 200"));
    let mut warm_health_latencies = Vec::with_capacity(100);
    for _ in 0..100 {
        let started = Instant::now();
        let health = uds_request(&socket, "/health").await;
        warm_health_latencies.push(started.elapsed());
        assert!(health.starts_with(b"HTTP/1.1 200"));
    }
    warm_health_latencies.sort_unstable();
    let p95 = warm_health_latencies[94];
    eprintln!("warm_health_p95_us={}", p95.as_micros());
    assert!(
        p95 < Duration::from_millis(10),
        "warm /health p95 was {p95:?}, expected under 10 ms"
    );
    let ready = uds_request(&socket, "/ready").await;
    assert!(ready.starts_with(b"HTTP/1.1 503"));

    let service = Arc::new(ExecutionService::open(directory.path().join("service")).unwrap());
    let artifacts =
        Arc::new(ArtifactStore::open(directory.path().join("artifacts"), 1024 * 1024).unwrap());
    sidecar.state().install(service, artifacts).await;
    let ready = uds_request(&socket, "/ready").await;
    assert!(ready.starts_with(b"HTTP/1.1 200"));

    sidecar.shutdown().await.unwrap();
    assert!(!socket.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn uds_refuses_non_socket_and_active_socket_paths() {
    let directory = tempdir().unwrap();
    let non_socket = directory.path().join("not-a-socket");
    std::fs::write(&non_socket, b"preserve me").unwrap();
    let result = UdsSidecar::start(&non_socket).await;
    assert!(matches!(result, Err(UdsSidecarError::UnsafeSocket(_))));
    assert_eq!(std::fs::read(&non_socket).unwrap(), b"preserve me");

    let socket = directory.path().join("active.sock");
    let first = UdsSidecar::start(&socket).await.unwrap();
    let second = UdsSidecar::start(&socket).await;
    assert!(matches!(second, Err(UdsSidecarError::SocketInUse(_))));
    assert!(uds_request(&socket, "/health")
        .await
        .starts_with(b"HTTP/1.1 200"));
    first.shutdown().await.unwrap();
}
