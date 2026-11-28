//! Pure scoped-evidence coverage rules for canonical phase closure.
//!
//! Intentionally disconnected from command admission, event replay, projections,
//! the offline inspector, and all operational entrypoints. The inputs below
//! are internal domain values, not wire formats. There is deliberately no
//! producer for the complete obligation projection or authenticated attribution
//! in this batch. In particular a legacy ScenarioOutcome, an Exec receipt alone,
//! or a caller-provided document MUST NOT construct an attributed observation.
//! The future canonical producer must establish the signed parent denominator,
//! frozen candidate, enrolled target, attempt, and verified product observations
//! (including controls, safety and cleanup) before supplying these values.
//! Coverage is not closure, authority, or permission to execute.

use super::{check_observations, refused};
use crate::phase_closure::{is_digest, ContractBinding, ScenarioOutcome, SCENARIO_PASS};
use crate::Result;
use std::collections::{BTreeMap, BTreeSet};

mod parent;
mod transport;
mod history;
mod close;
mod selection;
pub(super) use selection::{validate_selection, ScopedPacketSelection, SelectionReviewBinding};
pub(super) use close::{derive_for_close, ScopedClosureEvidence};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ObligationKey {
    pub(super) scenario_id: String,
    // Stable target slot in the complete parent interpretation, not a
    // receipt-supplied platform label.
    pub(super) target_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RequiredObservation {
    pub(super) key: ObligationKey,
    scenario_digest: String,
    // Identity of the exact required qualified target/profile.
    pub(super) target_identity_sha256: String,
    // Identity of the immutable complete obligation requirements, including
    // scope, fixture, oracle, control and safety/cleanup requirements.
    requirements_sha256: String,
    // Typed independent expected artifact identities from the exact parent.
    // They identify proof requirements, not product verdicts.
    artifact_checks: Vec<ArtifactExpectation>,
    pub(super) native_observer_profile_sha256: Option<String>,
    pub(super) case_command_ids: Vec<String>,
    pub(super) control_command_ids: Vec<String>,
    pub(super) safety_check_ids: Vec<String>,
    pub(super) prerequisite_command_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ArtifactCheckKind { Case, NegativeControl, Safety, Cleanup }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactExpectation {
    kind: ArtifactCheckKind,
    requirement_id: String,
    artifact_path: String,
    expected_sha256: String,
    // The actual independent source remains part of full adoption and
    // profile review; this string alone cannot establish independence.
    truth_source: String,
}

/// The complete parent-derived required target denominator, not the selected
/// packet. The canonical producer is still missing; the evaluator checks that
/// every binding scenario is represented, but cannot independently discover
/// additional required targets from ContractBinding's scenario-only map.
pub(super) struct Obligations<'a> {
    pub(super) project_id: &'a str,
    pub(super) run_id: &'a str,
    pub(super) phase_id: &'a str,
    pub(super) binding: &'a ContractBinding,
    pub(super) current_candidate_sha256: &'a str,
    pub(super) interpretation_sha256: &'a str,
    pub(super) required: &'a [RequiredObservation],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BoundObservationIdentity {
    pub(super) project_id: String,
    pub(super) run_id: String,
    pub(super) phase_id: String,
    pub(super) contract_digest: String,
    pub(super) binding_revision: u32,
    pub(super) candidate_sha256: String,
    pub(super) key: ObligationKey,
    pub(super) target_identity_sha256: String,
    pub(super) requirements_sha256: String,
    pub(super) attempt_id: String,
    pub(super) receipt_sha256: String,
    // Authenticated item identity within the receipt's verified observation
    // manifest. One receipt may support several distinct observation items.
    // No legacy or raw Exec field is assumed to supply this identity.
    pub(super) receipt_item_sha256: String,
    pub(super) producer_profile_sha256: String,
    pub(super) observer_manifest_sha256: String,
}

/// History stays append-only in the eventual canonical caller. This fold
/// neither removes nor modifies any input. Some(identity) is reserved for
/// observations attributed by the future authenticated canonical producer;
/// None preserves an observation without granting attribution. These types
/// have no deserialization or external constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) identity: Option<BoundObservationIdentity>,
    pub(super) outcome: ScenarioOutcome,
}

