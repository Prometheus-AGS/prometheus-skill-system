//! Offline, non-authorizing inspection of proposed scoped VLBV admission.
//!
//! These documents are deliberately not an execution protocol. This module
//! accepts no grants or authenticated receipts, opens no runtime, and cannot activate a run,
//! admit an effect, or evaluate closure. Its invariants are document checks
//! only; canonical source, ownership, host and compatibility proof remain
//! separate prerequisites for any future enabling path.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[path = "scoped_admission_groundwork.rs"]
mod groundwork;

use groundwork::{DeclaredBoundaryPacket, DeclaredNegativeControl, DeclaredObservation,
    DeclaredSafetyCheck, GroundworkReport, InspectionContext};

const INSPECTION_KIND: &str = "kbd.scoped-admission-inspection";
const HISTORICAL_PROFILE: &str = "exec-1.7.0-tier-p-seatbelt-macos-x86_64";
const HISTORICAL_PLATFORM: &str = "macos-x86_64";
const HISTORICAL_RUNTIME_BUILD: &str =
    "a77a8f4a861092ac919125f77d1707eefe071d5984e2fa58cdd289978511841c";
const HISTORICAL_RUNTIME_COMMIT: &str = "1b8d905f09fb233aec7eccbf1b3c0de8e032479d";
const HISTORICAL_RUNTIME_RECEIPT: &str =
    "95810a5221bd432a4f3041b45e8eb065efb735dfc227be66482057c652be83cd";
const HISTORICAL_INSTALL_BUILD: &str =
    "ae70b4d2e128f0f69afe25f21983a84a56c93fba351821f020e4c37976981e4f";
const HISTORICAL_INSTALLED: &str =
    "d4fcd01e7a0c4cbb2a6dc9657557c5a0aa3dbd54a21845deefe6a52bc68bcd1c";

// The source for the historical reference is the named local evidence archive,
// not the package build matrix or an installation receipt from another host.
const HISTORICAL_EVIDENCE: &str =
    ".kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-002-tier-p-sidecar.md";

