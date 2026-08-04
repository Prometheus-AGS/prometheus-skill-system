#![cfg(feature = "standalone")]

use std::collections::BTreeMap;

use chrono::Utc;
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorization,
    ComponentAuthorizationMode, EnvironmentCapabilities, ExecutionLimits, ExecutionProvenance,
    FilesystemCapabilities, NetworkCapabilities, RequestedTier, RunState, RuntimeKind,
    SignatureAlgorithm, SignedExecRequest, SCHEMA_VERSION,
};
use prometheus_exec_embedded::{
    EmbeddedExecutionApi, EmbeddedExecutionConfig, EmbeddedRunRequest, EmbeddedVerifyRequest,
};
use tempfile::tempdir;
use uuid::Uuid;

const REFERENCE_COMPONENT: &[u8] = include_bytes!(
    "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
);

fn request(request_id: Uuid, capabilities: CapabilityManifest) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc::now(),
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
        capabilities,
        limits: ExecutionLimits::default(),
        targets: Vec::new(),
        provenance: ExecutionProvenance {
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

fn open_api(root: &std::path::Path) -> EmbeddedExecutionApi {
    EmbeddedExecutionApi::open(EmbeddedExecutionConfig {
        state_root: root.into(),
        signing_key: SigningKey::from_bytes(&[117; 32]),
        component_pins: vec![hash_bytes(REFERENCE_COMPONENT)],
        artifact_budget_bytes: Some(64 * 1024 * 1024),
    })
    .unwrap()
}

fn reference_capabilities() -> CapabilityManifest {
    CapabilityManifest {
        fs: FilesystemCapabilities {
            read_only: vec![
                ".kbd-orchestrator/project.json".into(),
                ".evolver/".into(),
                ".refiner/".into(),
                "openspec/".into(),
            ],
            read_write: vec!["outputs/".into()],
        },
        net: NetworkCapabilities::default(),
        env: EnvironmentCapabilities::default(),
        clock: false,
        random: false,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn one_api_runs_reads_ordered_evidence_and_exactly_replays() {
    let directory = tempdir().unwrap();
    let api = open_api(directory.path());
    let request = request(Uuid::new_v4(), reference_capabilities());
    let invocation = EmbeddedRunRequest {
        request: request.clone(),
        component: REFERENCE_COMPONENT.to_vec(),
        inputs: BTreeMap::new(),
    };

    let first = api.run(invocation.clone()).await.unwrap();
    assert_eq!(
        first.state,
        RunState::Succeeded,
        "{:?}",
        first
            .receipt
            .as_ref()
            .and_then(|receipt| receipt.failure.as_ref())
    );
    assert!(!first.replayed);
    let receipt = first.receipt.clone().unwrap();

    let events = api.events(first.run_id, 0).await.unwrap();
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(events[0].event_id, "run.accepted");
    assert_eq!(events[1].event_id, "run.started");
    assert_eq!(events[2].event_id, "run.completed");
    assert_eq!(api.events(first.run_id, 1).await.unwrap(), events[1..]);

    assert_eq!(
        api.receipt(first.run_id).await.unwrap(),
        Some(receipt.clone())
    );
    let stdout = api.artifact(receipt.outputs.stdout.clone()).await.unwrap();
    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        r#"{"kbd":{"available":false},"evolver":{"available":false},"refiner":{"available":false},"openspec":{"available":false}}"#
    );
    let verification = api
        .verify(EmbeddedVerifyRequest {
            receipt: receipt.clone(),
            verification_key: api.verification_key(),
            request: Some(request),
            artifact_root: None,
            replay: None,
        })
        .await
        .unwrap();
    assert!(verification.valid, "{:?}", verification.failures);

    drop(api);
    let api = open_api(directory.path());
    let replay = api.run(invocation).await.unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.run_id, first.run_id);
    assert_eq!(replay.receipt, Some(receipt));
}

#[tokio::test(flavor = "multi_thread")]
async fn privileged_request_stops_at_durable_grant_pending_without_spawn() {
    let directory = tempdir().unwrap();
    let api = open_api(directory.path());
    let mut capabilities = reference_capabilities();
    capabilities.net = NetworkCapabilities {
        egress: vec!["https://example.invalid".into()],
    };

    let pending = api
        .run(EmbeddedRunRequest {
            request: request(Uuid::new_v4(), capabilities),
            component: REFERENCE_COMPONENT.to_vec(),
            inputs: BTreeMap::new(),
        })
        .await
        .unwrap();

    assert_eq!(pending.state, RunState::GrantPending);
    assert!(pending.receipt.is_none());
    let events = api.events(pending.run_id, 0).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_id, "run.accepted");
    assert_eq!(events[1].event_id, "embedded.grant-pending");
}

#[test]
fn empty_trust_fails_before_creating_embedded_state() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("state");
    let result = EmbeddedExecutionApi::open(EmbeddedExecutionConfig {
        state_root: root.clone(),
        signing_key: SigningKey::from_bytes(&[117; 32]),
        component_pins: Vec::new(),
        artifact_budget_bytes: None,
    });
    assert!(result.is_err());
    assert!(!root.exists());
}
