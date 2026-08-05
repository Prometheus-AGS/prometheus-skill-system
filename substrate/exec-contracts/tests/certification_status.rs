use std::collections::BTreeMap;

use prometheus_exec_contracts::{
    CertificationEvidence, Digest, EvidenceDimension, EvidenceStatus, ExecutionCertificationReport,
    SCHEMA_VERSION,
};

fn digest(value: &str) -> Digest {
    Digest::parse(format!("sha256:{value}")).unwrap()
}

fn completed(
    requirement_id: &str,
    dimension: EvidenceDimension,
    property: &str,
    property_hash: Digest,
    bundle_hash: Digest,
) -> CertificationEvidence {
    CertificationEvidence {
        requirement_id: requirement_id.into(),
        dimension,
        status: EvidenceStatus::Completed,
        environment: "macos-x86_64-disposable".into(),
        evidence_properties: BTreeMap::from([(property.into(), property_hash)]),
        bundle_index_hash: Some(bundle_hash),
        disposition: None,
        producer_method: Some("prometheus-exec".into()),
    }
}

fn pending(
    requirement_id: &str,
    dimension: EvidenceDimension,
    status: EvidenceStatus,
    disposition: &str,
) -> CertificationEvidence {
    CertificationEvidence {
        requirement_id: requirement_id.into(),
        dimension,
        status,
        environment: "release-1.7.0".into(),
        evidence_properties: BTreeMap::new(),
        bundle_index_hash: None,
        disposition: Some(disposition.into()),
        producer_method: None,
    }
}

fn completed_review() -> CertificationEvidence {
    let mut evidence = completed(
        "judge-review",
        EvidenceDimension::JudgeReview,
        "findingsHash",
        digest("44e679387a7073027c2ef2fba38af2297c36feba261c30b640af4cdf8d552d9c"),
        digest("40bd53a2199cbecd180e6733bd6b89cb2721325b56210ca7e8c5fa09e375c9b2"),
    );
    evidence.environment = "release-1.7.0".into();
    evidence.producer_method = Some("MiniMax-M3 via isolated local REST gateway".into());
    evidence
}

fn report() -> ExecutionCertificationReport {
    let evidence = [
        completed(
            "artifact-source",
            EvidenceDimension::ArtifactSource,
            "sourceHash",
            digest("ba438895404a23985d5226735b8f362cf3e8044894a1140852ba0992f2fdbe78"),
            digest("34f1a025e17ae570579c222cbe0d281ab52845ec34b7a0f7c04e216699129008"),
        ),
        completed(
            "disposable-runtime",
            EvidenceDimension::DisposableRuntime,
            "receiptHash",
            digest("6e8d0aef1abe3ff9ff52d74f8a5d83ea8e72b22e70e663949ee8b7df3bea5818"),
            digest("70b9ae775a672b79d8d034a1c5f391d9203d1fc077e46fb957fbd89e2e291a6c"),
        ),
        completed(
            "installed-host",
            EvidenceDimension::InstalledHost,
            "binaryHash",
            digest("d4fcd01e7a0c4cbb2a6dc9657557c5a0aa3dbd54a21845deefe6a52bc68bcd1c"),
            digest("6bac8940f22d87dc173ff9c4d8cf9e45b8e443fd670181dfd45fa7d316850f4c"),
        ),
        completed_review(),
        pending(
            "mobile-size",
            EvidenceDimension::MobileSize,
            EvidenceStatus::Blocked,
            "measured iOS and Android deltas exceed the 12 MiB gate",
        ),
        pending(
            "physical-device",
            EvidenceDimension::PhysicalDevice,
            EvidenceStatus::PendingEvidence,
            "no usable physical iOS or Android device is connected",
        ),
        pending(
            "remote-deployment",
            EvidenceDimension::RemoteDeployment,
            EvidenceStatus::PendingEvidence,
            "protocol kernel is disposable-runtime certified; production transport is not deployed",
        ),
    ];
    ExecutionCertificationReport {
        schema_version: SCHEMA_VERSION.into(),
        release: "1.7.0".into(),
        requirements: evidence
            .into_iter()
            .map(|entry| (entry.requirement_id.clone(), entry))
            .collect(),
    }
}

#[test]
fn status_dimensions_remain_separate_and_deterministic() {
    let report = report();
    report.validate().unwrap();
    let rendered = serde_json::to_string_pretty(&report).unwrap() + "\n";
    assert_eq!(
        rendered,
        include_str!("../../../docs/reference/api/prometheus-exec.evidence-status.json")
    );
}

#[test]
fn equivalent_properties_do_not_depend_on_producer_method() {
    let original = report().requirements.remove("artifact-source").unwrap();
    let mut external = original.clone();
    external.producer_method = Some("external-signed-builder".into());
    assert!(original.equivalent_evidence(&external));
}

#[test]
fn judge_and_environment_unavailability_cannot_be_collapsed() {
    let remote_as_review = pending(
        "remote",
        EvidenceDimension::RemoteDeployment,
        EvidenceStatus::PendingReview,
        "offline",
    );
    assert!(remote_as_review.validate().is_err());
    let judge_as_evidence = pending(
        "judge",
        EvidenceDimension::JudgeReview,
        EvidenceStatus::PendingEvidence,
        "offline",
    );
    assert!(judge_as_evidence.validate().is_err());
}