// The owner-approved arm64 source/build packet passed; the signed installed
// artifact has NOT been independently exercised as a runner. This is a
// reference classification only, never an eligible profile.
const ARM64_PROFILE: &str = "exec-1.7.0-tier-p-seatbelt-macos-aarch64";
const ARM64_PLATFORM: &str = "macos-aarch64";
const ARM64_SOURCE_COMMIT: &str = "66c35c5869b95756250fc651ea7fcfb119d8d060";
const ARM64_BUILD: &str = "ca7ea8dec23cf21efffcbd5d86d4df1087714feb0ccd1700f2dfd7d2165aa065";
const ARM64_INSTALLED: &str = "6ca67fd7b411ec30945ee903228a2b0501bca78c7b442bcef4196b8e97310376";
const ARM64_EVIDENCE: &str =
    "docs/superpowers/plans/2026-11-28-kbd-scoped-vlbv-exec-arm64-source-qualification.md";

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Identity {
    project_id: String,
    run_id: String,
    phase_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InspectionProposal {
    schema_version: String,
    kind: String,
    parent: ProposedParent,
    candidate_sha256: String,
    previous_candidate_sha256: Option<String>,
    scenarios: Vec<String>,
    target: ProposedTarget,
    #[serde(default)]
    observations: Vec<DeclaredObservation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProposedParent {
    identity: Identity,
    contract_digest: String,
    binding_revision: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProposedTarget {
    profile_id: String,
    platform: String,
    runner_sha256: String,
}

// These are projections of pre-existing documents, not new authority
// formats. Extra fields in the adopted contract and disposition are expected.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    kind: String,
    contract_version: String,
    identity: Identity,
    scenario_matrix: Vec<Scenario>,
    boundary_packet: Option<DeclaredBoundaryPacket>,
    #[serde(default)]
    required_safety_checks: Vec<DeclaredSafetyCheck>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    id: String,
    mandatory: Option<bool>,
    owner_task: Option<String>,
    #[serde(default)]
    scope_refs: Vec<String>,
    negative_control: Option<DeclaredNegativeControl>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DispositionDocument {
    schema: String,
    project_id: String,
    run_id: String,
    phase_id: String,
    binding: Option<ProvidedBinding>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProvidedBinding {
    contract_digest: String,
    binding_revision: u32,
    scenario_digests: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoricalProfile {
    profile_id: &'static str,
    platform: &'static str,
    runtime_commit: &'static str,
    runtime_build_sha256: &'static str,
    runtime_receipt_sha256: &'static str,
    installation_build_sha256: &'static str,
    installed_sha256: &'static str,
    evidence_path: &'static str,
    eligible: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Arm64SourceBuildReference {
    profile_id: &'static str,
    platform: &'static str,
    source_commit: &'static str,
    reproduced_build_sha256: &'static str,
    recorded_installed_sha256: &'static str,
    evidence_path: &'static str,
    installed_runtime_verified: bool,
    eligible: bool,
}

/// A diagnostic report, never a readiness token. All authorization and
/// closure booleans are generated here, not accepted from input.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionReport {
    schema: &'static str,
    status: &'static str,
    execution_authorized: bool,
    activation_authorized: bool,
    closure_eligible: bool,
    canonical_binding_verified: bool,
    parent_snapshot_matches_provided_binding: bool,
    inspected_snapshot_sha256: String,
    inspected_disposition_sha256: String,
    inspected_proposal_sha256: String,
    parent_scenario_count: usize,
    selected_scenarios: Vec<String>,
    remaining_scenarios: Vec<String>,
    candidate_changed: Option<bool>,
    target_reference_status: &'static str,
    historical_profile: HistoricalProfile,
    arm64_source_build_reference: Arm64SourceBuildReference,
    transaction_groundwork: GroundworkReport,
    blockers: Vec<&'static str>,
}

/// Inspect exact snapshot bytes against a *provided*, unauthenticated
/// disposition document. No call to this function establishes the current
/// canonical binding or target qualification.
pub fn inspect(
    snapshot_bytes: &[u8],
    disposition_bytes: &[u8],
    proposal_bytes: &[u8],
) -> Result<InspectionReport, String> {
    let proposal: InspectionProposal = decode(proposal_bytes, "proposal")?;
    let parent: Snapshot = decode(snapshot_bytes, "snapshot")?;
    let provided: DispositionDocument = decode(disposition_bytes, "disposition")?;
    if proposal.kind != INSPECTION_KIND || proposal.schema_version != "1" {
        return Err("unsupported offline inspection document".into());
    }
    if parent.kind != "kbd.adopted-contract" || parent.contract_version != "2" {
        return Err("inspection requires a version 2 adopted snapshot".into());
    }
    if provided.schema != "kbd.phase-disposition/v1" {
        return Err("unsupported provided disposition document".into());
    }
    let provided_identity = Identity {
        project_id: provided.project_id,
        run_id: provided.run_id,
        phase_id: provided.phase_id,
    };
    for value in [
        &provided_identity.project_id,
        &provided_identity.run_id,
        &provided_identity.phase_id,
        &proposal.target.profile_id,
        &proposal.target.platform,
    ] {
        if value.is_empty() || value.trim() != value.as_str() || value.chars().any(char::is_control) {
            return Err("identity fields must be nonempty printable strings".into());
        }
    }
    if provided_identity != parent.identity || parent.identity != proposal.parent.identity {
        return Err("provided parent, snapshot and proposal identities differ".into());
    }
    let binding = provided.binding.ok_or("provided document has no binding")?;
    let exact_snapshot_digest = format!("{:x}", Sha256::digest(snapshot_bytes));
    if binding.binding_revision == 0
        || binding.binding_revision != proposal.parent.binding_revision
        || binding.contract_digest != exact_snapshot_digest
        || proposal.parent.contract_digest != exact_snapshot_digest
    {
        return Err("provided binding and proposal do not name these exact snapshot bytes".into());
    }
    for digest in [
        &proposal.candidate_sha256,
        &proposal.target.runner_sha256,
    ].into_iter().chain(proposal.previous_candidate_sha256.iter()) {
        if !valid_sha256(digest) {
            return Err("invalid SHA-256 in proposal".into());
        }
    }

    let parent_ids: BTreeSet<&str> = parent.scenario_matrix.iter().map(|row| row.id.as_str()).collect();
    if parent_ids.is_empty() || parent_ids.contains("") || parent_ids.len() != parent.scenario_matrix.len() {
        return Err("snapshot scenario IDs must be nonempty and unique".into());
    }
    let provided_ids: BTreeSet<&str> = binding.scenario_digests.keys().map(String::as_str).collect();
    if provided_ids != parent_ids || binding.scenario_digests.values().any(|digest| !valid_sha256(digest)) {
        return Err("provided binding does not name every snapshot scenario with a digest".into());
    }

    let selected: BTreeSet<&str> = proposal.scenarios.iter().map(String::as_str).collect();
    if selected.is_empty() || selected.len() != proposal.scenarios.len() || !selected.is_subset(&parent_ids) {
        return Err("selection must be a nonempty, unique subset of parent scenarios".into());
    }

    let target_reference_status = if proposal.target.profile_id == HISTORICAL_PROFILE
        && proposal.target.platform == HISTORICAL_PLATFORM
        && proposal.target.runner_sha256 == HISTORICAL_RUNTIME_BUILD
    {
        "HISTORICAL_RUNTIME_REFERENCE_ONLY"
    } else if proposal.target.profile_id == HISTORICAL_PROFILE
        && proposal.target.platform == HISTORICAL_PLATFORM
        && (proposal.target.runner_sha256 == HISTORICAL_INSTALL_BUILD
            || proposal.target.runner_sha256 == HISTORICAL_INSTALLED)
    {
        "HISTORICAL_INSTALLATION_REFERENCE_ONLY"
    } else if proposal.target.profile_id == ARM64_PROFILE
        && proposal.target.platform == ARM64_PLATFORM
        && (proposal.target.runner_sha256 == ARM64_BUILD
            || proposal.target.runner_sha256 == ARM64_INSTALLED)
    {
        "ARM64_SOURCE_BUILD_REFERENCE_ONLY"
    } else {
        "UNSUPPORTED_OR_UNQUALIFIED"
    };

    let candidate_changed = proposal
        .previous_candidate_sha256
        .as_ref()
        .map(|previous| previous != &proposal.candidate_sha256);
    let mut blockers = vec![
        "ACTIVATION_DISABLED",
        "LEGACY_COMPATIBILITY_UNVERIFIED",
        "ACTIVE_PROFILE_UNVERIFIED",
        "CANONICAL_BINDING_AND_OWNER_READINESS_NOT_EVALUATED",
        "COMMAND_PREREQUISITES_CONTROLS_AND_SAFETY_NOT_EVALUATED",
        "SIGNED_AUTHORITY_NOT_EVALUATED",
        "CURRENT_CANDIDATE_AND_TARGET_NOT_VERIFIED",
        "NO_EVIDENCE_OR_CLOSURE_CREDIT",
        "DECLARED_OBSERVATIONS_NOT_CANONICALLY_VERIFIED",
        "WHOLE_PARENT_CLOSURE_NOT_EVALUATED",
    ];
    if candidate_changed == Some(true) {
        blockers.push("CANDIDATE_CHANGED_FRESH_PROOF_REQUIRED");
    }

    let transaction_groundwork = groundwork::inspect(
        &InspectionContext {
            scenarios: &parent.scenario_matrix,
            selected: &selected,
            scenario_digests: &binding.scenario_digests,
            contract_digest: &exact_snapshot_digest,
            binding_revision: binding.binding_revision,
            candidate_sha256: &proposal.candidate_sha256,
            target: &proposal.target,
            packet: parent.boundary_packet.as_ref(),
            safety_checks: &parent.required_safety_checks,
        },
        proposal.observations,
    )?;

    Ok(InspectionReport {
        schema: "kbd.scoped-admission-inspection-report/v1",
        status: "BLOCKED",
        execution_authorized: false,
        activation_authorized: false,
        closure_eligible: false,
        canonical_binding_verified: false,
        parent_snapshot_matches_provided_binding: true,
        inspected_snapshot_sha256: exact_snapshot_digest,
        inspected_disposition_sha256: format!("{:x}", Sha256::digest(disposition_bytes)),
        inspected_proposal_sha256: format!("{:x}", Sha256::digest(proposal_bytes)),
        parent_scenario_count: parent_ids.len(),
        selected_scenarios: selected.iter().map(|id| (*id).to_owned()).collect(),
        remaining_scenarios: parent_ids.difference(&selected).map(|id| (*id).to_owned()).collect(),
        candidate_changed,
        target_reference_status,
        historical_profile: HistoricalProfile {
            profile_id: HISTORICAL_PROFILE,
            platform: HISTORICAL_PLATFORM,
            runtime_commit: HISTORICAL_RUNTIME_COMMIT,
            runtime_build_sha256: HISTORICAL_RUNTIME_BUILD,
            runtime_receipt_sha256: HISTORICAL_RUNTIME_RECEIPT,
            installation_build_sha256: HISTORICAL_INSTALL_BUILD,
            installed_sha256: HISTORICAL_INSTALLED,
            evidence_path: HISTORICAL_EVIDENCE,
            eligible: false,
        },
        arm64_source_build_reference: Arm64SourceBuildReference {
            profile_id: ARM64_PROFILE,
            platform: ARM64_PLATFORM,
            source_commit: ARM64_SOURCE_COMMIT,
            reproduced_build_sha256: ARM64_BUILD,
            recorded_installed_sha256: ARM64_INSTALLED,
            evidence_path: ARM64_EVIDENCE,
            installed_runtime_verified: false,
            eligible: false,
        },
        transaction_groundwork,
        blockers,
    })
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8], document: &str) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("invalid {document}: {error}"))
}

fn valid_sha256(digest: &str) -> bool {
    digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
