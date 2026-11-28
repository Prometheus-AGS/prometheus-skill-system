//! Read-only evidence provenance. Reuse the existing Exec verifier; never
//! dispatch, enroll, sign, grant, or infer a KBD product verdict. The expected
//! attempt and qualified identities must come from a future verified
//! canonical producer. No external constructor or operational caller exists.

use super::{validate_denominator, ArtifactCheckKind, ArtifactExpectation, ObligationKey, Obligations};
use super::super::refused;
use crate::{phase_closure::is_digest, Result};
use prometheus_exec_contracts::{
    verify_receipt, verify_request_signature, Digest, EvidenceClass, ExecutionBackend,
    ExecutionReceipt, ExecutionTier, RequestedTier, RuntimeKind, SignedExecRequest, VerificationKey,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use super::history::AuthenticatedTransportEvent;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AttemptAttribution {
    project_id: String,
    run_id: String,
    phase_id: String,
    contract_digest: String,
    binding_revision: u32,
    scenario_id: String,
    scenario_digest: String,
    target_id: String,
    target_identity_sha256: String,
    requirements_sha256: String,
    candidate_sha256: String,
    attempt_id: String,
    // Exact canonical reserved scope identity, common to all physical checks
    // belonging to that finite attempt.
    attempt_scope_sha256: String,
    check_kind: ArtifactCheckKind,
    requirement_id: String,
}

/// Identity and exact request already bound to a canonical reserved attempt,
/// with requester and runner keys from a qualified enrolled profile. Never
/// deserialize a caller's expectation as this trusted type.
struct ExpectedExecAttempt {
    attribution: AttemptAttribution,
    request_hash: Digest,
    exec_run_id: Uuid,
    requester_key: VerificationKey,
    runner_key: VerificationKey,
    input_set_hash: Digest,
    env_hash: Digest,
    sandbox_profile_hash: Digest,
    toolchain_hash: Option<Digest>,
    // Enforced by the qualified runner and immutable receipt archive producer;
    // the verifier does not create or replenish any resource allowances.
    artifact_byte_ceiling: u64,
    artifact_count_ceiling: usize,
}

/// One comparison from the exact verified receipt. No process exit code or
/// receipt-controlled verdict can substitute for the parent expected bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactComparison {
    kind: ArtifactCheckKind,
    requirement_id: String,
    path: String,
    expected_sha256: String,
    observed_digest: Option<Digest>,
    matches_expected: bool,
}

/// Candidate payload for the existing signed canonical event ledger, NOT
/// a new ledger or an admission input. Deserializing a payload is not proof.
/// No PASS, FAIL, setup classification or closure status is stored here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TransportRecord {
    schema: String,
    attribution: AttemptAttribution,
    request: SignedExecRequest,
    receipt: ExecutionReceipt,
    receipt_hash: Digest,
    comparisons: Vec<ArtifactComparison>,
}

/// Trusted only after this module's actual read-only verification or replay of
/// an already authenticated canonical event. No wire deserialization.
pub(super) struct VerifiedTransport {
    record: TransportRecord,
}

/// The future canonical archive producer must enforce immutable, confined
/// byte storage before supplying this private type. No path-only constructor
/// exists here; a mutable arbitrary directory is not a verified archive.
struct ImmutableEvidenceArchive {
    root: PathBuf,
}

