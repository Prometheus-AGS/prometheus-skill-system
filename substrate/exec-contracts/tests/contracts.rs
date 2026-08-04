use std::fs;

use chrono::{TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use p256::ecdsa::SigningKey as P256SigningKey;
use prometheus_exec_contracts::*;
use proptest::prelude::*;
use rand_core::OsRng;
use tempfile::tempdir;
use uuid::Uuid;

fn digest(label: &str) -> Digest {
    hash_bytes(label.as_bytes())
}

fn request() -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::parse_str("16c43239-a1c7-44e6-81e8-a5ac36fcb201").unwrap(),
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::W,
        code: CodeIdentity {
            kind: CodeKind::Component,
            hash: digest("component"),
            runtime: RuntimeKind::WasmComponent,
            toolchain_pin: None,
        },
        inputs: vec![NamedInput {
            name: "prompt".into(),
            hash: digest("input"),
        }],
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits::default(),
        targets: vec![],
        provenance: ExecutionProvenance {
            skill: Some("refine-validate".into()),
            component_authorization: Some(ComponentAuthorization {
                mode: ComponentAuthorizationMode::HashPin,
                world: "prometheus:component@0.1.0".into(),
                manifest_hash: None,
                generation_id: None,
            }),
            ..ExecutionProvenance::default()
        },
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    }
}

fn receipt(request_hash: Digest) -> ExecutionReceipt {
    ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: Uuid::parse_str("465915e9-0ec4-45e2-9daa-7d3f6eb68fb6").unwrap(),
        request_hash,
        state: RunState::Succeeded,
        evidence_class: EvidenceClass::Verified,
        tier: ExecutionTier::W,
        code_hash: digest("component"),
        input_set_hash: digest("inputs"),
        env_hash: digest("env"),
        toolchain_hash: None,
        sandbox_profile_hash: digest("wasmtime-limits"),
        backend: ExecutionBackend::Cranelift,
        exit: ExecutionExit {
            status: 0,
            signal_or_trap: None,
        },
        outputs: ExecutionOutputs {
            stdout: digest("stdout"),
            stderr: digest("stderr"),
            artifacts: vec![ArtifactReference {
                path: "outputs/result.json".into(),
                hash: digest("artifact"),
                size_bytes: Some(8),
            }],
        },
        usage: ResourceUsage::default(),
        started_at: Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 1).unwrap(),
        finished_at: Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 2).unwrap(),
        executing_device: ExecutingDevice {
            key_id: String::new(),
            sig_alg: SignatureAlgorithm::Ed25519,
            platform: "macos-aarch64".into(),
        },
        grants: vec![ExecutionGrant {
            kind: GrantKind::CedarAuto,
            r#ref: None,
        }],
        component: Some(ComponentProvenance {
            authorization: request().provenance.component_authorization.unwrap(),
            engine_version: "wasmtime 46.0.0".into(),
            backend_profile_hash: digest("cranelift-profile"),
        }),
        failure: None,
        signature: None,
    }
}

#[test]
fn canonical_request_is_stable_across_object_order() {
    let request = request();
    let value = serde_json::to_value(&request).unwrap();
    let mut reversed = serde_json::Map::new();
    for (key, value) in value.as_object().unwrap().iter().rev() {
        reversed.insert(key.clone(), value.clone());
    }
    assert_eq!(
        canonical_bytes(&value).unwrap(),
        canonical_bytes(&serde_json::Value::Object(reversed)).unwrap()
    );
}

#[test]
fn request_rejects_every_zero_enforcement_limit() {
    for field in ["memory", "wall-clock", "output", "stack"] {
        let mut candidate = request();
        match field {
            "memory" => candidate.limits.memory_mb = 0,
            "wall-clock" => candidate.limits.wall_clock_ms = 0,
            "output" => candidate.limits.output_mb = 0,
            "stack" => candidate.limits.stack_kb = 0,
            _ => unreachable!(),
        }
        assert!(candidate.validate().is_err(), "zero {field} limit passed");
    }
}

#[test]
fn tier_w_requires_pinned_component_authorization() {
    let mut candidate = request();
    candidate.provenance.component_authorization = None;
    assert!(candidate.validate().is_err());

    let mut candidate = request();
    candidate.provenance.component_authorization = Some(ComponentAuthorization {
        mode: ComponentAuthorizationMode::SignedGeneration,
        world: "prometheus:component@0.1.0".into(),
        manifest_hash: None,
        generation_id: None,
    });
    assert!(candidate.validate().is_err());
}

#[test]
fn tier_p_wire_shape_omits_tier_w_extensions() {
    let mut native_request = request();
    native_request.tier = RequestedTier::P;
    native_request.code.kind = CodeKind::File;
    native_request.code.runtime = RuntimeKind::Python3;
    native_request.provenance.component_authorization = None;
    let request_json = serde_json::to_value(&native_request).unwrap();
    assert!(request_json["provenance"]
        .get("componentAuthorization")
        .is_none());

    let mut native_receipt = receipt(native_request.request_hash().unwrap());
    native_receipt.tier = ExecutionTier::P;
    native_receipt.evidence_class = EvidenceClass::Attested;
    native_receipt.backend = ExecutionBackend::Seatbelt;
    native_receipt.component = None;
    let receipt_json = serde_json::to_value(&native_receipt).unwrap();
    assert!(receipt_json.get("component").is_none());
    assert!(receipt_json.get("failure").is_none());
    native_receipt.validate().unwrap();
}

