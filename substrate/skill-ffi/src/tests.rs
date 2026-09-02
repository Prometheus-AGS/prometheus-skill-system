//! Round-trip tests across the FFI surface.
//!
//! These call the same functions the generated Dart bindings call and assert on
//! the VALUE returned, not merely that a call completed. A test that only
//! checked "it did not panic" would pass against a stub returning nothing.

use super::api::*;

use std::collections::BTreeMap;

use chrono::Utc;
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorization,
    ComponentAuthorizationMode, EnvironmentCapabilities, ExecutionLimits, ExecutionProvenance,
    FilesystemCapabilities, NetworkCapabilities, RequestedTier, RunState, RuntimeKind,
    SignatureAlgorithm, SignedExecRequest, SCHEMA_VERSION,
};
use prometheus_exec_core::{ArtifactBudgetProfile, ArtifactStore};
use prometheus_exec_embedded::{
    EmbeddedExecutionApi, EmbeddedExecutionConfig, EmbeddedRunResult, EmbeddedRunStatus,
};
use prometheus_exec_service::ExecutionService;
use tempfile::tempdir;
use uuid::Uuid;

const EXEC_REFERENCE_COMPONENT: &[u8] = include_bytes!(
    "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
);

fn exec_capabilities() -> CapabilityManifest {
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

fn exec_request(request_id: Uuid, capabilities: CapabilityManifest) -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc::now(),
        queued_at: None,
        validity_window_secs: 300,
        tier: RequestedTier::W,
        code: CodeIdentity {
            kind: CodeKind::Component,
            hash: hash_bytes(EXEC_REFERENCE_COMPONENT),
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

#[test]
fn run_skill_rejects_an_empty_id() {
    let e = run_skill(String::new(), "{}".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::InvalidInput));
    assert!(e.message.contains("skill_id"), "got: {}", e.message);
}

#[test]
fn run_skill_rejects_non_json_input() {
    let e = run_skill("entity-graph-optimize".into(), "not json".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::InvalidInput));
}

#[test]
fn run_skill_accepts_empty_input_like_the_shell_version() {
    // detect-orchestrators.sh ignores stdin entirely. Rejecting empty input
    // would be a behaviour change smuggled in under "port".
    let e = run_skill("entity-graph-optimize".into(), String::new()).unwrap_err();
    assert!(
        matches!(e.kind, SkillErrorKind::Unsupported),
        "empty input must reach the host, not be rejected as invalid"
    );
}

#[test]
fn run_skill_reports_no_host_rather_than_faking_success() {
    // THE assertion that matters. Until change-msp-008 de-stubs UAR's runtime,
    // the honest answer is Unsupported with a reason. Returning Ok here would
    // make a mobile caller believe a skill ran when nothing did.
    let e = run_skill("entity-graph-optimize".into(), "{}".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::Unsupported));
    assert!(
        e.message.contains("no host bound"),
        "the error must say WHY, got: {}",
        e.message
    );
}

#[test]
fn describe_round_trips_a_real_value() {
    let d = describe_skill("entity-graph-optimize".into()).unwrap();
    assert_eq!(d.id, "entity-graph-optimize");
    assert!(d.exports.contains(&"run".to_string()));
    assert!(d.exports.contains(&"describe".to_string()));
    assert!(d.capabilities.contains(&"kv-store".to_string()));
}

#[test]
fn describe_rejects_an_empty_id() {
    assert!(matches!(
        describe_skill("  ".into()).unwrap_err().kind,
        SkillErrorKind::InvalidInput
    ));
}

#[test]
fn world_version_matches_the_wit_package() {
    // A binding built against a different world than the host expects is a
    // silent mismatch; this is the value a client checks first.
    assert_eq!(world_version(), "prometheus:component@0.1.0");
}

#[test]
fn list_skills_reports_no_host_rather_than_an_empty_catalog() {
    // An empty Ok(vec![]) would read as "no skills exist" when the truth is
    // "nothing can answer yet" — the same lie-by-omission `run_skill` avoids.
    let e = list_skills().unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::Unsupported));
    assert!(e.message.contains("no host bound"), "got: {}", e.message);
}

const INDEX_FIXTURE: &str = r#"{
  "schemaVersion":"prometheus-skill-index-v1",
  "entries":[
    {"id":"api-design","description":"REST contracts","relativePath":"backend/api-design","searchText":"api-design rest contracts"},
    {"id":"rust-review","description":"Rust correctness","relativePath":"rust/rust-review","searchText":"rust-review rust correctness"}
  ],
  "sha256":"08cd6d1575faa7966b5aef855f3091a0e4e1a53d8f5fba2806519934a16a9888"
}"#;

#[test]
fn mobile_catalog_consumes_the_generation_index() {
    let listed = list_indexed_skills(INDEX_FIXTURE.into()).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].id, "api-design");
}

