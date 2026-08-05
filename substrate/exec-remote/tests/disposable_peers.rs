#![cfg(feature = "transport")]

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
};

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, key_id, sign_receipt_ed25519, sign_request_ed25519, CapabilityManifest,
    CodeIdentity, CodeKind, EvidenceClass, ExecutingDevice, ExecutionBackend, ExecutionExit,
    ExecutionLimits, ExecutionOutputs, ExecutionProvenance, ExecutionReceipt, ExecutionTier,
    RequestedTier, ResourceUsage, RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest,
    SCHEMA_VERSION,
};
use prometheus_exec_remote::{
    sign_dispatch_ed25519, sign_peer_response_ed25519, DispatchQueue, EnrollmentBinding,
    EnrollmentSnapshot, LocalExecutionHandoff, LocalExecutionOutcome, PeerDispatchState,
    RemoteError, RemoteOrigin, RemoteTarget, RemoteTransport, Result, SignedPeerDispatchResponse,
    SignedRemoteDispatch, REMOTE_SCHEMA_VERSION,
};
use tempfile::tempdir;
use uuid::Uuid;

const ORIGIN: &str = "disposable-origin";
const TARGET: &str = "disposable-target";

fn binding(endpoint: &str, key: &SigningKey) -> EnrollmentBinding {
    let public = key.verifying_key().to_bytes();
    EnrollmentBinding {
        endpoint_id: endpoint.into(),
        sig_alg: SignatureAlgorithm::Ed25519,
        key_id: key_id(SignatureAlgorithm::Ed25519, &public),
        public_key: URL_SAFE_NO_PAD.encode(public),
    }
}