/// Replay facts stay bound to the actual authenticated event, not a raw
/// record or a separately claimed event position.
pub(super) struct ReplayVerifiedTransport<'a> {
    event: &'a AuthenticatedTransportEvent,
}
impl<'a> ReplayVerifiedTransport<'a> {
    pub(super) fn event(&self) -> &'a AuthenticatedTransportEvent { self.event }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TransportItemKey {
    attempt_id: String,
    scenario_id: String,
    target_id: String,
    kind: ArtifactCheckKind,
    requirement_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AttemptEnvelope {
    project_id: String,
    run_id: String,
    phase_id: String,
    contract_digest: String,
    binding_revision: u32,
    candidate_sha256: String,
    attempt_scope_sha256: String,
}

impl VerifiedTransport {
    pub(super) fn into_record(self) -> TransportRecord { self.record }
}

impl TransportRecord {
    pub(super) fn attempt_id(&self) -> &str { &self.attribution.attempt_id }
    pub(super) fn project_id(&self) -> &str { &self.attribution.project_id }
    pub(super) fn run_id(&self) -> &str { &self.attribution.run_id }
    pub(super) fn phase_id(&self) -> &str { &self.attribution.phase_id }
    pub(super) fn receipt_hash(&self) -> &Digest { &self.receipt_hash }
    pub(super) fn request_hash(&self) -> &Digest { &self.receipt.request_hash }
    pub(super) fn item_key(&self) -> TransportItemKey {
        let a = &self.attribution;
        TransportItemKey { attempt_id: a.attempt_id.clone(), scenario_id: a.scenario_id.clone(),
            target_id: a.target_id.clone(), kind: a.check_kind, requirement_id: a.requirement_id.clone() }
    }
    pub(super) fn attempt_envelope(&self) -> AttemptEnvelope {
        let a = &self.attribution;
        AttemptEnvelope { project_id: a.project_id.clone(), run_id: a.run_id.clone(),
            phase_id: a.phase_id.clone(), contract_digest: a.contract_digest.clone(),
            binding_revision: a.binding_revision, candidate_sha256: a.candidate_sha256.clone(),
            attempt_scope_sha256: a.attempt_scope_sha256.clone() }
    }
}

fn requirements<'a>(
    parent: &Obligations<'a>, attempt: &ExpectedExecAttempt,
) -> Result<Vec<&'a ArtifactExpectation>> {
    let a = &attempt.attribution;
    let reject = |reason: &str| refused(parent.phase_id, format!("transport-provenance:{reason}"));
    if a.project_id != parent.project_id || a.run_id != parent.run_id || a.phase_id != parent.phase_id
        || a.contract_digest != parent.binding.contract_digest
        || a.binding_revision != parent.binding.binding_revision
        || !is_digest(&a.candidate_sha256)
        || !is_digest(&a.attempt_scope_sha256)
        || !super::name_is_valid(&a.attempt_id)
    {
        return Err(reject("parent-or-attempt-mismatch"));
    }
    let required = validate_denominator(parent)?;
    let key = ObligationKey { scenario_id: a.scenario_id.clone(), target_id: a.target_id.clone() };
    let row = required.get(&key).copied().ok_or_else(|| reject("unknown-obligation"))?;
    if row.scenario_digest != a.scenario_digest
        || row.target_identity_sha256 != a.target_identity_sha256
        || row.requirements_sha256 != a.requirements_sha256
    {
        return Err(reject("obligation-identity-mismatch"));
    }
    let checks: Vec<_> = row.artifact_checks.iter()
        .filter(|check| check.kind == a.check_kind && check.requirement_id == a.requirement_id)
        .collect();
    if checks.is_empty() { return Err(reject("no-parent-artifact-requirement")); }
    Ok(checks)
}

fn verify_identities(
    phase_id: &str, attempt: &ExpectedExecAttempt, request: &SignedExecRequest,
    receipt: &ExecutionReceipt, artifact_root: Option<&Path>,
) -> Result<Digest> {
    let reject = |reason: &str| refused(phase_id, format!("transport-provenance:{reason}"));
    request.validate().map_err(|_| reject("request-invariants"))?;
    verify_request_signature(request, &attempt.requester_key)
        .map_err(|_| reject("request-signature"))?;
    let request_hash = request.request_hash().map_err(|_| reject("request-hash"))?;
    // The only proposed staged profile is local macOS arm64 Python/Seatbelt.
    // Qualification is a separate mandatory precondition, not inferred by
    // these field comparisons, and no profile is enabled by this source.
    if request_hash != attempt.request_hash
        || request.tier != RequestedTier::P || request.code.runtime != RuntimeKind::Python3
        || receipt.run_id != attempt.exec_run_id || receipt.request_hash != request_hash
        || receipt.tier != ExecutionTier::P || receipt.evidence_class != EvidenceClass::Attested
        || receipt.backend != ExecutionBackend::Seatbelt
        || receipt.executing_device.platform != "macos-aarch64"
        || receipt.code_hash != request.code.hash
        || receipt.input_set_hash != attempt.input_set_hash
        || receipt.env_hash != attempt.env_hash
        || receipt.sandbox_profile_hash != attempt.sandbox_profile_hash
        || receipt.toolchain_hash != attempt.toolchain_hash
    {
        return Err(reject("exact-execution-identity-mismatch"));
    }
    // Signed historical time consistency, not a substitute for the
    // independently qualified use-boundary expiry/revocation enforcement.
    let window = i64::try_from(request.validity_window_secs).ok()
        .and_then(chrono::TimeDelta::try_seconds)
        .and_then(|delta| request.issued_at.checked_add_signed(delta))
        .ok_or_else(|| reject("invalid-request-window"))?;
    if receipt.started_at < request.issued_at || receipt.started_at > window {
        return Err(reject("receipt-outside-request-window"));
    }
    if let Some(root) = artifact_root {
        // Only the future immutable, private evidence archive is eligible.
        // Bound sizes before the established verifier reads artifact bytes.
        if attempt.artifact_byte_ceiling == 0 || attempt.artifact_count_ceiling == 0
            || receipt.outputs.artifacts.len() > attempt.artifact_count_ceiling {
            return Err(reject("artifact-bound"));
        }
        let canonical_root = root.canonicalize().map_err(|_| reject("artifact-root"))?;
        let mut bytes = 0u64;
        for artifact in &receipt.outputs.artifacts {
            prometheus_exec_contracts::validate_artifact_path(&artifact.path)
                .map_err(|_| reject("artifact-path"))?;
            let path = root.join(&artifact.path).canonicalize().map_err(|_| reject("artifact-path"))?;
            if !path.starts_with(&canonical_root) {
                return Err(reject("artifact-outside-root"));
            }
            let metadata = path.metadata().map_err(|_| reject("artifact-metadata"))?;
            if !metadata.is_file() || artifact.size_bytes.is_some_and(|size| size != metadata.len()) {
                return Err(reject("artifact-type-or-size"));
            }
            bytes = bytes.checked_add(metadata.len()).ok_or_else(|| reject("artifact-bound"))?;
            if bytes > attempt.artifact_byte_ceiling { return Err(reject("artifact-bound")); }
        }
    }
    let verification = verify_receipt(receipt, &attempt.runner_key, Some(request), artifact_root);
    if !verification.valid { return Err(reject("receipt-request-or-artifact-verification")); }
    verification.receipt_hash.ok_or_else(|| reject("missing-receipt-hash"))
}

