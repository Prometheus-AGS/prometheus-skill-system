use std::{sync::Arc, thread};

use chrono::{TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, sign_receipt_ed25519, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass,
    ExecutingDevice, ExecutionBackend, ExecutionExit, ExecutionLimits, ExecutionOutputs,
    ExecutionProvenance, ExecutionReceipt, ExecutionTier, RequestedTier, ResourceUsage, RunState,
    RuntimeKind, SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_service::{
    AcceptRunResult, GrantPendingRecord, RunLedger, RunLedgerError, RunRecord, SpawnStatus,
};
use tempfile::tempdir;
use uuid::Uuid;

fn request(request_id: Uuid, code: &[u8]) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 15, 0, 0).unwrap(),
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
    let signing = SigningKey::from_bytes(&[29_u8; 32]);
    let verification = VerificationKey::ed25519(signing.verifying_key().to_bytes());
    (signing, verification)
}

fn signed_receipt(record: &RunRecord, state: RunState, key: &SigningKey) -> ExecutionReceipt {
    let status = if state == RunState::Succeeded { 0 } else { 125 };
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 15, 0, 1).unwrap();
    let mut receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: record.run_id,
        request_hash: record.request_hash.clone(),
        state,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        code_hash: record.request.code.hash.clone(),
        input_set_hash: hash_bytes(b"inputs"),
        env_hash: hash_bytes(b"environment"),
        toolchain_hash: Some(hash_bytes(b"python3")),
        sandbox_profile_hash: hash_bytes(b"seatbelt"),
        backend: ExecutionBackend::Seatbelt,
        exit: ExecutionExit {
            status,
            signal_or_trap: (state == RunState::Interrupted).then(|| "service-restart".into()),
        },
        outputs: ExecutionOutputs {
            stdout: hash_bytes(b"stdout"),
            stderr: hash_bytes(b"stderr"),
            artifacts: vec![],
        },
        usage: ResourceUsage::default(),
        started_at: now,
        finished_at: now,
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

#[test]
fn same_hash_replays_with_the_original_run_across_restart() {
    let directory = tempdir().unwrap();
    let request = request(Uuid::new_v4(), b"print('durable')");
    let ledger = RunLedger::open(directory.path()).unwrap();
    let first = ledger.accept(request.clone()).unwrap();
    assert!(matches!(first, AcceptRunResult::Accepted(_)));
    let run_id = first.record().run_id;
    let replay = ledger.accept(request.clone()).unwrap();
    assert!(matches!(replay, AcceptRunResult::Replay(_)));
    assert_eq!(replay.record().run_id, run_id);

    drop(ledger);
    let reopened = RunLedger::open(directory.path()).unwrap();
    let replay = reopened.accept(request).unwrap();
    assert!(!replay.created());
    assert_eq!(replay.record().run_id, run_id);
}

#[test]
fn same_id_with_a_different_hash_conflicts_after_restart() {
    let directory = tempdir().unwrap();
    let request_id = Uuid::new_v4();
    let original = request(request_id, b"print('first')");
    let candidate = request(request_id, b"print('different')");
    let original_hash = original.request_hash().unwrap();
    RunLedger::open(directory.path())
        .unwrap()
        .accept(original)
        .unwrap();

    let reopened = RunLedger::open(directory.path()).unwrap();
    let error = reopened.accept(candidate).unwrap_err();
    assert!(matches!(
        error,
        RunLedgerError::RequestHashConflict { existing, .. } if existing == original_hash
    ));
    assert_eq!(
        reopened.get(request_id).unwrap().unwrap().request_hash,
        original_hash
    );
}

#[test]
fn concurrent_accept_has_exactly_one_creator() {
    let directory = tempdir().unwrap();
    let request = request(Uuid::new_v4(), b"print('once')");
    let root = directory.path().to_path_buf();
    let request = Arc::new(request);
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let root = root.clone();
            let request = request.clone();
            thread::spawn(move || {
                RunLedger::open(root)
                    .unwrap()
                    .accept((*request).clone())
                    .unwrap()
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.created()).count(), 1);
    let run_ids: std::collections::HashSet<_> = results
        .iter()
        .map(|result| result.record().run_id)
        .collect();
    assert_eq!(run_ids.len(), 1);
}