fn fixture() -> (
    SignedRemoteDispatch,
    EnrollmentSnapshot,
    SigningKey,
    SigningKey,
) {
    let origin_key = SigningKey::from_bytes(&[41; 32]);
    let target_key = SigningKey::from_bytes(&[42; 32]);
    let now = Utc::now();
    let mut request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::new_v4(),
        issued_at: now,
        queued_at: Some(now),
        validity_window_secs: 600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print('remote evidence')"),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: Vec::new(),
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: vec![TARGET.into()],
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    sign_request_ed25519(&mut request, &origin_key).unwrap();
    let enrollment = EnrollmentSnapshot {
        schema_version: REMOTE_SCHEMA_VERSION.into(),
        captured_at: now,
        bindings: BTreeMap::from([
            (ORIGIN.into(), binding(ORIGIN, &origin_key)),
            (TARGET.into(), binding(TARGET, &target_key)),
        ]),
    };
    let mut dispatch = SignedRemoteDispatch {
        schema_version: REMOTE_SCHEMA_VERSION.into(),
        dispatch_id: Uuid::new_v4(),
        request_hash: request.request_hash().unwrap(),
        request,
        origin_endpoint_id: ORIGIN.into(),
        target_endpoint_id: TARGET.into(),
        enrollment_snapshot_hash: enrollment.snapshot_hash().unwrap(),
        issued_at: now,
        validity_window_secs: 300,
        signer_key_id: String::new(),
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    sign_dispatch_ed25519(&mut dispatch, &origin_key).unwrap();
    (dispatch, enrollment, origin_key, target_key)
}

struct CountingExecutor {
    key: SigningKey,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LocalExecutionHandoff for CountingExecutor {
    async fn execute(&self, dispatch: &SignedRemoteDispatch) -> Result<LocalExecutionOutcome> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        let run_id = Uuid::new_v4();
        let mut receipt = ExecutionReceipt {
            schema_version: SCHEMA_VERSION.into(),
            run_id,
            request_hash: dispatch.request_hash.clone(),
            state: RunState::Succeeded,
            evidence_class: EvidenceClass::Attested,
            tier: ExecutionTier::P,
            code_hash: dispatch.request.code.hash.clone(),
            input_set_hash: hash_bytes(b"remote-inputs"),
            env_hash: hash_bytes(b"disposable-environment"),
            toolchain_hash: Some(hash_bytes(b"python3")),
            sandbox_profile_hash: hash_bytes(b"seatbelt"),
            backend: ExecutionBackend::Seatbelt,
            exit: ExecutionExit {
                status: 0,
                signal_or_trap: None,
            },
            outputs: ExecutionOutputs {
                stdout: hash_bytes(b"remote evidence\n"),
                stderr: hash_bytes(b""),
                artifacts: Vec::new(),
            },
            usage: ResourceUsage::default(),
            started_at: now,
            finished_at: now,
            executing_device: ExecutingDevice {
                key_id: String::new(),
                sig_alg: SignatureAlgorithm::Ed25519,
                platform: "disposable-peer".into(),
            },
            grants: Vec::new(),
            component: None,
            failure: None,
            signature: None,
        };
        sign_receipt_ed25519(&mut receipt, &self.key)?;
        Ok(LocalExecutionOutcome {
            state: PeerDispatchState::Applied,
            run_id: Some(run_id),
            receipt: Some(receipt),
            failure: None,
        })
    }
}

struct TargetTransport {
    target: Arc<RemoteTarget<CountingExecutor>>,
    lose_first_response: AtomicBool,
}

struct InvalidReceiptTransport {
    target: Arc<RemoteTarget<CountingExecutor>>,
    target_key: SigningKey,
}

#[async_trait]
impl RemoteTransport for InvalidReceiptTransport {
    async fn deliver(&self, dispatch: SignedRemoteDispatch) -> Result<SignedPeerDispatchResponse> {
        let mut response = self.target.receive(dispatch, Utc::now()).await?;
        response
            .receipt
            .as_mut()
            .expect("target fixture returns a receipt")
            .signature = Some(URL_SAFE_NO_PAD.encode([0_u8; 64]));
        sign_peer_response_ed25519(&mut response, &self.target_key)?;
        Ok(response)
    }
}

#[async_trait]
impl RemoteTransport for TargetTransport {
    async fn deliver(&self, dispatch: SignedRemoteDispatch) -> Result<SignedPeerDispatchResponse> {
        let response = self.target.receive(dispatch, Utc::now()).await?;
        if self.lose_first_response.swap(false, Ordering::SeqCst) {
            return Err(RemoteError::Transport(
                "simulated response loss after target commit".into(),
            ));
        }
        Ok(response)
    }
}

#[test]
fn response_loss_resumes_offline_without_reexecuting_and_survives_restart() {
    let (dispatch, enrollment, _, target_key) = fixture();
    let origin_dir = tempdir().unwrap();
    let target_dir = tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let target = Arc::new(
        RemoteTarget::new(
            TARGET,
            DispatchQueue::open(target_dir.path()).unwrap(),
            enrollment.clone(),
            target_key.clone(),
            Arc::new(CountingExecutor {
                key: target_key,
                calls: calls.clone(),
            }),
        )
        .unwrap(),
    );
    let origin = RemoteOrigin::new(
        DispatchQueue::open(origin_dir.path()).unwrap(),
        enrollment.clone(),
        Arc::new(TargetTransport {
            target,
            lose_first_response: AtomicBool::new(true),
        }),
    );

    let unavailable = futures::executor::block_on(origin.dispatch(dispatch.clone(), Utc::now()))
        .expect("response loss is durably classified");
    assert_eq!(unavailable.state, PeerDispatchState::Unavailable);
    let applied = futures::executor::block_on(origin.dispatch(dispatch.clone(), Utc::now()))
        .expect("same dispatch resumes");
    assert_eq!(applied.state, PeerDispatchState::Applied);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    drop(origin);
    let reopened = DispatchQueue::open(origin_dir.path()).unwrap();
    let record = reopened.get(dispatch.dispatch_id).unwrap().unwrap();
    assert_eq!(record.state, PeerDispatchState::Applied);
    assert_eq!(record.receipt.unwrap().request_hash, dispatch.request_hash);
}

#[test]
fn origin_rejects_a_valid_peer_envelope_with_an_invalid_nested_receipt() {
    let (dispatch, enrollment, _, target_key) = fixture();
    let origin_dir = tempdir().unwrap();
    let target_dir = tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let target = Arc::new(
        RemoteTarget::new(
            TARGET,
            DispatchQueue::open(target_dir.path()).unwrap(),
            enrollment.clone(),
            target_key.clone(),
            Arc::new(CountingExecutor {
                key: target_key.clone(),
                calls: calls.clone(),
            }),
        )
        .unwrap(),
    );
    let queue = DispatchQueue::open(origin_dir.path()).unwrap();
    let origin = RemoteOrigin::new(
        queue,
        enrollment,
        Arc::new(InvalidReceiptTransport { target, target_key }),
    );

    let error = futures::executor::block_on(origin.dispatch(dispatch.clone(), Utc::now()))
        .expect_err("origin must verify the nested receipt independently");

    assert!(matches!(error, RemoteError::InvalidPeerResponse(_)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let record = DispatchQueue::open(origin_dir.path())
        .unwrap()
        .get(dispatch.dispatch_id)
        .unwrap()
        .unwrap();
    assert_eq!(record.state, PeerDispatchState::Queued);
    assert!(record.receipt.is_none());
}

#[test]
fn unknown_endpoint_signer_mismatch_replay_and_expiry_are_rejected() {
    let (mut dispatch, enrollment, origin_key, target_key) = fixture();
    let target_dir = tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let wrong_key = SigningKey::from_bytes(&[43; 32]);
    assert!(matches!(
        RemoteTarget::new(
            TARGET,
            DispatchQueue::open(target_dir.path()).unwrap(),
            enrollment.clone(),
            wrong_key,
            Arc::new(CountingExecutor {
                key: target_key.clone(),
                calls: calls.clone(),
            }),
        ),
        Err(RemoteError::SignerMismatch(_))
    ));

    let target = RemoteTarget::new(
        TARGET,
        DispatchQueue::open(target_dir.path()).unwrap(),
        enrollment.clone(),
        target_key,
        Arc::new(CountingExecutor {
            key: SigningKey::from_bytes(&[42; 32]),
            calls,
        }),
    )
    .unwrap();
    let mut wrong_target = dispatch.clone();
    wrong_target.target_endpoint_id = "not-enrolled".into();
    assert!(matches!(
        futures::executor::block_on(target.receive(wrong_target, Utc::now())),
        Err(RemoteError::UnknownEndpoint(_))
    ));

    let queue_dir = tempdir().unwrap();
    let queue = DispatchQueue::open(queue_dir.path()).unwrap();
    let first = queue
        .accept(dispatch.clone(), &enrollment, Utc::now())
        .unwrap();
    assert!(!first.replayed);
    assert!(
        queue
            .accept(dispatch.clone(), &enrollment, Utc::now())
            .unwrap()
            .replayed
    );

    dispatch.issued_at = Utc::now() - Duration::seconds(10);
    dispatch.validity_window_secs = 1;
    dispatch.dispatch_id = Uuid::new_v4();
    sign_dispatch_ed25519(&mut dispatch, &origin_key).unwrap();
    assert!(matches!(
        queue.accept(dispatch, &enrollment, Utc::now()),
        Err(RemoteError::Expired)
    ));
}

#[test]
fn target_rechecks_expiry_immediately_before_execution() {
    let (mut dispatch, enrollment, origin_key, target_key) = fixture();
    dispatch.issued_at = Utc::now() - Duration::seconds(10);
    dispatch.validity_window_secs = 1;
    sign_dispatch_ed25519(&mut dispatch, &origin_key).unwrap();
    let acceptance_time = dispatch.issued_at;
    let target_dir = tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let target = RemoteTarget::new(
        TARGET,
        DispatchQueue::open(target_dir.path()).unwrap(),
        enrollment,
        target_key.clone(),
        Arc::new(CountingExecutor {
            key: target_key,
            calls: calls.clone(),
        }),
    )
    .unwrap();

    let response = futures::executor::block_on(target.receive(dispatch, acceptance_time))
        .expect("receipt is recorded before the execution-time expiry check");

    assert_eq!(response.state, PeerDispatchState::Expired);
    assert!(response
        .failure
        .as_deref()
        .is_some_and(|failure| failure.contains("before local execution")));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

struct BlockingTransport {
    started: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

#[async_trait]
impl RemoteTransport for BlockingTransport {
    async fn deliver(&self, _dispatch: SignedRemoteDispatch) -> Result<SignedPeerDispatchResponse> {
        self.started.send(()).unwrap();
        self.release.lock().unwrap().recv().unwrap();
        Err(RemoteError::Transport("released slow transport".into()))
    }
}

struct OfflineTransport;

#[async_trait]
impl RemoteTransport for OfflineTransport {
    async fn deliver(&self, _dispatch: SignedRemoteDispatch) -> Result<SignedPeerDispatchResponse> {
        Err(RemoteError::Transport("peer offline".into()))
    }
}

#[test]
fn slow_transport_isolated_from_an_independent_dispatch() {
    let (slow_dispatch, enrollment, _, _) = fixture();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let slow_dir = tempdir().unwrap();
    let slow_origin = RemoteOrigin::new(
        DispatchQueue::open(slow_dir.path()).unwrap(),
        enrollment.clone(),
        Arc::new(BlockingTransport {
            started: started_tx,
            release: Mutex::new(release_rx),
        }),
    );
    let slow = std::thread::spawn(move || {
        futures::executor::block_on(slow_origin.dispatch(slow_dispatch, Utc::now())).unwrap()
    });
    started_rx.recv().unwrap();

    let (fast_dispatch, fast_enrollment, _, _) = fixture();
    let fast_dir = tempdir().unwrap();
    let fast_origin = RemoteOrigin::new(
        DispatchQueue::open(fast_dir.path()).unwrap(),
        fast_enrollment,
        Arc::new(OfflineTransport),
    );
    let fast = futures::executor::block_on(fast_origin.dispatch(fast_dispatch, Utc::now()))
        .expect("independent dispatch completes while slow peer is blocked");
    assert_eq!(fast.state, PeerDispatchState::Unavailable);

    release_tx.send(()).unwrap();
    assert_eq!(slow.join().unwrap().state, PeerDispatchState::Unavailable);
}