#[test]
fn mobile_search_uses_shared_deterministic_selection() {
    let selected =
        search_indexed_skills(INDEX_FIXTURE.into(), "rust correctness".into(), 1).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, "rust-review");
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_ffi_returns_values_preserves_interruptions_and_never_accepts_private_keys() {
    let directory = tempdir().unwrap();
    let state_root = directory.path().join("exec");
    let interrupted_request = exec_request(Uuid::new_v4(), exec_capabilities());

    // Reproduce a process loss after the durable spawn boundary and before a
    // receipt exists. Opening the embedded API must sign interruption evidence
    // without running the component a second time.
    let artifacts = ArtifactStore::open_for_profile(
        state_root.join("artifacts"),
        ArtifactBudgetProfile::Desktop,
        Some(64 * 1024 * 1024),
    )
    .unwrap();
    artifacts.put(EXEC_REFERENCE_COMPONENT).unwrap();
    artifacts.retain_for_request(&interrupted_request).unwrap();
    let service = ExecutionService::open(state_root.join("service")).unwrap();
    let accepted = service.submit(interrupted_request.clone()).unwrap();
    service
        .claim_for_execution(
            interrupted_request.request_id,
            &accepted.record.request_hash,
        )
        .unwrap();
    let interrupted_run_id = accepted.record.run_id;
    drop(service);

    let private_key_bytes = [119_u8; 32];
    let api = EmbeddedExecutionApi::open(EmbeddedExecutionConfig {
        state_root,
        signing_key: SigningKey::from_bytes(&private_key_bytes),
        component_pins: vec![hash_bytes(EXEC_REFERENCE_COMPONENT)],
        artifact_budget_bytes: Some(64 * 1024 * 1024),
    })
    .unwrap();
    let duplicate = api.clone();
    crate::exec_host::install_execution_api(api).unwrap();
    assert!(crate::exec_host::install_execution_api(duplicate).is_err());
    assert!(matches!(
        exec_status("not-a-uuid".into()).await.unwrap_err().kind,
        SkillErrorKind::InvalidInput
    ));

    let interrupted: EmbeddedRunStatus =
        serde_json::from_str(&exec_status(interrupted_run_id.to_string()).await.unwrap()).unwrap();
    assert_eq!(interrupted.state, RunState::Interrupted);
    let interrupted_receipt = interrupted.receipt.unwrap();
    assert_eq!(
        interrupted_receipt.failure.as_ref().unwrap().code,
        "embedded.restart-interrupted"
    );
    assert_eq!(
        exec_artifact(interrupted_receipt.outputs.stdout.to_string())
            .await
            .unwrap(),
        Vec::<u8>::new()
    );
    let loaded_interruption: prometheus_exec_contracts::ExecutionReceipt =
        serde_json::from_str(&exec_receipt(interrupted_run_id.to_string()).await.unwrap()).unwrap();
    assert_eq!(loaded_interruption, interrupted_receipt);

    let request = exec_request(Uuid::new_v4(), exec_capabilities());
    let result: EmbeddedRunResult = serde_json::from_str(
        &exec_run(
            serde_json::to_string(&request).unwrap(),
            EXEC_REFERENCE_COMPONENT.to_vec(),
            BTreeMap::new(),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result.state, RunState::Succeeded);
    let receipt = result.receipt.unwrap();
    let loaded_receipt: prometheus_exec_contracts::ExecutionReceipt =
        serde_json::from_str(&exec_receipt(result.run_id.to_string()).await.unwrap()).unwrap();
    assert_eq!(loaded_receipt, receipt);
    let returned = exec_artifact(receipt.outputs.stdout.to_string())
        .await
        .unwrap();
    assert_eq!(
        String::from_utf8(returned).unwrap(),
        r#"{"kbd":{"available":false},"evolver":{"available":false},"refiner":{"available":false},"openspec":{"available":false}}"#
    );

    let events: Vec<prometheus_exec_service::RunEvent> =
        serde_json::from_str(&exec_events(result.run_id.to_string(), 0).await.unwrap()).unwrap();
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let public_key_json = exec_public_key().unwrap();
    let verified: prometheus_exec_contracts::VerificationResult = serde_json::from_str(
        &exec_verify(
            serde_json::to_string(&receipt).unwrap(),
            Some(serde_json::to_string(&request).unwrap()),
            public_key_json.clone(),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    assert!(verified.valid, "{:?}", verified.failures);

    let mut privileged = exec_capabilities();
    privileged.net.egress.push("https://example.invalid".into());
    let pending: EmbeddedRunResult = serde_json::from_str(
        &exec_run(
            serde_json::to_string(&exec_request(Uuid::new_v4(), privileged)).unwrap(),
            EXEC_REFERENCE_COMPONENT.to_vec(),
            BTreeMap::new(),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    assert_eq!(pending.state, RunState::GrantPending);
    assert!(pending.receipt.is_none());
    let pending_events: Vec<prometheus_exec_service::RunEvent> =
        serde_json::from_str(&exec_events(pending.run_id.to_string(), 0).await.unwrap()).unwrap();
    assert_eq!(pending_events.len(), 2);
    assert_eq!(pending_events[1].event_id, "embedded.grant-pending");

    let bridge_source = include_str!("api.rs");
    let exec_surface = bridge_source
        .split("pub async fn exec_run")
        .nth(1)
        .expect("exec FFI surface exists");
    assert!(!exec_surface.contains("SigningKey"));
    assert!(!exec_surface.contains("signing_key"));
    assert!(!public_key_json.contains("private"));
    assert!(!public_key_json.contains("secret"));
    assert!(!public_key_json.contains(&format!("{:?}", private_key_bytes)));
}