#[test]
fn concurrent_execution_claim_has_exactly_one_spawner() {
    let directory = tempdir().unwrap();
    let request = request(Uuid::new_v4(), b"print('claim-once')");
    let request_id = request.request_id;
    let request_hash = request.request_hash().unwrap();
    RunLedger::open(directory.path())
        .unwrap()
        .accept(request)
        .unwrap();
    let root = directory.path().to_path_buf();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let root = root.clone();
            let request_hash = request_hash.clone();
            thread::spawn(move || {
                RunLedger::open(root)
                    .unwrap()
                    .claim_for_execution(request_id, &request_hash)
                    .unwrap()
            })
        })
        .collect();
    let claims: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(claims.iter().filter(|claim| claim.claimed).count(), 1);
    assert_eq!(
        claims
            .iter()
            .map(|claim| claim.record.run_id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        1
    );
}

#[test]
fn spawn_boundary_is_durable_and_idempotent() {
    let directory = tempdir().unwrap();
    let request = request(Uuid::new_v4(), b"print('spawn')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let accepted = ledger.accept(request.clone()).unwrap().record().clone();
    let spawned = ledger.mark_spawned(request.request_id, &hash).unwrap();
    assert_eq!(spawned.state, RunState::Running);
    assert!(matches!(spawned.spawn, SpawnStatus::Spawned { .. }));
    assert_eq!(
        ledger.mark_spawned(request.request_id, &hash).unwrap(),
        spawned
    );

    drop(ledger);
    let reopened = RunLedger::open(directory.path()).unwrap();
    let persisted = reopened.get(request.request_id).unwrap().unwrap();
    assert_eq!(persisted.run_id, accepted.run_id);
    assert_eq!(persisted, spawned);
}

#[test]
fn grant_pending_is_durable_idempotent_and_preserved_by_reconciliation() {
    let directory = tempdir().unwrap();
    let (_, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('grant pending')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let accepted = ledger.accept(request.clone()).unwrap().record().clone();
    let pending = GrantPendingRecord {
        event_id: "grant.policy".into(),
        reason: "network egress requires a trusted-host grant".into(),
        occurred_at: Utc::now(),
    };
    let marked = ledger
        .mark_grant_pending(request.request_id, &hash, pending.clone())
        .unwrap();
    assert_eq!(marked.state, RunState::GrantPending);
    assert_eq!(marked.spawn, SpawnStatus::NotSpawned);
    assert_eq!(marked.grant_pending.as_ref(), Some(&pending));
    let repeated = GrantPendingRecord {
        occurred_at: Utc::now(),
        ..pending.clone()
    };
    assert_eq!(
        ledger
            .mark_grant_pending(request.request_id, &hash, repeated)
            .unwrap(),
        marked
    );
    assert!(matches!(
        ledger.claim_for_execution(request.request_id, &hash),
        Err(RunLedgerError::InvalidRecord(_))
    ));

    drop(ledger);
    let reopened = RunLedger::open(directory.path()).unwrap();
    let report = reopened
        .reconcile(&verification, |_| {
            panic!("grant-pending run must not spawn")
        })
        .unwrap();
    assert!(report.requeued.is_empty());
    assert!(report.interrupted.is_empty());
    let recovered = reopened.get(request.request_id).unwrap().unwrap();
    assert_eq!(recovered.run_id, accepted.run_id);
    assert_eq!(recovered, marked);
}

#[test]
fn successful_terminal_receipt_requires_the_durable_spawn_boundary() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('must spawn')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let record = ledger.accept(request.clone()).unwrap().record().clone();
    let error = ledger
        .commit_terminal(
            request.request_id,
            &hash,
            signed_receipt(&record, RunState::Succeeded, &signing),
            &verification,
        )
        .unwrap_err();
    assert!(matches!(error, RunLedgerError::InvalidReceipt(_)));
    assert_eq!(ledger.get(request.request_id).unwrap().unwrap(), record);
    assert!(ledger.receipt_log().segments().unwrap().is_empty());
}

#[test]
fn terminal_commit_is_exactly_replayable_after_restart() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('terminal')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let record = ledger.accept(request.clone()).unwrap().record().clone();
    ledger.mark_spawned(request.request_id, &hash).unwrap();
    let receipt = signed_receipt(&record, RunState::Succeeded, &signing);
    let committed = ledger
        .commit_terminal(request.request_id, &hash, receipt.clone(), &verification)
        .unwrap();
    assert!(committed.created);
    assert_eq!(committed.record.terminal.as_ref().unwrap().receipt, receipt);
    assert_eq!(ledger.receipt_log().segments().unwrap().len(), 1);
    let replay = ledger
        .commit_terminal(request.request_id, &hash, receipt.clone(), &verification)
        .unwrap();
    assert!(!replay.created);
    assert_eq!(replay.record, committed.record);

    drop(ledger);
    let reopened = RunLedger::open(directory.path()).unwrap();
    let accepted = reopened.accept(request).unwrap();
    assert!(!accepted.created());
    assert_eq!(accepted.record(), &committed.record);
}