/// Names one immutable receipt item. The canonical source of these records
/// must authenticate its identity and the complete observation history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReceiptItemId {
    pub(super) receipt_sha256: String,
    pub(super) item_sha256: String,
}
impl ReceiptItemId {
    fn key(&self) -> (&str, &str) {
        (&self.receipt_sha256, &self.item_sha256)
    }
}

/// Private domain facts from a future independently authenticated, exact
/// existing-authority repair resolution. No wire schema, signer, grant,
/// constructor, or resolver of authority is provided here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RepairResolution {
    pub(super) failed: ReceiptItemId,
    pub(super) replacement: ReceiptItemId,
    pub(super) authority_record_sha256: String,
    pub(super) replacement_candidate_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedFailure {
    pub(super) failure_index: usize,
    pub(super) replacement_index: usize,
    pub(super) repair_resolution_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Attribution {
    Unknown,
    ParentMismatch,
    StaleBinding,
    UnknownObligation,
    ScenarioMismatch,
    TargetOrRequirementsMismatch,
    Incomplete,
    StaleCandidate,
    StaleCandidateFailure,
    NotExercised,
    CurrentFailure,
    CurrentPass,
}

/// A decision is keyed to one input history index, never a synthesized tuple.
/// A compatible repeat of the exact record is a reference, not a new credit.
/// A conflicting identity is preserved as history, but cannot earn credit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ObservationDecision {
    pub(super) input_index: usize,
    pub(super) attribution: Attribution,
    pub(super) replay_of: Option<usize>,
    pub(super) conflicting_receipt_item: bool,
    pub(super) accepted_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ObligationCoverage {
    current_passes: Vec<usize>,
    unresolved_failures: Vec<usize>,
    conflicts: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Coverage {
    pub(super) observations: Vec<ObservationDecision>,
    // Multiple independent concordant observations can support the same
    // obligation; it remains one element of the required denominator.
    pub(super) accepted: BTreeMap<ObligationKey, Vec<usize>>,
    pub(super) uncovered: BTreeSet<ObligationKey>,
    pub(super) unresolved_failures: BTreeMap<ObligationKey, Vec<usize>>,
    pub(super) conflicts: BTreeMap<ObligationKey, Vec<usize>>,
    pub(super) resolved_failures: BTreeMap<ObligationKey, Vec<ResolvedFailure>>,
    pub(super) rejected_repair_resolutions: Vec<usize>,
    // Preserve claimed failures which cannot be tied to a current, exact
    // obligation. This is unresolved history, not a confirmed product FAIL.
    // A parent amendment or incomplete attribution cannot clear it implicitly.
    pub(super) unresolved_failure_history: Vec<usize>,
}

impl Coverage {
    /// The evidence-only whole-parent decision. The canonical caller must
    /// additionally enforce all existing applicable closure gates (verified
    /// profile/source-at-use, attempt reconciliation, budgets and cleanup).
    /// This method never addresses release or creates new authority.
    fn require_complete(&self, phase_id: &str) -> Result<()> {
        let mut reasons = Vec::new();
        if !self.uncovered.is_empty() {
            reasons.push(format!("uncovered-obligations:{:?}", self.uncovered));
        }
        if !self.unresolved_failures.is_empty() {
            reasons.push(format!("unresolved-failures:{:?}", self.unresolved_failures));
        }
        if !self.conflicts.is_empty() {
            reasons.push(format!("conflicting-receipt-items:{:?}", self.conflicts));
        }
        if !self.rejected_repair_resolutions.is_empty() {
            reasons.push(format!("rejected-repair-resolutions:{:?}", self.rejected_repair_resolutions));
        }
        if !self.unresolved_failure_history.is_empty() {
            reasons.push(format!("unattributed-failure-history:{:?}", self.unresolved_failure_history));
        }
        if reasons.is_empty() { Ok(()) }
        else { Err(refused(phase_id, reasons.join("; "))) }
    }
}

fn name_is_valid(name: &str) -> bool {
    !name.is_empty() && name.trim() == name && !name.chars().any(char::is_control)
}

fn validate_denominator<'a>(
    parent: &Obligations<'a>,
) -> Result<BTreeMap<ObligationKey, &'a RequiredObservation>> {
    let binding = parent.binding;
    if ![parent.project_id, parent.run_id, parent.phase_id].into_iter().all(name_is_valid)
        || binding.binding_revision == 0
        || !is_digest(&binding.contract_digest)
        || !is_digest(parent.current_candidate_sha256)
        || !is_digest(parent.interpretation_sha256)
        || binding.scenario_digests.is_empty()
    {
        return Err(refused(parent.phase_id, "scoped-evidence-parent-identity-invalid".into()));
    }

    let mut required = BTreeMap::new();
    let mut represented = BTreeSet::new();
    for row in parent.required {
        if !name_is_valid(&row.key.scenario_id)
            || !name_is_valid(&row.key.target_id)
            || !is_digest(&row.scenario_digest)
            || !is_digest(&row.target_identity_sha256)
            || !is_digest(&row.requirements_sha256)
            || row.native_observer_profile_sha256.as_ref().is_some_and(|digest| !is_digest(digest))
            || binding.scenario_digests.get(&row.key.scenario_id) != Some(&row.scenario_digest)
        {
            return Err(refused(parent.phase_id, "scoped-evidence-obligation-identity-invalid".into()));
        }
        if required.insert(row.key.clone(), row).is_some() {
            return Err(refused(parent.phase_id, "scoped-evidence-duplicate-obligation".into()));
        }
        represented.insert(row.key.scenario_id.as_str());
    }
    // Never replace the whole parent with the packet's selected subset.
    if !represented.iter().copied().eq(binding.scenario_digests.keys().map(String::as_str)) {
        return Err(refused(parent.phase_id, "scoped-evidence-incomplete-parent-denominator".into()));
    }
    Ok(required)
}

