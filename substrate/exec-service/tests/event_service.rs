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
    ExecutionService, ExecutionServiceError, RunEventData, RunEventLog, RunEventLogError, RunRecord,
};
use tempfile::tempdir;
use uuid::Uuid;

fn request(request_id: Uuid) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 16, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print('events')"),
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
    let signing = SigningKey::from_bytes(&[41_u8; 32]);
    let verification = VerificationKey::ed25519(signing.verifying_key().to_bytes());
    (signing, verification)
}

fn signed_receipt(record: &RunRecord, key: &SigningKey) -> ExecutionReceipt {
    let started_at = Utc.with_ymd_and_hms(2026, 8, 4, 16, 0, 1).unwrap();
    let finished_at = Utc.with_ymd_and_hms(2026, 8, 4, 16, 0, 2).unwrap();
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
        component: None,
        failure: None,
        signature: None,
    };
    sign_receipt_ed25519(&mut receipt, key).unwrap();
    receipt
}

#[test]
fn event_log_serializes_concurrent_writers_and_resumes_after_sequence() {
    let directory = tempdir().unwrap();
    let log = Arc::new(RunEventLog::open(directory.path()).unwrap());
    let run_id = Uuid::new_v4();
    let occurred_at = Utc.with_ymd_and_hms(2026, 8, 4, 16, 1, 0).unwrap();
    let handles: Vec<_> = (0..16)
        .map(|index| {
            let log = log.clone();
            thread::spawn(move || {
                log.append(
                    run_id,
                    format!("stdout.{index}"),
                    occurred_at,
                    RunEventData::Stdout {
                        chunk: index.to_string(),
                    },
                )
                .unwrap()
            })
        })
        .collect();
    for handle in handles {
        assert!(handle.join().unwrap().created);
    }
    let events = log.events(run_id).unwrap();
    assert_eq!(events.len(), 16);
    assert!(events
        .iter()
        .enumerate()
        .all(|(index, event)| event.sequence == index as u64 + 1));
    let resumed = log.events_after(run_id, 11).unwrap();
    assert_eq!(
        resumed
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![12, 13, 14, 15, 16]
    );
    let (page, has_more) = log.events_page_after(run_id, 11, 2, 1024 * 1024).unwrap();
    assert_eq!(
        page.iter().map(|event| event.sequence).collect::<Vec<_>>(),
        vec![12, 13]
    );
    assert!(has_more);
    let (last_page, has_more) = log.events_page_after(run_id, 15, 2, 1024 * 1024).unwrap();
    assert_eq!(last_page[0].sequence, 16);
    assert!(!has_more);
    let error = log
        .events_page_after(run_id, 11, 2, 1)
        .expect_err("an oversized first event must not create a stalled cursor");
    assert!(error.to_string().contains("exceeds page byte limit 1"));
}

#[test]
fn event_identity_replays_exact_content_and_rejects_conflicts() {
    let directory = tempdir().unwrap();
    let log = RunEventLog::open(directory.path()).unwrap();
    let run_id = Uuid::new_v4();
    let occurred_at = Utc.with_ymd_and_hms(2026, 8, 4, 16, 2, 0).unwrap();
    let first = log
        .append(
            run_id,
            "progress.compile",
            occurred_at,
            RunEventData::Progress {
                message: "compiled".into(),
                completed: Some(1),
                total: Some(2),
            },
        )
        .unwrap();
    let replay = log
        .append(
            run_id,
            "progress.compile",
            occurred_at,
            RunEventData::Progress {
                message: "compiled".into(),
                completed: Some(1),
                total: Some(2),
            },
        )
        .unwrap();
    assert!(!replay.created);
    assert_eq!(replay.event, first.event);

    let error = log
        .append(
            run_id,
            "progress.compile",
            occurred_at,
            RunEventData::Progress {
                message: "different".into(),
                completed: Some(2),
                total: Some(2),
            },
        )
        .unwrap_err();
    assert!(matches!(error, RunEventLogError::EventIdConflict { .. }));
    assert_eq!(log.events(run_id).unwrap(), vec![first.event]);
}

#[test]
fn response_loss_returns_original_receipt_and_exact_event_history_after_restart() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4());
    let hash = request.request_hash().unwrap();
    let service = ExecutionService::open(directory.path()).unwrap();
    let submitted = service.submit(request.clone()).unwrap();
    let run_id = submitted.record.run_id;
    service.mark_spawned(request.request_id, &hash).unwrap();
    service
        .append_runtime_event(
            run_id,
            "stdout.1",
            Utc.with_ymd_and_hms(2026, 8, 4, 16, 0, 1).unwrap(),
            RunEventData::Stdout {
                chunk: "evidence\n".into(),
            },
        )
        .unwrap();
    let receipt = signed_receipt(&submitted.record, &signing);
    service
        .commit_terminal(request.request_id, &hash, receipt.clone(), &verification)
        .unwrap();
    let original_events = service.events_after(run_id, 0).unwrap();
    assert_eq!(original_events.len(), 4);
    drop(service);

    let reopened = ExecutionService::open(directory.path()).unwrap();
    let replay = reopened.submit(request).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.record.run_id, run_id);
    assert_eq!(reopened.receipt(run_id).unwrap(), Some(receipt));
    assert_eq!(reopened.events_after(run_id, 0).unwrap(), original_events);
    assert!(reopened.events_after(run_id, 4).unwrap().is_empty());
}

