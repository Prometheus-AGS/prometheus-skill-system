//! Non-authorizing document groundwork for the scoped-admission inspector.
//!
//! All inputs here are *declared references*, not authenticated events,
//! evidence, receipts, eligibility or readiness. No canonical writer imports
//! this module; it is a child of the offline inspector. Keep each observation
//! intact so reporting cannot mix references from different receipts.

use super::{valid_sha256, ProposedTarget, Scenario};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// Diagnostic resource ceilings only. They are deliberately not execution
// authority, contract budgets or eligibility rules. All values are bounded
// even when a supplied document repeats references many times.
const MAX_SCENARIOS: usize = 1024;
const MAX_COMMANDS: usize = 4096;
const MAX_SAFETY_CHECKS: usize = 2048;
const MAX_OBSERVATIONS: usize = 2048;
const MAX_REFERENCE_LINKS: usize = 16_384;
const MAX_REFERENCE_BYTES: usize = 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeclaredBoundaryPacket {
    #[serde(default)]
    commands: Vec<DeclaredCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeclaredCommand {
    id: String,
    kind: String,
    #[serde(default)]
    scenario_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeclaredNegativeControl {
    command_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeclaredSafetyCheck {
    id: String,
    #[serde(default)]
    scenario_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RecordedVerdict {
    Pass,
    Fail,
    Blocked,
    NotExercised,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DeclaredTarget {
    profile_id: Option<String>,
    platform: Option<String>,
    runner_sha256: Option<String>,
}

/// One input tuple, copied whole into the diagnostic output. The document
/// may claim a receipt digest; no receipt bytes or signature are authenticated.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DeclaredObservation {
    receipt_sha256: Option<String>,
    attempt_id: Option<String>,
    contract_digest: Option<String>,
    binding_revision: Option<u32>,
    scenario_id: Option<String>,
    scenario_digest: Option<String>,
    candidate_sha256: Option<String>,
    target: Option<DeclaredTarget>,
    scope_ref: Option<String>,
    fixture_ref: Option<String>,
    oracle_ref: Option<String>,
    recorded_verdict: Option<RecordedVerdict>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MappingInspection {
    scenario_id: String,
    selected: bool,
    declared_owner_task: Option<String>,
    declared_case_command_refs: Vec<String>,
    declared_control_command_ref: Option<String>,
    declared_safety_check_ids: Vec<String>,
    document_issues: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ReferenceState {
    UnknownAttribution,
    StaleCandidateReference,
    ParentReferenceMismatch,
    OtherTargetNotEvaluated,
    RepeatedReceiptReference,
    DeclaredReferencesMatchUnverified,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SafetyMappingIssue {
    declared_safety_check_id: String,
    document_issues: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservationInspection {
    /// Zero-based position in the input; identity is not inferred from position.
    input_index: usize,
    declared_tuple: DeclaredObservation,
    reference_state: ReferenceState,
    reasons: Vec<&'static str>,
    canonical_attribution: &'static str,
    accepted_current: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParentObligationInspection {
    scenario_id: String,
    declared_mandatory: bool,
    selected: bool,
    declared_observation_indexes: Vec<usize>,
    matching_unverified_reference_count: usize,
    canonical_accepted_count: usize,
    closure_status: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GroundworkReport {
    schema: &'static str,
    status: &'static str,
    observation_source: &'static str,
    execution_authorized: bool,
    whole_parent_closure_evaluated: bool,
    closure_eligible: bool,
    mappings: Vec<MappingInspection>,
    safety_mapping_issues: Vec<SafetyMappingIssue>,
    observations: Vec<ObservationInspection>,
    parent_obligations: Vec<ParentObligationInspection>,
}

pub(super) struct InspectionContext<'a> {
    pub(super) scenarios: &'a [Scenario],
    pub(super) selected: &'a BTreeSet<&'a str>,
    pub(super) scenario_digests: &'a BTreeMap<String, String>,
    pub(super) contract_digest: &'a str,
    pub(super) binding_revision: u32,
    pub(super) candidate_sha256: &'a str,
    pub(super) target: &'a ProposedTarget,
    pub(super) packet: Option<&'a DeclaredBoundaryPacket>,
    pub(super) safety_checks: &'a [DeclaredSafetyCheck],
}

pub(super) fn inspect(
    context: &InspectionContext<'_>,
    declared_observations: Vec<DeclaredObservation>,
) -> Result<GroundworkReport, String> {
    validate_limits(context, &declared_observations)?;
    let mappings = inspect_mappings(context);
    let safety_mapping_issues = inspect_safety_document(context);
    let scenarios_by_id: BTreeMap<_, _> = context.scenarios.iter()
        .map(|scenario| (scenario.id.as_str(), scenario)).collect();
    let mut counts = BTreeMap::<String, usize>::new();
    for observation in &declared_observations {
        if let Some(receipt) = observation.receipt_sha256.as_deref().filter(|s| valid_sha256(s)) {
            *counts.entry(receipt.to_owned()).or_default() += 1;
        }
    }

    let observations: Vec<_> = declared_observations
        .into_iter()
        .enumerate()
        .map(|(input_index, observation)| {
            let repeated = observation
                .receipt_sha256
                .as_ref()
                .and_then(|id| counts.get(id))
                .is_some_and(|count| *count > 1);
            let scenario = observation.scenario_id.as_deref()
                .and_then(|id| scenarios_by_id.get(id).copied());
            let (reference_state, reasons) = inspect_reference(context, scenario, &observation, repeated);
            ObservationInspection {
                input_index,
                // Move the original tuple without constructing a new tuple
                // from matching fields elsewhere in the document.
                declared_tuple: observation,
                reference_state,
                reasons,
                canonical_attribution: "NOT_EVALUATED",
                accepted_current: false,
            }
        })
        .collect();

    let mut observation_indexes = BTreeMap::<&str, Vec<usize>>::new();
    for observation in &observations {
        if let Some(id) = observation.declared_tuple.scenario_id.as_deref() {
            observation_indexes.entry(id).or_default().push(observation.input_index);
        }
    }
    let parent_obligations = context.scenarios.iter().map(|scenario| {
        let own = observation_indexes.get(scenario.id.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        ParentObligationInspection {
            scenario_id: scenario.id.clone(),
            declared_mandatory: scenario.mandatory.unwrap_or(true),
            selected: context.selected.contains(scenario.id.as_str()),
            declared_observation_indexes: own.to_vec(),
            matching_unverified_reference_count: own.iter().filter(|&&index| {
                observations[index].reference_state == ReferenceState::DeclaredReferencesMatchUnverified
            }).count(),
            canonical_accepted_count: 0,
            // No target denominator or canonical evidence is established by
            // this document, even when all declared references look complete.
            closure_status: "NOT_EVALUATED",
        }
    }).collect();

    drop(observation_indexes);
    Ok(GroundworkReport {
        schema: "kbd.scoped-transaction-groundwork-report/v1",
        status: "BLOCKED",
        observation_source: "UNAUTHENTICATED_DECLARED_REFERENCES",
        execution_authorized: false,
        whole_parent_closure_evaluated: false,
        closure_eligible: false,
        mappings,
        safety_mapping_issues,
        observations,
        parent_obligations,
    })
}

// Limit the collection sizes, total association count and displayed strings
// before indexing or cloning data. This is a diagnostic rejection, not a
// claim about the size or validity of any real parent contract.
fn validate_limits(context: &InspectionContext<'_>, observations: &[DeclaredObservation]) -> Result<(), String> {
    let commands = context.packet.map(|packet| packet.commands.as_slice()).unwrap_or(&[]);
    if context.scenarios.len() > MAX_SCENARIOS || commands.len() > MAX_COMMANDS
        || context.safety_checks.len() > MAX_SAFETY_CHECKS || observations.len() > MAX_OBSERVATIONS
    {
        return Err("offline groundwork exceeds diagnostic collection limits".into());
    }
    let mut links = 0usize;
    let check = |value: Option<&str>| -> Result<(), String> {
        if value.is_some_and(|s| s.len() > MAX_REFERENCE_BYTES) {
            Err("offline groundwork reference exceeds diagnostic byte limit".into())
        } else {
            Ok(())
        }
    };
    for scenario in context.scenarios {
        check(Some(scenario.id.as_str()))?;
        check(scenario.owner_task.as_deref())?;
        check(scenario.negative_control.as_ref().and_then(|control| control.command_ref.as_deref()))?;
        links = links.saturating_add(scenario.scope_refs.len());
        for scope in &scenario.scope_refs {
            check(Some(scope.as_str()))?;
        }
    }
    for command in commands {
        check(Some(command.id.as_str()))?;
        check(Some(command.kind.as_str()))?;
        links = links.saturating_add(command.scenario_ids.len());
        for id in &command.scenario_ids {
            check(Some(id.as_str()))?;
        }
    }
    for safety in context.safety_checks {
        check(Some(safety.id.as_str()))?;
        links = links.saturating_add(safety.scenario_ids.len());
        for id in &safety.scenario_ids {
            check(Some(id.as_str()))?;
        }
    }
    if links > MAX_REFERENCE_LINKS {
        return Err("offline groundwork exceeds diagnostic association limit".into());
    }
    check(Some(context.target.profile_id.as_str()))?;
    check(Some(context.target.platform.as_str()))?;
    for observation in observations {
        for value in [
            observation.receipt_sha256.as_deref(), observation.attempt_id.as_deref(),
            observation.contract_digest.as_deref(), observation.scenario_id.as_deref(),
            observation.scenario_digest.as_deref(), observation.candidate_sha256.as_deref(),
            observation.scope_ref.as_deref(), observation.fixture_ref.as_deref(),
            observation.oracle_ref.as_deref(),
        ] {
            check(value)?;
        }
        if let Some(target) = &observation.target {
            check(target.profile_id.as_deref())?;
            check(target.platform.as_deref())?;
            check(target.runner_sha256.as_deref())?;
        }
    }
    Ok(())
}

fn nonempty(value: Option<&str>) -> bool {
    value.is_some_and(|s| !s.is_empty() && s.trim() == s && !s.chars().any(char::is_control))
}

fn has_digest(value: Option<&str>) -> bool {
    value.is_some_and(valid_sha256)
}

// Report malformed safety declarations even when they have no known parent
// row to attach to. This remains a reference diagnostic, not canonical lint.
fn inspect_safety_document(context: &InspectionContext<'_>) -> Vec<SafetyMappingIssue> {
    let parent_ids: BTreeSet<_> = context.scenarios.iter()
        .map(|scenario| scenario.id.as_str()).collect();
    context.safety_checks.iter().filter_map(|check| {
        let mut issues = Vec::new();
        if !nonempty(Some(check.id.as_str())) {
            issues.push("SAFETY_CHECK_ID_MISSING_OR_INVALID");
        }
        if check.scenario_ids.is_empty() {
            issues.push("SAFETY_CHECK_MAPPING_EMPTY");
        }
        if check.scenario_ids.iter().any(|id| !parent_ids.contains(id.as_str())) {
            issues.push("SAFETY_CHECK_MAPPING_REFERENCES_UNKNOWN_SCENARIO");
        }
        if issues.is_empty() {
            None
        } else {
            Some(SafetyMappingIssue {
                declared_safety_check_id: check.id.clone(),
                document_issues: issues,
            })
        }
    }).collect()
}

fn inspect_mappings(context: &InspectionContext<'_>) -> Vec<MappingInspection> {
    let commands = context.packet.map(|packet| packet.commands.as_slice()).unwrap_or(&[]);
    let parent_ids: BTreeSet<_> = context.scenarios.iter().map(|scenario| scenario.id.as_str()).collect();
    let mut commands_by_id = BTreeMap::<&str, Vec<&DeclaredCommand>>::new();
    let mut cases_by_scenario = BTreeMap::<&str, Vec<&DeclaredCommand>>::new();
    let mut control_owners = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut safety_by_scenario = BTreeMap::<&str, Vec<&DeclaredSafetyCheck>>::new();
    for command in commands {
        commands_by_id.entry(command.id.as_str()).or_default().push(command);
        if command.kind == "scenario" {
            for id in command.scenario_ids.iter().map(String::as_str).collect::<BTreeSet<_>>() {
                cases_by_scenario.entry(id).or_default().push(command);
            }
        }
    }
    for scenario in context.scenarios {
        if let Some(control_ref) = scenario.negative_control.as_ref().and_then(|control| control.command_ref.as_deref()) {
            control_owners.entry(control_ref).or_default().insert(scenario.id.as_str());
        }
    }
    for safety in context.safety_checks {
        for id in safety.scenario_ids.iter().map(String::as_str).collect::<BTreeSet<_>>() {
            safety_by_scenario.entry(id).or_default().push(safety);
        }
    }
    context.scenarios.iter().map(|scenario| {
        let selected = context.selected.contains(scenario.id.as_str());
        let cases = cases_by_scenario.get(scenario.id.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        let applicable_safety = safety_by_scenario.get(scenario.id.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        let control_ref = scenario.negative_control.as_ref().and_then(|control| control.command_ref.as_deref());
        let named_controls = control_ref.and_then(|id| commands_by_id.get(id)).map(Vec::as_slice).unwrap_or(&[]);
        let controls: Vec<_> = named_controls.iter().filter(|command| command.kind == "negativeControl").collect();
        let mut issues = vec![
            "CANONICAL_OWNER_READINESS_NOT_EVALUATED",
            "PREREQUISITE_CLOSURE_NOT_ESTABLISHED",
            "SAFETY_CONTROLS_NOT_VERIFIED",
        ];
        if !nonempty(scenario.owner_task.as_deref()) {
            issues.push("OWNER_REFERENCE_MISSING");
        }
        if cases.is_empty() {
            issues.push("CASE_MAPPING_MISSING");
        } else if cases.iter().any(|case| {
            !nonempty(Some(case.id.as_str())) || commands_by_id.get(case.id.as_str()).map(Vec::len) != Some(1)
        }) {
            issues.push("CASE_COMMAND_ID_INVALID_OR_AMBIGUOUS");
        }
        if !nonempty(control_ref) || controls.len() != 1 || named_controls.len() != 1 {
            issues.push("CONTROL_MAPPING_MISSING_OR_AMBIGUOUS");
        }
        if selected && cases.iter().any(|case| {
            case.scenario_ids.iter().any(|id| !context.selected.contains(id.as_str()))
        }) {
            issues.push("CASE_MAPPING_INCLUDES_UNSELECTED_SCOPE");
        }
        if selected && control_ref.is_some() &&
            (controls.iter().any(|control| {
                control.scenario_ids.iter().any(|id| !context.selected.contains(id.as_str()))
            }) || control_ref.and_then(|id| control_owners.get(id)).is_some_and(|owners| {
                owners.iter().any(|id| !context.selected.contains(*id))
            }))
        {
            issues.push("CONTROL_MAPPING_SHARED_WITH_UNSELECTED_SCOPE");
        }
        if selected && applicable_safety.iter().any(|safety| {
            safety.scenario_ids.iter().any(|id| !context.selected.contains(id.as_str()))
        }) {
            issues.push("SAFETY_CHECK_MAPPING_INCLUDES_UNSELECTED_SCOPE");
        }
        if applicable_safety.iter().any(|safety| {
            safety.scenario_ids.iter().any(|id| !parent_ids.contains(id.as_str()))
        }) {
            issues.push("SAFETY_CHECK_MAPPING_REFERENCES_UNKNOWN_SCENARIO");
        }
        MappingInspection {
            scenario_id: scenario.id.clone(),
            selected,
            declared_owner_task: scenario.owner_task.clone(),
            declared_case_command_refs: cases.iter().map(|command| command.id.clone()).collect(),
            declared_control_command_ref: control_ref.map(str::to_owned),
            declared_safety_check_ids: applicable_safety.iter().map(|check| check.id.clone()).collect(),
            document_issues: issues,
        }
    }).collect()
}

fn inspect_reference(
    context: &InspectionContext<'_>,
    scenario: Option<&Scenario>,
    observation: &DeclaredObservation,
    repeated: bool,
) -> (ReferenceState, Vec<&'static str>) {
    let mut reasons = Vec::new();
    let mut unknown = false;
    let mut parent_mismatch = false;

    if !has_digest(observation.receipt_sha256.as_deref()) || !nonempty(observation.attempt_id.as_deref()) {
        unknown = true;
        reasons.push("RECEIPT_OR_ATTEMPT_REFERENCE_MISSING_OR_INVALID");
    }
    if scenario.is_none() {
        unknown = true;
        reasons.push("SCENARIO_REFERENCE_UNKNOWN");
    }
    if !has_digest(observation.contract_digest.as_deref()) || observation.binding_revision.is_none() {
        unknown = true;
        reasons.push("PARENT_BINDING_REFERENCE_MISSING_OR_INVALID");
    } else if observation.contract_digest.as_deref() != Some(context.contract_digest)
        || observation.binding_revision != Some(context.binding_revision)
    {
        parent_mismatch = true;
        reasons.push("PARENT_BINDING_REFERENCE_MISMATCH");
    }
    if !has_digest(observation.scenario_digest.as_deref()) {
        unknown = true;
        reasons.push("SCENARIO_DIGEST_REFERENCE_MISSING_OR_INVALID");
    } else if let Some(scenario) = scenario {
        if context.scenario_digests.get(&scenario.id) != observation.scenario_digest.as_ref() {
            parent_mismatch = true;
            reasons.push("SCENARIO_DIGEST_REFERENCE_MISMATCH");
        }
    }

    let stale = has_digest(observation.candidate_sha256.as_deref())
        && observation.candidate_sha256.as_deref() != Some(context.candidate_sha256);
    if !has_digest(observation.candidate_sha256.as_deref()) {
        unknown = true;
        reasons.push("CANDIDATE_REFERENCE_UNKNOWN");
    } else if stale {
        reasons.push("CANDIDATE_REFERENCE_STALE");
    }

    let mut other_target = false;
    match &observation.target {
        Some(target) if nonempty(target.profile_id.as_deref())
            && nonempty(target.platform.as_deref())
            && has_digest(target.runner_sha256.as_deref()) => {
            other_target = target.profile_id.as_deref() != Some(context.target.profile_id.as_str())
                || target.platform.as_deref() != Some(context.target.platform.as_str())
                || target.runner_sha256.as_deref() != Some(context.target.runner_sha256.as_str());
            if other_target {
                reasons.push("OTHER_TARGET_REFERENCE_NOT_EVALUATED");
            }
        }
        _ => {
            unknown = true;
            reasons.push("TARGET_REFERENCE_UNKNOWN");
        }
    }
    if !nonempty(observation.scope_ref.as_deref())
        || !nonempty(observation.fixture_ref.as_deref())
        || !nonempty(observation.oracle_ref.as_deref())
        || observation.recorded_verdict.is_none()
    {
        unknown = true;
        reasons.push("SCOPE_FIXTURE_ORACLE_OR_VERDICT_REFERENCE_UNKNOWN");
    }
    if let (Some(scenario), Some(scope)) = (scenario, observation.scope_ref.as_ref()) {
        if !scenario.scope_refs.contains(scope) {
            unknown = true;
            reasons.push("SCOPE_REFERENCE_NOT_ESTABLISHED_IN_PARENT");
        }
    }
    if repeated {
        reasons.push("REPEATED_RECEIPT_REFERENCE_REQUIRES_RECONCILIATION");
    }

    // Multiple issues remain in the row regardless of the display priority.
    // Never allow one observation to overwrite or invalidate a neighbor.
    let state = if repeated {
        ReferenceState::RepeatedReceiptReference
    } else if parent_mismatch {
        ReferenceState::ParentReferenceMismatch
    } else if stale {
        ReferenceState::StaleCandidateReference
    } else if unknown {
        ReferenceState::UnknownAttribution
    } else if other_target {
        ReferenceState::OtherTargetNotEvaluated
    } else {
        ReferenceState::DeclaredReferencesMatchUnverified
    };
    reasons.push("CANONICAL_RECEIPT_AND_ATTRIBUTION_NOT_VERIFIED");
    (state, reasons)
}
