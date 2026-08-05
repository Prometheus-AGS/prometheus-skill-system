use std::{collections::BTreeMap, path::PathBuf};

use chrono::Utc;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ExecutionLimits, ExecutionProvenance,
    RequestedTier, RuntimeKind, SignatureAlgorithm, SignedExecRequest,
};
use prometheus_exec_core::ExecutionJob;
use prometheus_exec_tier_p::{
    BwrapConfig, LandlockClassification, LandlockCompatibility, LandlockProbe,
    LandlockRulesetStatus, LinuxSandboxSelection,
};
use uuid::Uuid;

fn job() -> prometheus_exec_core::ValidatedExecutionJob {
    job_with_capabilities(CapabilityManifest::default())
}

fn job_with_capabilities(
    capabilities: CapabilityManifest,
) -> prometheus_exec_core::ValidatedExecutionJob {
    let code = b"print('sandboxed')\n";
    ExecutionJob {
        request: SignedExecRequest {
            schema_version: prometheus_exec_contracts::SCHEMA_VERSION.into(),
            request_id: Uuid::new_v4(),
            issued_at: Utc::now(),
            queued_at: None,
            validity_window_secs: 60,
            tier: RequestedTier::P,
            code: CodeIdentity {
                kind: CodeKind::Inline,
                hash: hash_bytes(code),
                runtime: RuntimeKind::Python3,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities,
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance::default(),
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        },
        code: code.to_vec(),
        inputs: BTreeMap::new(),
        grants: Vec::new(),
    }
    .validate()
    .unwrap()
}

fn config() -> BwrapConfig {
    BwrapConfig::new(
        "/usr/bin/bwrap",
        "bubblewrap 0.11.0",
        ["/bin", "/etc", "/lib", "/usr"]
            .into_iter()
            .map(PathBuf::from),
    )
    .unwrap()
}

fn plan() -> prometheus_exec_tier_p::BwrapPlan {
    config()
        .plan(
            &job(),
            PathBuf::from("/tmp/run-123").as_path(),
            PathBuf::from("/tmp/run-123/outputs").as_path(),
            PathBuf::from("/usr/bin/python3").as_path(),
            PathBuf::from("/tmp/run-123/program.py").as_path(),
            &BTreeMap::new(),
        )
        .unwrap()
}

#[test]
fn bwrap_plan_is_deterministic_and_network_isolated() {
    let first = plan();
    let second = plan();

    assert_eq!(first, second);
    assert_eq!(first.program(), PathBuf::from("/usr/bin/bwrap"));
    assert!(first.args().iter().any(|arg| arg == "--unshare-net"));
    assert!(!first.args().iter().any(|arg| arg == "--share-net"));
    assert_eq!(first.profile_hash().as_str().len(), "sha256:".len() + 64);
}

#[test]
fn writable_output_is_layered_after_read_only_run_root() {
    let plan = plan();
    let args = plan.args();
    let ro_index = args
        .windows(3)
        .position(|window| window == ["--ro-bind", "/tmp/run-123", "/work"])
        .unwrap();
    let rw_index = args
        .windows(3)
        .position(|window| window == ["--bind", "/tmp/run-123/outputs", "/work/outputs"])
        .unwrap();

    assert!(ro_index < rw_index);
    assert_eq!(
        &args[args.len() - 3..],
        ["--", "/usr/bin/python3", "/work/program.py"]
    );
}

#[test]
fn bwrap_plan_rejects_capability_broadening() {
    let mut environment = BTreeMap::new();
    environment.insert("SECRET".into(), "value".into());
    let error = config()
        .plan(
            &job(),
            PathBuf::from("/tmp/run-123").as_path(),
            PathBuf::from("/tmp/run-123/outputs").as_path(),
            PathBuf::from("/usr/bin/python3").as_path(),
            PathBuf::from("/tmp/run-123/program.py").as_path(),
            &environment,
        )
        .unwrap_err();
    assert!(error.to_string().contains("unexpected environment"));
}

#[test]
fn bwrap_plan_rejects_writable_code_or_escaped_layouts() {
    let writable_code = config()
        .plan(
            &job(),
            PathBuf::from("/tmp/run-123").as_path(),
            PathBuf::from("/tmp/run-123/outputs").as_path(),
            PathBuf::from("/usr/bin/python3").as_path(),
            PathBuf::from("/tmp/run-123/outputs/program.py").as_path(),
            &BTreeMap::new(),
        )
        .unwrap_err();
    assert!(writable_code.to_string().contains("writable output tree"));

    let escaped = config()
        .plan(
            &job(),
            PathBuf::from("/tmp/run-123").as_path(),
            PathBuf::from("/tmp/run-123/outputs").as_path(),
            PathBuf::from("/usr/bin/python3").as_path(),
            PathBuf::from("/tmp/elsewhere/program.py").as_path(),
            &BTreeMap::new(),
        )
        .unwrap_err();
    assert!(escaped.to_string().contains("beneath the run root"));
}

#[test]
fn landlock_partial_enforcement_is_explicit_and_not_attestable() {
    let partial = LandlockClassification::classify(&LandlockProbe {
        compatibility: LandlockCompatibility::BestEffort,
        ruleset: LandlockRulesetStatus::PartiallyEnforced,
        no_new_privs: true,
        effective_abi: Some(4),
        kernel_abi: None,
    });
    assert!(!partial.is_fully_enforced());
    assert!(matches!(
        partial,
        LandlockClassification::PartiallyEnforced { .. }
    ));

    let selected = LinuxSandboxSelection::select(None, partial);
    assert!(matches!(
        selected,
        LinuxSandboxSelection::TierUnavailable { .. }
    ));
}

#[test]
fn fully_enforced_landlock_is_classified_but_not_selected_without_certification() {
    let full = LandlockClassification::classify(&LandlockProbe {
        compatibility: LandlockCompatibility::BestEffort,
        ruleset: LandlockRulesetStatus::FullyEnforced,
        no_new_privs: true,
        effective_abi: Some(6),
        kernel_abi: Some(7),
    });
    assert!(full.is_fully_enforced());

    let selected = LinuxSandboxSelection::select(None, full);
    let LinuxSandboxSelection::TierUnavailable { reason, .. } = selected else {
        panic!("Landlock must not execute before Linux runtime certification")
    };
    assert!(reason.contains("not yet runtime-certified"));
}

#[test]
fn bwrap_rejects_requested_network_egress_before_command_construction() {
    let mut capabilities = CapabilityManifest::default();
    capabilities.net.egress = vec!["https://example.invalid".into()];

    let error = config()
        .plan(
            &job_with_capabilities(capabilities),
            PathBuf::from("/tmp/run-123").as_path(),
            PathBuf::from("/tmp/run-123/outputs").as_path(),
            PathBuf::from("/usr/bin/python3").as_path(),
            PathBuf::from("/tmp/run-123/program.py").as_path(),
            &BTreeMap::new(),
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("isolated network namespace only"));
}

#[cfg(not(target_os = "linux"))]
#[test]
fn linux_backend_detection_is_explicitly_unavailable_off_linux() {
    let error = BwrapConfig::detect().unwrap_err();
    assert!(error.to_string().contains("unavailable on this platform"));
}

#[test]
fn landlock_without_no_new_privs_is_partial_and_unavailable() {
    let partial = LandlockClassification::classify(&LandlockProbe {
        compatibility: LandlockCompatibility::BestEffort,
        ruleset: LandlockRulesetStatus::FullyEnforced,
        no_new_privs: false,
        effective_abi: Some(6),
        kernel_abi: None,
    });
    let LandlockClassification::PartiallyEnforced { warning, .. } = &partial else {
        panic!("missing no_new_privs must prevent full classification")
    };
    assert!(warning.contains("no_new_privs"));
    assert!(matches!(
        LinuxSandboxSelection::select(None, partial),
        LinuxSandboxSelection::TierUnavailable { .. }
    ));
}