fn compare_artifacts(
    expected: &[&ArtifactExpectation], receipt: &ExecutionReceipt,
) -> Vec<ArtifactComparison> {
    expected.iter().map(|check| {
        let observed_digest = receipt.outputs.artifacts.iter()
            .find(|artifact| artifact.path == check.artifact_path)
            .map(|artifact| artifact.hash.clone());
        let matches_expected = observed_digest.as_ref().is_some_and(|digest|
            digest.as_str().strip_prefix("sha256:") == Some(check.expected_sha256.as_str()));
        ArtifactComparison {
            kind: check.kind,
            requirement_id: check.requirement_id.clone(),
            path: check.artifact_path.clone(),
            expected_sha256: check.expected_sha256.clone(),
            observed_digest,
            matches_expected,
        }
    }).collect()
}

/// Actual read-only host verification: no optional artifact root, no signer
/// or public key taken from the supplied receipt. Missing artifacts never
/// prove a product outcome, even when the signed process succeeded.
fn verify_transport(
    parent: &Obligations<'_>, attempt: &ExpectedExecAttempt,
    request: &SignedExecRequest, receipt: &ExecutionReceipt, archive: &ImmutableEvidenceArchive,
) -> Result<VerifiedTransport> {
    let expected = requirements(parent, attempt)?;
    let receipt_hash = verify_identities(parent.phase_id, attempt, request, receipt, Some(&archive.root))?;
    let record = TransportRecord {
        schema: "kbd.scoped-transport-observation/v1".into(),
        attribution: attempt.attribution.clone(),
        request: request.clone(),
        receipt: receipt.clone(),
        receipt_hash,
        comparisons: compare_artifacts(&expected, receipt),
    };
    Ok(VerifiedTransport { record })
}

/// Replay rechecks signed transport identities and deterministic comparisons.
/// No filesystem reads. Host artifact verification is trusted ONLY as a
/// fact recorded in an authenticated, supported canonical event, not from a
/// caller-supplied record. There is not yet an operational event producer.
fn replay_transport<'a>(
    parent: &Obligations<'_>, attempt: &ExpectedExecAttempt, event: &'a AuthenticatedTransportEvent,
) -> Result<ReplayVerifiedTransport<'a>> {
    let record = event.record();
    let expected = requirements(parent, attempt)?;
    let receipt_hash = verify_identities(parent.phase_id, attempt, &record.request, &record.receipt, None)?;
    if record.schema != "kbd.scoped-transport-observation/v1"
        || record.attribution != attempt.attribution
        || record.receipt_hash != receipt_hash
        || record.comparisons != compare_artifacts(&expected, &record.receipt)
    {
        return Err(refused(parent.phase_id, "transport-provenance:record-mismatch".into()));
    }
    Ok(ReplayVerifiedTransport { event })
}
