use std::fs;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, key_id, sign_receipt_ed25519, sign_request_ed25519, verify_evidence_bundle,
    ArtifactEvidence, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass, EvidenceFile,
    EvidenceIdentity, ExecutingDevice, ExecutionBackend, ExecutionEvidenceIndex, ExecutionExit,
    ExecutionLimits, ExecutionOutputs, ExecutionProvenance, ExecutionReceipt, ExecutionTier,
    RequestedTier, ResourceUsage, RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest,
    SCHEMA_VERSION,
};
use tempfile::tempdir;
use uuid::Uuid;

fn indexed(path: &str, bytes: &[u8]) -> EvidenceFile {
    EvidenceFile {
        path: path.into(),
        hash: hash_bytes(bytes),
        size_bytes: bytes.len() as u64,
    }
}

fn bundle() -> (tempfile::TempDir, ExecutionEvidenceIndex) {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("outputs")).unwrap();
    let key = SigningKey::from_bytes(&[51; 32]);
    let now = Utc::now();
    let mut request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::new_v4(),
        issued_at: now,
        queued_at: Some(now),
        validity_window_secs: 60,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print(42)"),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: Vec::new(),
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: Vec::new(),
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    sign_request_ed25519(&mut request, &key).unwrap();
    let artifact = b"portable result\n";
    let environment = b"{\"platform\":\"fixture\"}\n";
    let run_id = Uuid::new_v4();
    let mut receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id,
        request_hash: request.request_hash().unwrap(),
        state: RunState::Succeeded,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        code_hash: request.code.hash.clone(),
        input_set_hash: hash_bytes(b"inputs"),
        env_hash: hash_bytes(environment),
        toolchain_hash: Some(hash_bytes(b"python3")),
        sandbox_profile_hash: hash_bytes(b"seatbelt"),
        backend: ExecutionBackend::Seatbelt,
        exit: ExecutionExit {
            status: 0,
            signal_or_trap: None,
        },
        outputs: ExecutionOutputs {
            stdout: hash_bytes(b"42\n"),
            stderr: hash_bytes(b""),
            artifacts: vec![prometheus_exec_contracts::ArtifactReference {
                path: "outputs/result.txt".into(),
                hash: hash_bytes(artifact),
                size_bytes: Some(artifact.len() as u64),
            }],
        },
        usage: ResourceUsage::default(),
        started_at: now,
        finished_at: now,
        executing_device: ExecutingDevice {
            key_id: String::new(),
            sig_alg: SignatureAlgorithm::Ed25519,
            platform: "fixture".into(),
        },
        grants: Vec::new(),
        component: None,
        failure: None,
        signature: None,
    };
    sign_receipt_ed25519(&mut receipt, &key).unwrap();
    let request_bytes = serde_json::to_vec_pretty(&request).unwrap();
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
    fs::write(root.path().join("request.json"), &request_bytes).unwrap();
    fs::write(root.path().join("receipt.json"), &receipt_bytes).unwrap();
    fs::write(root.path().join("outputs/result.txt"), artifact).unwrap();
    fs::write(root.path().join("environment.json"), environment).unwrap();
    let public = key.verifying_key().to_bytes();
    let index = ExecutionEvidenceIndex {
        schema_version: SCHEMA_VERSION.into(),
        requirement_id: "exec.portable-evidence".into(),
        run_id,
        environment: "fixture-offline".into(),
        receipt: indexed("receipt.json", &receipt_bytes),
        request: indexed("request.json", &request_bytes),
        verification_identity: EvidenceIdentity {
            sig_alg: SignatureAlgorithm::Ed25519,
            key_id: key_id(SignatureAlgorithm::Ed25519, &public),
            public_key: URL_SAFE_NO_PAD.encode(public),
        },
        artifacts: vec![ArtifactEvidence {
            receipt_path: "outputs/result.txt".into(),
            file: indexed("outputs/result.txt", artifact),
        }],
        environments: vec![indexed("environment.json", environment)],
    };
    (root, index)
}

#[test]
fn portable_bundle_verifies_without_runtime_state() {
    let (root, index) = bundle();
    let result = verify_evidence_bundle(&index, root.path());
    assert!(result.valid, "{:?}", result.failures);
    assert!(result.index_hash.is_some());
    assert!(result.receipt_hash.is_some());
}

#[test]
fn tamper_and_path_escape_cannot_false_green() {
    let (root, mut index) = bundle();
    fs::write(root.path().join("outputs/result.txt"), b"tampered result\n").unwrap();
    assert!(!verify_evidence_bundle(&index, root.path()).valid);

    index.request.path = "../request.json".into();
    assert!(!verify_evidence_bundle(&index, root.path()).valid);
}
