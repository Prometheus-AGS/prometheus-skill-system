use std::collections::BTreeMap;

use chrono::{TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, verify_receipt, CapabilityManifest, CodeIdentity, CodeKind, EvidenceClass,
    ExecutionBackend, ExecutionExit, ExecutionGrant, ExecutionLimits, ExecutionProvenance,
    ExecutionTier, GrantKind, NamedInput, RequestedTier, ResourceUsage, RunState, RuntimeKind,
    SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::{
    BackendExecution, BaselinePolicy, Ed25519ReceiptSigner, ExecutionJob, PolicyEvaluator,
    PolicyOutcome, PolicyReason, ProducedArtifact, ReceiptAssembler,
};
use uuid::Uuid;

fn request(code: &[u8], input: &[u8]) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::parse_str("b2d2dfe0-a4e3-4e9b-a928-623fbf73741e").unwrap(),
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(code),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![NamedInput {
            name: "input.txt".into(),
            hash: hash_bytes(input),
        }],
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: vec![],
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    }
}

fn job() -> ExecutionJob {
    ExecutionJob {
        request: request(b"print('ok')", b"hello"),
        code: b"print('ok')".to_vec(),
        inputs: BTreeMap::from([("input.txt".into(), b"hello".to_vec())]),
        grants: vec![ExecutionGrant {
            kind: GrantKind::CedarAuto,
            r#ref: None,
        }],
    }
}

#[test]
fn job_validation_binds_code_and_inputs() {
    let validated = job().validate().unwrap();
    assert_eq!(validated.code(), b"print('ok')");
    assert_eq!(validated.inputs()["input.txt"], b"hello");

    let mut changed = job();
    changed.code.push(b'!');
    assert!(changed.validate().is_err());

    let mut changed = job();
    changed
        .inputs
        .insert("input.txt".into(), b"changed".to_vec());
    assert!(changed.validate().is_err());
}

#[test]
fn baseline_policy_is_deterministic_and_never_broadens() {
    let policy = BaselinePolicy;
    let mut request = job().request;
    assert_eq!(policy.evaluate(&request), PolicyOutcome::AutoApproved);

    request.capabilities.net.egress = vec!["example.com:443".into()];
    request.capabilities.env.read = vec!["TOKEN".into()];
    let expected = PolicyOutcome::GrantRequired {
        reasons: vec![
            PolicyReason::NetworkEgress,
            PolicyReason::EnvironmentPassthrough,
        ],
    };
    assert_eq!(policy.evaluate(&request), expected);
    assert_eq!(policy.evaluate(&request), expected);
}

#[test]
fn receipt_assembly_produces_portable_signed_evidence() {
    let validated = job().validate().unwrap();
    let signer = Ed25519ReceiptSigner::new(SigningKey::from_bytes(&[9_u8; 32]));
    let public = VerificationKey::ed25519(signer.public_key());
    let assembler = ReceiptAssembler::new(signer);
    let execution = BackendExecution {
        state: RunState::Succeeded,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        sandbox_profile_hash: hash_bytes(b"seatbelt-profile"),
        backend: ExecutionBackend::Seatbelt,
        exit: ExecutionExit {
            status: 0,
            signal_or_trap: None,
        },
        stdout: b"ok\n".to_vec(),
        stderr: Vec::new(),
        artifacts: vec![ProducedArtifact {
            path: "outputs/result.txt".into(),
            bytes: b"result".to_vec(),
        }],
        usage: ResourceUsage {
            wall_clock_ms: 4,
            cpu_ms: 2,
            peak_mem_mb: 8,
            fuel_consumed: 0,
        },
        started_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 1).unwrap(),
        finished_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 2).unwrap(),
        toolchain_hash: Some(hash_bytes(b"python3-toolchain")),
        environment: BTreeMap::new(),
        platform: "macos-aarch64".into(),
        component: None,
        failure: None,
    };

    let run_id = Uuid::new_v4();
    let receipt = assembler
        .assemble_for_run(run_id, &validated, execution)
        .unwrap();
    assert_eq!(receipt.run_id, run_id);
    assert_eq!(receipt.evidence_class, EvidenceClass::Attested);
    assert_eq!(receipt.outputs.stdout, hash_bytes(b"ok\n"));
    assert_eq!(receipt.grants.len(), 1);
    assert!(verify_receipt(&receipt, &public, Some(validated.request()), None).valid);
}