fn classify(
    parent: &Obligations<'_>,
    required: &BTreeMap<ObligationKey, &RequiredObservation>,
    observation: &Observation,
) -> Attribution {
    let Some(identity) = observation.identity.as_ref() else {
        return Attribution::Unknown;
    };
    if ![identity.project_id.as_str(), identity.run_id.as_str(), identity.phase_id.as_str(),
        identity.attempt_id.as_str(), identity.key.scenario_id.as_str(), identity.key.target_id.as_str()]
        .into_iter().all(name_is_valid)
        || ![identity.contract_digest.as_str(), identity.candidate_sha256.as_str(),
            identity.target_identity_sha256.as_str(), identity.requirements_sha256.as_str(),
            identity.receipt_sha256.as_str(), identity.receipt_item_sha256.as_str(),
            identity.producer_profile_sha256.as_str(), identity.observer_manifest_sha256.as_str()]
            .into_iter().all(is_digest)
        || identity.binding_revision == 0
    {
        return Attribution::Incomplete;
    }
    if identity.project_id != parent.project_id
        || identity.run_id != parent.run_id
        || identity.phase_id != parent.phase_id
    {
        return Attribution::ParentMismatch;
    }
    if identity.contract_digest != parent.binding.contract_digest
        || identity.binding_revision != parent.binding.binding_revision
    {
        return Attribution::StaleBinding;
    }

    let Some(row) = required.get(&identity.key) else {
        return Attribution::UnknownObligation;
    };
    let outcome = &observation.outcome;
    // Exact identity, not legacy prefix matching or permissive optional digest.
    // No implicit carry-forward between contracts in scoped evidence.
    if outcome.id != row.key.scenario_id
        || outcome.scenario_digest.as_ref() != Some(&row.scenario_digest)
        || outcome.carried_from_contract.is_some()
    {
        return Attribution::ScenarioMismatch;
    }
    if identity.target_identity_sha256 != row.target_identity_sha256
        || identity.requirements_sha256 != row.requirements_sha256
        || row.native_observer_profile_sha256.as_ref().is_some_and(|profile| profile != &identity.producer_profile_sha256)
    {
        return Attribution::TargetOrRequirementsMismatch;
    }

    let stale = identity.candidate_sha256 != parent.current_candidate_sha256;
    match outcome.status.as_str() {
        "BLOCKED" | "NOT_EXERCISED" => {
            if stale { Attribution::StaleCandidate } else { Attribution::NotExercised }
        }
        SCENARIO_PASS | "FAIL" => {
            // Reuse canonical observation completeness; exact ID/digest were
            // required above. These fields are necessary, not a substitute for
            // the upstream authenticated observation and actual oracle.
            if check_observations(parent.phase_id, std::iter::once(&row.key.scenario_id),
                std::slice::from_ref(outcome)).is_err()
            {
                return Attribution::Incomplete;
            }
            match (stale, outcome.status.as_str() == SCENARIO_PASS) {
                (true, true) => Attribution::StaleCandidate,
                (true, false) => Attribution::StaleCandidateFailure,
                (false, true) => Attribution::CurrentPass,
                (false, false) => Attribution::CurrentFailure,
            }
        }
        _ => Attribution::Incomplete,
    }
}

