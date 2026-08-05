use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{hash_serializable, Digest, SCHEMA_VERSION};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Completed,
    Failed,
    Blocked,
    PendingEvidence,
    PendingReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDimension {
    ArtifactSource,
    DisposableRuntime,
    InstalledHost,
    RemoteDeployment,
    MobileSize,
    PhysicalDevice,
    JudgeReview,
}

/// One requirement's evidence classification. `producer_method` is
/// informational and deliberately excluded from the evidence projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CertificationEvidence {
    pub requirement_id: String,
    pub dimension: EvidenceDimension,
    pub status: EvidenceStatus,
    pub environment: String,
    #[serde(default)]
    pub evidence_properties: BTreeMap<String, Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_index_hash: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_method: Option<String>,
}

impl CertificationEvidence {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.requirement_id.trim().is_empty() || self.environment.trim().is_empty() {
            return Err("requirement ID and environment must be present".into());
        }
        match self.status {
            EvidenceStatus::Completed => {
                if self.bundle_index_hash.is_none() || self.evidence_properties.is_empty() {
                    return Err(
                        "completed evidence requires a bundle hash and verified properties".into(),
                    );
                }
            }
            EvidenceStatus::PendingEvidence => {
                if self.dimension == EvidenceDimension::JudgeReview {
                    return Err("judge unavailability must use pending_review".into());
                }
                require_disposition_without_bundle(self)?;
            }
            EvidenceStatus::PendingReview => {
                if self.dimension != EvidenceDimension::JudgeReview {
                    return Err("pending_review is reserved for judge review".into());
                }
                require_disposition_without_bundle(self)?;
            }
            EvidenceStatus::Failed | EvidenceStatus::Blocked => {
                if self.disposition.as_deref().is_none_or(str::is_empty) {
                    return Err("failed or blocked evidence requires a disposition".into());
                }
            }
        }
        Ok(())
    }

    /// Hash verifiable properties while excluding the tool, language, shell,
    /// or producer that created them.
    pub fn method_independent_hash(&self) -> std::result::Result<Digest, String> {
        self.validate()?;
        hash_serializable(&(
            &self.requirement_id,
            self.dimension,
            self.status,
            &self.environment,
            &self.evidence_properties,
            &self.bundle_index_hash,
            &self.disposition,
        ))
        .map_err(|error| error.to_string())
    }

    pub fn equivalent_evidence(&self, other: &Self) -> bool {
        match (
            self.method_independent_hash(),
            other.method_independent_hash(),
        ) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCertificationReport {
    pub schema_version: String,
    pub release: String,
    pub requirements: BTreeMap<String, CertificationEvidence>,
}

impl ExecutionCertificationReport {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.schema_version != SCHEMA_VERSION || self.release.trim().is_empty() {
            return Err("certification schema and release must be present and supported".into());
        }
        if self.requirements.is_empty() {
            return Err("certification report requires at least one requirement".into());
        }
        for (requirement_id, evidence) in &self.requirements {
            if requirement_id != &evidence.requirement_id {
                return Err(format!(
                    "requirement map key {requirement_id} does not match {}",
                    evidence.requirement_id
                ));
            }
            evidence.validate()?;
        }
        Ok(())
    }
}

fn require_disposition_without_bundle(
    evidence: &CertificationEvidence,
) -> std::result::Result<(), String> {
    if evidence.disposition.as_deref().is_none_or(str::is_empty) {
        return Err("pending status requires an explicit disposition".into());
    }
    if evidence.bundle_index_hash.is_some() || !evidence.evidence_properties.is_empty() {
        return Err("pending status cannot carry completed-evidence claims".into());
    }
    Ok(())
}