#[test]
fn reconciliation_closes_the_receipt_log_to_record_crash_window() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('crash-window')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let record = ledger.accept(request.clone()).unwrap().record().clone();
    ledger.mark_spawned(request.request_id, &hash).unwrap();
    let receipt = signed_receipt(&record, RunState::Succeeded, &signing);
    ledger
        .receipt_log()
        .append(receipt.clone(), &verification)
        .unwrap();
    drop(ledger);

    let reopened = RunLedger::open(directory.path()).unwrap();
    let report = reopened
        .reconcile(&verification, |_| panic!("receipt already exists"))
        .unwrap();
    assert_eq!(report.recovered_from_log, vec![request.request_id]);
    let recovered = reopened.get(request.request_id).unwrap().unwrap();
    assert_eq!(recovered.state, RunState::Succeeded);
    assert_eq!(recovered.terminal.unwrap().receipt, receipt);
}

#[test]
fn reconciliation_interrupts_spawned_runs_and_requeues_unspawned_runs() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let spawned_request = request(Uuid::new_v4(), b"print('spawned')");
    let queued_request = request(Uuid::new_v4(), b"print('queued')");
    let ledger = RunLedger::open(directory.path()).unwrap();
    let spawned = ledger
        .accept(spawned_request.clone())
        .unwrap()
        .record()
        .clone();
    ledger
        .mark_spawned(spawned_request.request_id, &spawned.request_hash)
        .unwrap();
    ledger.accept(queued_request.clone()).unwrap();

    let report = ledger
        .reconcile(&verification, |record| {
            Ok(signed_receipt(record, RunState::Interrupted, &signing))
        })
        .unwrap();
    assert_eq!(report.interrupted, vec![spawned_request.request_id]);
    assert_eq!(report.requeued, vec![queued_request.request_id]);
    assert_eq!(
        ledger
            .get(spawned_request.request_id)
            .unwrap()
            .unwrap()
            .state,
        RunState::Interrupted
    );
    let queued = ledger.get(queued_request.request_id).unwrap().unwrap();
    assert_eq!(queued.state, RunState::Queued);
    assert_eq!(queued.spawn, SpawnStatus::NotSpawned);
}

#[test]
fn invalid_terminal_receipt_never_mutates_the_record() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4(), b"print('invalid')");
    let hash = request.request_hash().unwrap();
    let ledger = RunLedger::open(directory.path()).unwrap();
    let original = ledger.accept(request.clone()).unwrap().record().clone();
    let mut receipt = signed_receipt(&original, RunState::Succeeded, &signing);
    receipt.run_id = Uuid::new_v4();
    let error = ledger
        .commit_terminal(request.request_id, &hash, receipt, &verification)
        .unwrap_err();
    assert!(matches!(error, RunLedgerError::InvalidReceipt(_)));
    assert_eq!(ledger.get(request.request_id).unwrap().unwrap(), original);
    assert!(ledger.receipt_log().segments().unwrap().is_empty());
}