/// Evaluate immutable parent coverage from already authenticated history.
/// Never perform effects, persist decisions, issue approval, or infer close.
/// The future caller must also enforce source-at-use, reservation/cleanup and
/// budgets; only an independently authenticated exact repair resolution can
/// resolve a particular failure. The fold cannot authorize repairs.
fn evaluate(
    parent: &Obligations<'_>,
    history: &[Observation],
    verified_repair_resolutions: &[RepairResolution],
) -> Result<Coverage> {
    let required = validate_denominator(parent)?;
    let mut decisions: Vec<_> = history.iter().enumerate().map(|(index, observation)| {
        ObservationDecision {
            input_index: index,
            attribution: classify(parent, &required, observation),
            replay_of: None,
            conflicting_receipt_item: false,
            accepted_current: false,
        }
    }).collect();

    // A receipt item, not a raw receipt encompassing multiple observations,
    // identifies one exact immutable record. Reuse never creates a new credit.
    let mut by_receipt_item = BTreeMap::<(&str, &str), Vec<usize>>::new();
    for (index, observation) in history.iter().enumerate() {
        if let Some(identity) = observation.identity.as_ref().filter(|identity| {
            is_digest(&identity.receipt_sha256) && is_digest(&identity.receipt_item_sha256)
        }) {
            by_receipt_item.entry((&identity.receipt_sha256, &identity.receipt_item_sha256))
                .or_default().push(index);
        }
    }
    for group in by_receipt_item.values() {
        let first = group[0];
        if group.iter().any(|&index| history[index] != history[first]) {
            for &index in group {
                decisions[index].conflicting_receipt_item = true;
            }
        } else {
            for &index in group.iter().skip(1) {
                decisions[index].replay_of = Some(first);
            }
        }
    }

    // Evaluate exact, independently authenticated repair links. A new PASS
    // alone never clears a failure. Invalid or conflicting resolution records
    // do not clear debt and are returned for the canonical caller to refuse.
    // Earlier legitimately resolved debt stays historical when candidate
    // freshness changes; its replacement can never earn stale current credit.
    let (resolved, rejected_repair_resolutions) = evaluate_repair_resolutions(
        history, &decisions, &by_receipt_item, verified_repair_resolutions,
    );

    let mut by_obligation: BTreeMap<_, _> = required.keys()
        .map(|key| (key.clone(), ObligationCoverage::default())).collect();
    for decision in &decisions {
        let Some(identity) = history[decision.input_index].identity.as_ref() else { continue };
        if identity.project_id != parent.project_id
            || identity.run_id != parent.run_id
            || identity.phase_id != parent.phase_id
        {
            continue;
        }
        let Some(coverage) = by_obligation.get_mut(&identity.key) else { continue };
        if decision.conflicting_receipt_item {
            // Quarantine only the affected obligation; unrelated current
            // evidence remains valid. No conflict resolution in this fold.
            coverage.conflicts.push(decision.input_index);
        } else if decision.replay_of.is_none() {
            match decision.attribution {
                Attribution::CurrentPass => coverage.current_passes.push(decision.input_index),
                Attribution::CurrentFailure | Attribution::StaleCandidateFailure => {
                    if !resolved.contains_key(&decision.input_index) {
                        coverage.unresolved_failures.push(decision.input_index);
                    }
                }
                _ => {}
            }
        }
    }

    let unresolved_failure_history = decisions.iter().filter_map(|decision| {
        let observation = &history[decision.input_index];
        let belongs_here = observation.identity.as_ref().is_none_or(|identity| {
            identity.project_id == parent.project_id
                && identity.run_id == parent.run_id
                && identity.phase_id == parent.phase_id
        });
        (belongs_here && observation.outcome.status == "FAIL"
            && !matches!(decision.attribution, Attribution::CurrentFailure | Attribution::StaleCandidateFailure))
            .then_some(decision.input_index)
    }).collect();
    let mut result = Coverage {
        observations: decisions,
        accepted: BTreeMap::new(),
        uncovered: BTreeSet::new(),
        unresolved_failures: BTreeMap::new(),
        conflicts: BTreeMap::new(),
        resolved_failures: BTreeMap::new(),
        rejected_repair_resolutions,
        unresolved_failure_history,
    };
    for (index, repair) in resolved {
        if let Some(identity) = history[index].identity.as_ref() {
            result.resolved_failures.entry(identity.key.clone()).or_default().push(repair);
        }
    }
    for (key, coverage) in by_obligation {
        let covered = !coverage.current_passes.is_empty()
            && coverage.unresolved_failures.is_empty()
            && coverage.conflicts.is_empty();
        if covered {
            for &index in &coverage.current_passes {
                result.observations[index].accepted_current = true;
            }
            result.accepted.insert(key.clone(), coverage.current_passes);
        } else {
            result.uncovered.insert(key.clone());
        }
        if !coverage.unresolved_failures.is_empty() {
            result.unresolved_failures.insert(key.clone(), coverage.unresolved_failures);
        }
        if !coverage.conflicts.is_empty() {
            result.conflicts.insert(key, coverage.conflicts);
        }
    }
    Ok(result)
}


