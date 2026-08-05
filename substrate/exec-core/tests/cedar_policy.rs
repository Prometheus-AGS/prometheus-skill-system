use chrono::{TimeZone as _, Utc};
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ExecutionLimits, ExecutionProvenance,
    RequestedTier, RuntimeKind, SignatureAlgorithm, SignedExecRequest, SCHEMA_VERSION,
};
use prometheus_exec_core::{CedarTighteningPolicy, PolicyEvaluator, PolicyOutcome, PolicyReason};
use uuid::Uuid;

fn safe_request() -> SignedExecRequest {
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::parse_str("d50d433c-bc87-43af-8ac8-f3a5811afe37").unwrap(),
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 14, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print('policy')"),
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

#[test]
fn embedded_policy_auto_approves_only_safe_baseline_request() {
    let policy = CedarTighteningPolicy::embedded_default();
    assert_eq!(
        policy.evaluate(&safe_request()),
        PolicyOutcome::AutoApproved
    );
    assert!(policy.policy_hash().as_str().starts_with("sha256:"));
}

#[test]
fn permit_all_cedar_cannot_broaden_the_hard_ceiling() {
    let policy =
        CedarTighteningPolicy::from_policy_text(r#"permit(principal, action, resource);"#).unwrap();
    let mut request = safe_request();
    request.capabilities.net.egress = vec!["example.com:443".into()];
    request.capabilities.env.read = vec!["SECRET".into()];
    request
        .capabilities
        .fs
        .read_write
        .push("outputs/../outside".into());
    request.capabilities.fs.read_only.push("C:\\Secrets".into());

    assert_eq!(
        policy.evaluate(&request),
        PolicyOutcome::GrantRequired {
            reasons: vec![
                PolicyReason::NetworkEgress,
                PolicyReason::EnvironmentPassthrough,
                PolicyReason::ExternalWritePath("outputs/../outside".into()),
                PolicyReason::ExternalReadPath("C:\\Secrets".into()),
            ]
        }
    );
}

#[test]
fn operator_policy_can_tighten_safe_request_to_grant_required() {
    let policy = CedarTighteningPolicy::from_policy_text(
        r#"
permit(principal, action == Action::"exec.autoApprove", resource)
when { context.memoryMb <= 128 };
"#,
    )
    .unwrap();
    let outcome = policy.evaluate(&safe_request());
    assert!(matches!(outcome, PolicyOutcome::GrantRequired { .. }));
    assert_eq!(outcome, policy.evaluate(&safe_request()));
}

#[test]
fn invalid_operator_policy_fails_at_load_time() {
    assert!(CedarTighteningPolicy::from_policy_text("permit(").is_err());
}
