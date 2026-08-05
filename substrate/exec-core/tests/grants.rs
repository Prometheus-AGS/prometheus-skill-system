use std::{fs, process::Command};

use chrono::{Duration, TimeZone as _, Utc};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ExecutionLimits, ExecutionProvenance,
    GrantKind, RequestedTier, RuntimeKind, SignatureAlgorithm, SignedExecRequest, SCHEMA_VERSION,
};
use prometheus_exec_core::{
    verify_interactive_grant, GrantValidationError, InteractiveGrantIssuer, SshGrantManifest,
    SshGrantVerifier, GRANT_NAMESPACE,
};
use tempfile::tempdir;
use uuid::Uuid;

fn privileged_request() -> SignedExecRequest {
    let mut capabilities = CapabilityManifest::default();
    capabilities.net.egress = vec!["api.example.com:443".into()];
    SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::parse_str("f98e4f98-3453-470e-b1b8-82869da2f0db").unwrap(),
        issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 15, 0, 0).unwrap(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print('grant')"),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![],
        capabilities,
        limits: ExecutionLimits::default(),
        targets: vec![],
        provenance: ExecutionProvenance::default(),
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    }
}

#[test]
fn real_ssh_signature_verifies_with_namespace_and_allowed_signer() {
    let directory = tempdir().unwrap();
    let key_path = directory.path().join("approver");
    let identity = "operator@example.com";
    let status = Command::new("/usr/bin/ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .unwrap();
    assert!(status.success());
    let public = fs::read_to_string(key_path.with_extension("pub")).unwrap();
    let fields: Vec<_> = public.split_whitespace().take(2).collect();
    let allowed_signers = directory.path().join("allowed_signers");
    fs::write(
        &allowed_signers,
        format!("{identity} {} {}\n", fields[0], fields[1]),
    )
    .unwrap();

    let request = privileged_request();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 15, 1, 0).unwrap();
    let manifest = SshGrantManifest::for_request(
        &request,
        Uuid::new_v4(),
        now,
        now + Duration::minutes(30),
        identity,
        "approved network call",
    )
    .unwrap();
    let manifest_path = directory.path().join("grant.json");
    fs::write(&manifest_path, manifest.canonical_bytes().unwrap()).unwrap();
    let status = Command::new("/usr/bin/ssh-keygen")
        .args(["-Y", "sign", "-f"])
        .arg(&key_path)
        .args(["-n", GRANT_NAMESPACE])
        .arg(&manifest_path)
        .status()
        .unwrap();
    assert!(status.success());
    let signature = fs::read(manifest_path.with_extension("json.sig")).unwrap();

    let verified = SshGrantVerifier::new("/usr/bin/ssh-keygen", allowed_signers)
        .verify(&request, &manifest, &signature, now)
        .unwrap();
    assert_eq!(verified.grant.kind, GrantKind::SshManifest);
    assert_eq!(
        verified.grant.r#ref,
        Some(manifest.canonical_hash().unwrap())
    );

    let mut wrong_request = request;
    wrong_request.request_id = Uuid::new_v4();
    assert!(matches!(
        SshGrantVerifier::new(
            "/usr/bin/ssh-keygen",
            directory.path().join("allowed_signers")
        )
        .verify(&wrong_request, &manifest, &signature, now),
        Err(GrantValidationError::RequestMismatch)
    ));
}

#[test]
fn wrong_ssh_namespace_is_rejected() {
    let directory = tempdir().unwrap();
    let key_path = directory.path().join("approver");
    assert!(Command::new("/usr/bin/ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .unwrap()
        .success());
    let public = fs::read_to_string(key_path.with_extension("pub")).unwrap();
    let fields: Vec<_> = public.split_whitespace().take(2).collect();
    let allowed_signers = directory.path().join("allowed_signers");
    fs::write(
        &allowed_signers,
        format!("operator {} {}\n", fields[0], fields[1]),
    )
    .unwrap();
    let request = privileged_request();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 15, 1, 0).unwrap();
    let manifest = SshGrantManifest::for_request(
        &request,
        Uuid::new_v4(),
        now,
        now + Duration::minutes(10),
        "operator",
        "wrong namespace fixture",
    )
    .unwrap();
    let path = directory.path().join("grant.json");
    fs::write(&path, manifest.canonical_bytes().unwrap()).unwrap();
    assert!(Command::new("/usr/bin/ssh-keygen")
        .args(["-Y", "sign", "-f"])
        .arg(&key_path)
        .args(["-n", "wrong-namespace"])
        .arg(&path)
        .status()
        .unwrap()
        .success());
    let signature = fs::read(path.with_extension("json.sig")).unwrap();
    assert!(matches!(
        SshGrantVerifier::new("/usr/bin/ssh-keygen", allowed_signers)
            .verify(&request, &manifest, &signature, now),
        Err(GrantValidationError::SshRejected(_))
    ));
}

#[test]
fn interactive_grant_is_device_signed_bound_and_expiring() {
    let request = privileged_request();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 15, 1, 0).unwrap();
    let issuer = InteractiveGrantIssuer::new(SigningKey::from_bytes(&[21_u8; 32]), "tauri-host");
    let public = issuer.public_key();
    let token = issuer
        .issue(
            &request,
            now,
            now + Duration::minutes(10),
            "local user",
            "approved in trusted dialog",
        )
        .unwrap();
    let verified = verify_interactive_grant(&request, &token, &public, "tauri-host", now).unwrap();
    assert_eq!(verified.grant.kind, GrantKind::Interactive);
    assert!(verified.grant.r#ref.is_some());

    let mut mutated = token.clone();
    mutated.statement.reason = "silently changed".into();
    assert!(matches!(
        verify_interactive_grant(&request, &mutated, &public, "tauri-host", now),
        Err(GrantValidationError::InteractiveSignatureRejected)
    ));
    assert!(matches!(
        verify_interactive_grant(
            &request,
            &token,
            &public,
            "tauri-host",
            now + Duration::minutes(11)
        ),
        Err(GrantValidationError::Expired)
    ));
}