// The receipt/item pair identifies the exact failed record and replacement.
// Canonical ingestion must separately verify the actual authority, attempt
// chronology, candidate-at-use, cleanup, and immutable event history.
fn evaluate_repair_resolutions(
    history: &[Observation],
    decisions: &[ObservationDecision],
    by_receipt_item: &BTreeMap<(&str, &str), Vec<usize>>,
    repairs: &[RepairResolution],
) -> (BTreeMap<usize, ResolvedFailure>, Vec<usize>) {
    let mut by_failed_item = BTreeMap::<(&str, &str), Vec<usize>>::new();
    for (index, repair) in repairs.iter().enumerate() {
        by_failed_item.entry(repair.failed.key()).or_default().push(index);
    }
    let mut resolved = BTreeMap::new();
    let mut rejected = Vec::new();
    for group in by_failed_item.values() {
        let index = group[0];
        let repair = &repairs[index];
        if group.iter().any(|&other| &repairs[other] != repair)
            || !is_digest(&repair.authority_record_sha256)
            || !is_digest(&repair.replacement_candidate_sha256)
        {
            rejected.extend(group.iter().copied());
            continue;
        }
        let pair = by_receipt_item.get(&repair.failed.key())
            .zip(by_receipt_item.get(&repair.replacement.key()))
            .map(|(failed, replacement)| (failed[0], replacement[0]));
        let Some((failed, replacement)) = pair else {
            rejected.extend(group.iter().copied());
            continue;
        };
        let failed_decision = &decisions[failed];
        let replacement_decision = &decisions[replacement];
        let valid = !failed_decision.conflicting_receipt_item
            && !replacement_decision.conflicting_receipt_item
            && failed < replacement
            && matches!(failed_decision.attribution,
                Attribution::CurrentFailure | Attribution::StaleCandidateFailure)
            && matches!(replacement_decision.attribution,
                Attribution::CurrentPass | Attribution::StaleCandidate)
            && history[replacement].outcome.status == SCENARIO_PASS
            && history[failed].identity.as_ref().zip(history[replacement].identity.as_ref())
                .is_some_and(|(failure, next)| {
                    failure.key == next.key
                        && failure.attempt_id != next.attempt_id
                        && next.candidate_sha256 == repair.replacement_candidate_sha256
                });
        if valid {
            resolved.insert(failed, ResolvedFailure {
                failure_index: failed,
                replacement_index: replacement,
                repair_resolution_index: index,
            });
        } else {
            rejected.extend(group.iter().copied());
        }
    }
    rejected.sort_unstable();
    (resolved, rejected)
}
