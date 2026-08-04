use std::collections::BTreeMap;

use prometheus_exec_contracts::{
    hash_bytes, ArtifactReference, EvidenceClass, ExecutionBackend, ExecutionReceipt,
    ExecutionTier, SignedExecRequest, TierWReplayResult, VerificationKey,
};
use prometheus_exec_core::ExecutionJob;

use crate::{
    ComponentAuthorizer, EngineProfile, TierWEngine, TierWError, TierWExecutionPort, TierWPortError,
};

#[derive(Debug, thiserror::Error)]
pub enum TierWReplayError {
    #[error("portable replay engine failed: {0}")]
    Engine(#[from] TierWError),
    #[error("portable replay request failed validation: {0}")]
    Job(#[from] prometheus_exec_core::JobValidationError),
    #[error("portable replay execution failed: {0}")]
    Port(#[from] TierWPortError),
}

/// Re-executes a cryptographically verified Tier W receipt with the portable
/// Pulley profile. The transport-free signature/hash verifier remains in
/// `prometheus-exec-contracts`; this function is an optional backend replay.
pub fn replay_verified_receipt(
    receipt: &ExecutionReceipt,
    request: &SignedExecRequest,
    verification_key: &VerificationKey,
    component_bytes: Vec<u8>,
    inputs: BTreeMap<String, Vec<u8>>,
) -> Result<TierWReplayResult, TierWReplayError> {
    let verification =
        prometheus_exec_contracts::verify_receipt(receipt, verification_key, Some(request), None);
    if !verification.valid {
        return Ok(TierWReplayResult {
            valid: false,
            checks: verification
                .checks
                .into_iter()
                .map(|check| format!("transport.{}", check.code))
                .collect(),
            mismatches: verification
                .failures
                .into_iter()
                .map(|failure| format!("transport.{}", failure.code))
                .collect(),
        });
    }

    let Some(expected_component) = receipt.component.as_ref() else {
        return Ok(mismatch("receipt.component"));
    };
    if receipt.tier != ExecutionTier::W {
        return Ok(mismatch("receipt.tier"));
    }
    if receipt.evidence_class != EvidenceClass::Verified {
        return Ok(mismatch("receipt.evidenceClass"));
    }
    if !matches!(
        receipt.backend,
        ExecutionBackend::Cranelift | ExecutionBackend::Pulley
    ) {
        return Ok(mismatch("receipt.backend"));
    }

    let job = ExecutionJob {
        request: request.clone(),
        code: component_bytes,
        inputs,
        grants: receipt.grants.clone(),
    }
    .validate()?;
    let input_set_hash_matches = job.input_set_hash() == &receipt.input_set_hash;
    let authorizer = ComponentAuthorizer::verified_receipt(
        receipt.code_hash.clone(),
        expected_component.authorization.clone(),
    );
    let port = TierWExecutionPort::new(
        TierWEngine::new(EngineProfile::portable_replay())?,
        authorizer,
    );
    let replay = port.execute_job(&job)?;

    let replay_artifacts: Vec<_> = replay
        .artifacts
        .iter()
        .map(|artifact| ArtifactReference {
            path: artifact.path.clone(),
            hash: hash_bytes(&artifact.bytes),
            size_bytes: Some(artifact.bytes.len() as u64),
        })
        .collect();
    let actual_component = replay
        .component
        .as_ref()
        .expect("Tier W execution always records component provenance");
    let fields = [
        ("request.inputSetHash", input_set_hash_matches),
        ("receipt.state", replay.state == receipt.state),
        (
            "receipt.stdout",
            hash_bytes(&replay.stdout) == receipt.outputs.stdout,
        ),
        (
            "receipt.stderr",
            hash_bytes(&replay.stderr) == receipt.outputs.stderr,
        ),
        (
            "receipt.artifacts",
            replay_artifacts == receipt.outputs.artifacts,
        ),
        ("receipt.failure", replay.failure == receipt.failure),
        (
            "component.authorization",
            actual_component.authorization == expected_component.authorization,
        ),
        (
            "component.engineVersion",
            actual_component.engine_version == expected_component.engine_version,
        ),
        (
            "component.deterministicProjectionHash",
            actual_component.deterministic_projection_hash
                == expected_component.deterministic_projection_hash,
        ),
        ("replay.backend", replay.backend == ExecutionBackend::Pulley),
    ];
    let mut result = TierWReplayResult {
        valid: true,
        checks: vec!["transport.receipt-and-request".into()],
        mismatches: Vec::new(),
    };
    for (field, matches) in fields {
        if matches {
            result.checks.push(field.into());
        } else {
            result.valid = false;
            result.mismatches.push(field.into());
        }
    }
    Ok(result)
}

fn mismatch(field: &str) -> TierWReplayResult {
    TierWReplayResult {
        valid: false,
        checks: vec!["transport.receipt-and-request".into()],
        mismatches: vec![field.into()],
    }
}

#[cfg(all(test, feature = "cranelift", feature = "pulley"))]
mod tests {
    use super::*;
    use chrono::{TimeZone as _, Utc};
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorization,
        ComponentAuthorizationMode, ExecutionLimits, ExecutionProvenance, RequestedTier,
        RuntimeKind, SignatureAlgorithm, SCHEMA_VERSION,
    };
    use prometheus_exec_core::{Ed25519ReceiptSigner, ReceiptAssembler};
    use uuid::Uuid;

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    fn request() -> SignedExecRequest {
        SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id: Uuid::from_u128(0x33),
            issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 30, 0).unwrap(),
            queued_at: None,
            validity_window_secs: 300,
            tier: RequestedTier::W,
            code: CodeIdentity {
                kind: CodeKind::Component,
                hash: hash_bytes(REFERENCE_COMPONENT),
                runtime: RuntimeKind::WasmComponent,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities: CapabilityManifest::default(),
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                component_authorization: Some(ComponentAuthorization {
                    mode: ComponentAuthorizationMode::HashPin,
                    world: crate::COMPONENT_WORLD.into(),
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

    fn original_receipt() -> (SignedExecRequest, ExecutionReceipt, VerificationKey) {
        let request = request();
        let job = ExecutionJob {
            request: request.clone(),
            code: REFERENCE_COMPONENT.to_vec(),
            inputs: BTreeMap::new(),
            grants: Vec::new(),
        }
        .validate()
        .unwrap();
        let port = TierWExecutionPort::new(
            TierWEngine::new(EngineProfile::desktop()).unwrap(),
            ComponentAuthorizer::hash_pins([request.code.hash.clone()]),
        );
        let execution = port.execute_job(&job).unwrap();
        let signer = Ed25519ReceiptSigner::new(SigningKey::from_bytes(&[63; 32]));
        let key = VerificationKey::ed25519(signer.public_key());
        let receipt = ReceiptAssembler::new(signer)
            .assemble_for_run(Uuid::from_u128(0x44), &job, execution)
            .unwrap();
        (request, receipt, key)
    }

    #[test]
    fn portable_pulley_replay_matches_signed_cranelift_receipt() {
        let (request, receipt, key) = original_receipt();
        assert_eq!(receipt.backend, ExecutionBackend::Cranelift);

        let result = replay_verified_receipt(
            &receipt,
            &request,
            &key,
            REFERENCE_COMPONENT.to_vec(),
            BTreeMap::new(),
        )
        .unwrap();

        assert!(result.valid, "{:?}", result.mismatches);
        assert!(result.checks.contains(&"replay.backend".into()));
        assert!(result
            .checks
            .contains(&"component.deterministicProjectionHash".into()));
    }

    #[test]
    fn replay_refuses_a_receipt_with_a_modified_output_hash() {
        let (request, mut receipt, key) = original_receipt();
        receipt.outputs.stdout = hash_bytes(b"tampered");

        let result = replay_verified_receipt(
            &receipt,
            &request,
            &key,
            REFERENCE_COMPONENT.to_vec(),
            BTreeMap::new(),
        )
        .unwrap();

        assert!(!result.valid);
        assert!(result
            .mismatches
            .iter()
            .any(|field| field == "transport.receipt.signature"));
    }
}