#[test]
fn tier_w_pre_execution_rejection_is_honest_without_component_provenance() {
    let mut rejected = receipt(request().request_hash().unwrap());
    rejected.state = RunState::Rejected;
    rejected.exit.status = 125;
    rejected.component = None;
    rejected.failure = Some(ExecutionFailure {
        kind: ExecutionFailureKind::ComponentUnauthorized,
        code: "component_unauthorized".into(),
        message: "component did not pass the configured trust policy".into(),
    });
    rejected.validate().unwrap();

    rejected.failure = None;
    assert!(rejected.validate().is_err());
}

#[test]
fn ed25519_receipt_mutation_fails() {
    let key = SigningKey::generate(&mut OsRng);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let request = request();
    let mut receipt = receipt(request.request_hash().unwrap());
    sign_receipt_ed25519(&mut receipt, &key).unwrap();
    assert!(verify_receipt(&receipt, &public, Some(&request), None).valid);

    receipt.exit.status = 1;
    let result = verify_receipt(&receipt, &public, Some(&request), None);
    assert!(!result.valid);
    assert!(result
        .failures
        .iter()
        .any(|failure| failure.code == "receipt.signature"));
}

#[test]
fn p256_receipt_verifies_and_algorithm_mismatch_fails() {
    let key = P256SigningKey::random(&mut OsRng);
    let encoded = key.verifying_key().to_encoded_point(true);
    let public = VerificationKey::p256_sec1(encoded.as_bytes());
    let mut receipt = receipt(request().request_hash().unwrap());
    sign_receipt_p256(&mut receipt, &key).unwrap();
    assert!(verify_receipt(&receipt, &public, None, None).valid);

    let wrong = VerificationKey::ed25519([0_u8; 32]);
    assert!(!verify_receipt(&receipt, &wrong, None, None).valid);
}

#[test]
fn artifact_tampering_and_path_escape_fail() {
    let key = SigningKey::generate(&mut OsRng);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let temp = tempdir().unwrap();
    fs::create_dir(temp.path().join("outputs")).unwrap();
    fs::write(temp.path().join("outputs/result.json"), b"tampered").unwrap();
    let mut receipt = receipt(request().request_hash().unwrap());
    sign_receipt_ed25519(&mut receipt, &key).unwrap();
    let result = verify_receipt(&receipt, &public, None, Some(temp.path()));
    assert!(!result.valid);
    assert!(result
        .failures
        .iter()
        .any(|failure| failure.code == "artifact.hash_mismatch"));

    receipt.outputs.artifacts[0].path = "outputs/../../secret".into();
    assert!(matches!(
        receipt.validate(),
        Err(ContractError::UnsafeArtifactPath(_))
    ));
}

#[test]
fn receipt_segment_detects_chain_and_entry_corruption() {
    let key = SigningKey::generate(&mut OsRng);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let mut receipt = receipt(request().request_hash().unwrap());
    sign_receipt_ed25519(&mut receipt, &key).unwrap();
    let entry = ReceiptLogEntry::new(receipt).unwrap();
    let segment = ReceiptLogSegment::seal(
        7,
        Some(digest("previous")),
        Utc.with_ymd_and_hms(2026, 8, 4, 10, 1, 0).unwrap(),
        vec![entry],
    )
    .unwrap();
    let expected = digest("previous");
    assert_eq!(
        segment
            .verify(Some(&expected), |_, _| Some(public.clone()))
            .unwrap()
            .len(),
        1
    );
    assert!(segment.verify(None, |_, _| Some(public.clone())).is_err());

    let mut corrupted = segment;
    corrupted.entries[0].receipt.exit.status = 9;
    assert!(corrupted
        .verify(Some(&expected), |_, _| Some(public.clone()))
        .is_err());
}

#[test]
fn generated_contracts_are_deterministic() {
    let first = serde_json::to_vec_pretty(&openapi_components()).unwrap();
    let second = serde_json::to_vec_pretty(&openapi_components()).unwrap();
    assert_eq!(first, second);
}

proptest! {
    #[test]
    fn any_signed_receipt_usage_mutation_is_detected(cpu_ms in any::<u64>()) {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
        let mut signed = receipt(request().request_hash().unwrap());
        signed.usage.cpu_ms = cpu_ms;
        sign_receipt_ed25519(&mut signed, &key).unwrap();

        let mut mutated = signed.clone();
        mutated.usage.cpu_ms = cpu_ms.wrapping_add(1);
        prop_assert!(!verify_receipt(&mutated, &public, None, None).valid);
        prop_assert!(verify_receipt(&signed, &public, None, None).valid);
    }
}