#[test]
fn opening_service_reconstructs_lifecycle_events_from_durable_state() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let request = request(Uuid::new_v4());
    let hash = request.request_hash().unwrap();
    let service = ExecutionService::open(directory.path()).unwrap();
    let record = service
        .ledger()
        .accept(request.clone())
        .unwrap()
        .record()
        .clone();
    service
        .ledger()
        .mark_spawned(request.request_id, &hash)
        .unwrap();
    let receipt = signed_receipt(&record, &signing);
    service
        .ledger()
        .commit_terminal(request.request_id, &hash, receipt, &verification)
        .unwrap();
    assert!(service
        .event_log()
        .events(record.run_id)
        .unwrap()
        .is_empty());
    drop(service);

    let reopened = ExecutionService::open(directory.path()).unwrap();
    let events = reopened.events_after(record.run_id, 0).unwrap();
    assert_eq!(events.len(), 3);
    assert!(matches!(events[0].data, RunEventData::Accepted { .. }));
    assert_eq!(events[0].sequence, 1);
    assert_eq!(events[1].data, RunEventData::Started);
    assert!(matches!(events[2].data, RunEventData::Completed { .. }));
}

#[test]
fn runtime_events_cannot_forge_lifecycle_or_append_after_completion() {
    let directory = tempdir().unwrap();
    let (signing, verification) = signing_material();
    let pending_request = request(Uuid::new_v4());
    let hash = pending_request.request_hash().unwrap();
    let service = ExecutionService::open(directory.path()).unwrap();
    let record = service.submit(pending_request.clone()).unwrap().record;
    let reserved = service
        .append_runtime_event(
            record.run_id,
            "forged.complete",
            Utc::now(),
            RunEventData::Completed {
                state: RunState::Succeeded,
                receipt_hash: hash_bytes(b"forged"),
            },
        )
        .unwrap_err();
    assert!(matches!(
        reserved,
        ExecutionServiceError::ReservedLifecycleEvent
    ));
    let before_spawn = service
        .append_runtime_event(
            record.run_id,
            "stdout.early",
            Utc::now(),
            RunEventData::Stdout {
                chunk: "early".into(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        before_spawn,
        ExecutionServiceError::RunNotSpawned(_)
    ));
    let pending = service
        .mark_grant_pending(
            pending_request.request_id,
            &hash,
            "grant.waiting",
            "operator approval required",
        )
        .unwrap();
    assert_eq!(pending.state, RunState::GrantPending);
    assert_eq!(
        pending.spawn,
        prometheus_exec_service::SpawnStatus::NotSpawned
    );
    let forged_pending = service
        .append_runtime_event(
            record.run_id,
            "grant.forged",
            Utc::now(),
            RunEventData::GrantPending {
                reason: "not service-owned".into(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        forged_pending,
        ExecutionServiceError::ReservedLifecycleEvent
    ));
    drop(service);

    let service = ExecutionService::open(directory.path()).unwrap();
    let recovered = service.run(record.run_id).unwrap().unwrap();
    assert_eq!(recovered.state, RunState::GrantPending);
    assert_eq!(
        service
            .events_after(record.run_id, 0)
            .unwrap()
            .iter()
            .filter(|event| matches!(event.data, RunEventData::GrantPending { .. }))
            .count(),
        1
    );

    let completed_request = request(Uuid::new_v4());
    let completed_hash = completed_request.request_hash().unwrap();
    let completed = service.submit(completed_request.clone()).unwrap().record;
    service
        .mark_spawned(completed_request.request_id, &completed_hash)
        .unwrap();
    service
        .commit_terminal(
            completed_request.request_id,
            &completed_hash,
            signed_receipt(&completed, &signing),
            &verification,
        )
        .unwrap();
    let late = service
        .append_runtime_event(
            completed.run_id,
            "stdout.late",
            Utc::now(),
            RunEventData::Stdout {
                chunk: "late".into(),
            },
        )
        .unwrap_err();
    assert!(matches!(late, ExecutionServiceError::RunTerminal(_)));
}

#[test]
fn tampered_event_chain_is_rejected_on_reopen() {
    let directory = tempdir().unwrap();
    let log = RunEventLog::open(directory.path()).unwrap();
    let run_id = Uuid::new_v4();
    log.append(
        run_id,
        "stdout.1",
        Utc.with_ymd_and_hms(2026, 8, 4, 16, 3, 0).unwrap(),
        RunEventData::Stdout {
            chunk: "original".into(),
        },
    )
    .unwrap();
    let segments = directory.path().join(run_id.to_string()).join("segments");
    let path = std::fs::read_dir(segments)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["chunk"] = serde_json::Value::String("tampered".into());
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let error = RunEventLog::open(directory.path()).unwrap_err();
    assert!(matches!(error, RunEventLogError::InvalidEvent(_)));
}
