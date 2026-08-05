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

fn report() -> ExecutionCertificationReport {
    let evidence = [
        completed(
            "artifact-source",
            EvidenceDimension::ArtifactSource,
            "sourceHash",
            digest("ba438895404a23985d5226735b8f362cf3e8044894a1140852ba0992f2fdbe78"),
            digest("dec0135c83d49ef00321593fe765f8b170794fe39bdd782d3dea240f4b15d406"),
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
            digest("a6b6d50b5e090cc9ea2c95798fdb4bbb8779ac4961fb338da9c180a2bd04ce91"),
            digest("77222110eab6b97a3c20901ab79c7fd7dfce51598838dc73f6d3a51bca1d2cca"),
        ),
        pending(
            "judge-review",
            EvidenceDimension::JudgeReview,
            EvidenceStatus::PendingReview,
            "distinct-model review found remediable findings; remediation is in progress",
        ),
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
