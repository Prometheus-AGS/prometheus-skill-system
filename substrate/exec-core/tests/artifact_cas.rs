use std::fs;

use std::collections::BTreeMap;

use chrono::Utc;
use prometheus_exec_contracts::{
    hash_bytes, ArtifactReference, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass,
    ExecutingDevice, ExecutionBackend, ExecutionExit, ExecutionLimits, ExecutionOutputs,
    ExecutionProvenance, ExecutionReceipt, ExecutionTier, NamedInput, RequestedTier, ResourceUsage,
    RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest, SCHEMA_VERSION,
};
use prometheus_exec_core::{ArtifactStore, CasError};
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn put_is_atomic_deduplicated_and_corruption_detected() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 1024).unwrap();
    let first = store.put(b"same bytes").unwrap();
    let second = store.put(b"same bytes").unwrap();
    assert_eq!(first, second);
    assert_eq!(store.get(&first.hash).unwrap(), b"same bytes");

    let hex = first.hash.as_str().trim_start_matches("sha256:");
    let path = root
        .path()
        .join("blobs/sha256")
        .join(&hex[..2])
        .join(&hex[2..]);
    fs::write(path, b"tampered").unwrap();
    assert!(matches!(
        store.get(&first.hash),
        Err(CasError::Corrupt { .. })
    ));
}

#[test]
fn output_collection_is_sorted_bounded_and_content_addressed() {
    let store_root = tempdir().unwrap();
    let run_root = tempdir().unwrap();
    fs::create_dir_all(run_root.path().join("outputs/nested")).unwrap();
    fs::write(run_root.path().join("outputs/z.txt"), b"z").unwrap();
    fs::write(run_root.path().join("outputs/nested/a.txt"), b"alpha").unwrap();
    let store = ArtifactStore::open(store_root.path(), 1024).unwrap();

    let artifacts = store.collect_outputs(run_root.path(), 6).unwrap();
    assert_eq!(
        artifacts
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>(),
        vec!["outputs/nested/a.txt", "outputs/z.txt"]
    );
    assert_eq!(store.get(&hash_bytes(b"alpha")).unwrap(), b"alpha");
    assert!(matches!(
        store.collect_outputs(run_root.path(), 5),
        Err(CasError::OutputBudgetExceeded { .. })
    ));
}

#[cfg(unix)]
#[test]
fn output_collection_rejects_symlinks_even_when_the_target_is_inside() {
    use std::os::unix::fs::symlink;

    let store_root = tempdir().unwrap();
    let run_root = tempdir().unwrap();
    fs::create_dir(run_root.path().join("outputs")).unwrap();
    fs::write(run_root.path().join("real.txt"), b"secret").unwrap();
    symlink(
        run_root.path().join("real.txt"),
        run_root.path().join("outputs/link.txt"),
    )
    .unwrap();
    let store = ArtifactStore::open(store_root.path(), 1024).unwrap();

    assert!(matches!(
        store.collect_outputs(run_root.path(), 1024),
        Err(CasError::UnsafeOutput(_))
    ));
}

#[test]
fn garbage_collection_never_removes_pinned_content() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 4).unwrap();
    let pinned = store.put(b"keep").unwrap();
    let removable = store.put(b"discard").unwrap();
    store.pin(&pinned.hash, "open-certification").unwrap();

    let report = store.garbage_collect().unwrap();
    assert_eq!(store.get(&pinned.hash).unwrap(), b"keep");
    assert!(store.get(&removable.hash).is_err());
    assert!(report.pinned.contains(&pinned.hash));
    assert!(report.removed.contains(&removable.hash));
    assert_eq!(report.bytes_after, 4);

    assert!(store.unpin(&pinned.hash, "open-certification").unwrap());
    assert!(!store.is_pinned(&pinned.hash).unwrap());
}

