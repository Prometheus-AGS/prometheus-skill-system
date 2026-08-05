use std::collections::BTreeMap;

use prometheus_exec_contracts::{
    CertificationEvidence, Digest, EvidenceDimension, EvidenceStatus, ExecutionCertificationReport,
    SCHEMA_VERSION,
};

fn digest(character: char) -> Digest {
    Digest::parse(format!("sha256:{}", character.to_string().repeat(64))).unwrap()
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
            digest('1'),
            digest('a'),
        ),
        completed(
            "disposable-runtime",
            EvidenceDimension::DisposableRuntime,
            "receiptHash",
            digest('2'),
            digest('b'),
        ),
        completed(
            "installed-host",
            EvidenceDimension::InstalledHost,
            "binaryHash",
            digest('3'),
            digest('c'),
        ),
        pending(
            "judge-review",
            EvidenceDimension::JudgeReview,
            EvidenceStatus::PendingReview,
            "distinct judge is temporarily unavailable",
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
