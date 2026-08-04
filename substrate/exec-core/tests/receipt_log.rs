use std::{collections::BTreeMap, fs, sync::Arc, thread};

use chrono::{TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass, ExecutionBackend,
    ExecutionExit, ExecutionLimits, ExecutionProvenance, ExecutionTier, RequestedTier,
    ResourceUsage, RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest, VerificationKey,
    SCHEMA_VERSION,
};
use prometheus_exec_core::{
    BackendExecution, Ed25519ReceiptSigner, ExecutionJob, ReceiptAssembler, ReceiptLog,
    ReceiptLogError,
};
use tempfile::tempdir;
use uuid::Uuid;

fn signed_receipt(index: u128, key: &SigningKey) -> prometheus_exec_contracts::ExecutionReceipt {
    let code = format!("print({index})").into_bytes();
    let request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::from_u128(index + 1),
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 13, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(&code),
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
    };
    let job = ExecutionJob {
        request,
        code,
        inputs: BTreeMap::new(),
        grants: vec![],
    }
    .validate()
    .unwrap();
    ReceiptAssembler::new(Ed25519ReceiptSigner::new(key.clone()))
        .assemble(
            &job,
            BackendExecution {
                state: RunState::Succeeded,
                evidence_class: EvidenceClass::Attested,
                tier: ExecutionTier::P,
                sandbox_profile_hash: hash_bytes(b"seatbelt"),
                backend: ExecutionBackend::Seatbelt,
                exit: ExecutionExit {
                    status: 0,
                    signal_or_trap: None,
                },
                stdout: index.to_string().into_bytes(),
                stderr: vec![],
                artifacts: vec![],
                usage: ResourceUsage::default(),
                started_at: Utc.with_ymd_and_hms(2026, 8, 4, 13, 0, 1).unwrap(),
                finished_at: Utc.with_ymd_and_hms(2026, 8, 4, 13, 0, 2).unwrap(),
                toolchain_hash: Some(hash_bytes(b"python3")),
                environment: BTreeMap::new(),
                platform: "macos-aarch64".into(),
            },
        )
        .unwrap()
}

#[test]
fn append_is_hash_linked_replay_safe_and_restart_safe() {
    let directory = tempdir().unwrap();
    let key = SigningKey::from_bytes(&[11_u8; 32]);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let log = ReceiptLog::open(directory.path()).unwrap();
    let first_receipt = signed_receipt(1, &key);
    let first = log.append(first_receipt.clone(), &public).unwrap();
    assert!(first.created);
    assert_eq!(first.sequence, 0);

    let replay = log.append(first_receipt, &public).unwrap();
    assert!(!replay.created);
    assert_eq!(replay.segment_hash, first.segment_hash);

    drop(log);
    let reopened = ReceiptLog::open(directory.path()).unwrap();
    let second = reopened.append(signed_receipt(2, &key), &public).unwrap();
    assert_eq!(second.sequence, 1);
    let segments = reopened.segments().unwrap();
    assert_eq!(
        segments[1].header.previous_segment_hash,
        Some(first.segment_hash)
    );
    assert_eq!(
        reopened.verify(|_, _| Some(public.clone())).unwrap().len(),
        2
    );
}

#[test]
fn concurrent_append_serializes_writers_without_lost_receipts() {
    let directory = tempdir().unwrap();
    let key = SigningKey::from_bytes(&[12_u8; 32]);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let log = Arc::new(ReceiptLog::open(directory.path()).unwrap());
    let handles: Vec<_> = (0_u128..8)
        .map(|index| {
            let log = Arc::clone(&log);
            let key = key.clone();
            let public = public.clone();
            thread::spawn(move || {
                log.append(signed_receipt(index + 10, &key), &public)
                    .unwrap()
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }

    let segments = log.segments().unwrap();
    assert_eq!(segments.len(), 8);
    for (sequence, segment) in segments.iter().enumerate() {
        assert_eq!(segment.header.sequence, sequence as u64);
    }
}

#[test]
fn tampered_segment_is_rejected_before_append() {
    let directory = tempdir().unwrap();
    let key = SigningKey::from_bytes(&[13_u8; 32]);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let log = ReceiptLog::open(directory.path()).unwrap();
    log.append(signed_receipt(1, &key), &public).unwrap();

    let path = fs::read_dir(directory.path().join("segments"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["header"]["receiptCount"] = serde_json::json!(7);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    assert!(matches!(
        log.append(signed_receipt(2, &key), &public),
        Err(ReceiptLogError::InvalidChain(_))
    ));
}