#[test]
fn upload_to_request_transfer_preserves_every_materialized_blob() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 1024).unwrap();
    let code = store.put(b"print('present')").unwrap();
    let missing = hash_bytes(b"missing input");
    let request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::new_v4(),
        issued_at: Utc::now(),
        queued_at: None,
        validity_window_secs: 60,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: code.hash.clone(),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![NamedInput {
            name: "missing.json".into(),
            hash: missing,
        }],
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: Vec::new(),
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };

    let upload_reason = format!("upload:{}", request.request_id);
    store.pin(&code.hash, &upload_reason).unwrap();
    store.transfer_upload_to_request(&request).unwrap();
    assert!(store.is_pinned(&code.hash).unwrap());
    store.release_request(&request).unwrap();
    assert!(!store.is_pinned(&code.hash).unwrap());
}

#[test]
fn receipt_retention_protects_all_materialized_evidence_from_budget_gc() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 1).unwrap();
    let code = store.put(b"print('retained')").unwrap();
    let input = store.put(b"input").unwrap();
    let stdout = store.put(b"stdout").unwrap();
    let stderr = store.put(b"").unwrap();
    let artifact = store.put(b"artifact").unwrap();
    let removable = store.put(b"unreferenced").unwrap();
    let request_id = Uuid::new_v4();
    let request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc::now(),
        queued_at: None,
        validity_window_secs: 60,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: code.hash.clone(),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![NamedInput {
            name: "input.json".into(),
            hash: input.hash.clone(),
        }],
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: Vec::new(),
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    let receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: request_id,
        request_hash: request.request_hash().unwrap(),
        state: RunState::Succeeded,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        code_hash: code.hash.clone(),
        input_set_hash: prometheus_exec_contracts::hash_serializable(&BTreeMap::from([(
            "input.json".to_string(),
            input.hash.clone(),
        )]))
        .unwrap(),
        env_hash: prometheus_exec_contracts::hash_serializable(&BTreeMap::<String, String>::new())
            .unwrap(),
        toolchain_hash: None,
        sandbox_profile_hash: hash_bytes(b"profile"),
        backend: ExecutionBackend::Seatbelt,
        exit: ExecutionExit {
            status: 0,
            signal_or_trap: None,
        },
        outputs: ExecutionOutputs {
            stdout: stdout.hash.clone(),
            stderr: stderr.hash.clone(),
            artifacts: vec![ArtifactReference {
                path: "outputs/result.txt".into(),
                hash: artifact.hash.clone(),
                size_bytes: Some(8),
            }],
        },
        usage: ResourceUsage::default(),
        started_at: Utc::now(),
        finished_at: Utc::now(),
        executing_device: ExecutingDevice {
            key_id: "fixture".into(),
            sig_alg: SignatureAlgorithm::Ed25519,
            platform: "macos-test".into(),
        },
        grants: Vec::new(),
        signature: None,
    };

    let upload_reason = format!("upload:{}", request.request_id);
    store.pin(&code.hash, &upload_reason).unwrap();
    store.pin(&input.hash, &upload_reason).unwrap();
    store.retain_for_request(&request).unwrap();
    store.retain_for_receipt(&request, &receipt).unwrap();
    store.release_upload(&request).unwrap();
    store.release_request(&request).unwrap();
    let report = store.garbage_collect().unwrap();

    for retained in [
        &code.hash,
        &input.hash,
        &stdout.hash,
        &stderr.hash,
        &artifact.hash,
    ] {
        assert!(
            store.is_pinned(retained).unwrap(),
            "{retained} was not pinned"
        );
        assert!(store.get(retained).is_ok(), "{retained} was collected");
    }
    assert!(report.removed.contains(&removable.hash));
    assert!(store.get(&removable.hash).is_err());
    assert!(
        report.bytes_after > 1,
        "pinned evidence may exceed the CAS budget"
    );

    store.release_receipt(&request, &receipt).unwrap();
    for released in [
        &code.hash,
        &input.hash,
        &stdout.hash,
        &stderr.hash,
        &artifact.hash,
    ] {
        assert!(!store.is_pinned(released).unwrap());
    }
}
