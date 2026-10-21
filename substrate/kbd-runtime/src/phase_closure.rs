//! Canonical phase closure (issue #192).
//!
//! One authority decides whether a contract-bound phase is closed: a typed
//! `PhaseDispositionRecorded` event, bound to project, run, phase, the bound
//! contract digest and the evidence digest. Evaluation, dispatch, Reflect,
//! child-exit and next-phase all read the same disposition.
//!
//! Invariants enforced in `apply` (so a refusal emits no event and replay is
//! deterministic):
//! * a bound phase reaches `Complete` only through a PASS disposition;
//! * a terminal closure is immutable; a repeated identical close is a no-op;
//! * PASS requires every bound scenario to have PASS evidence; evidence for
//!   scenarios outside the bound contract never changes the verdict;
//! * recorded budget consumption never decreases;
//! * a changed contract digest requires a signed amendment manifest;
//! * a successor phase starts only from a PASS closure with a signed
//!   start-successor manifest, exactly once.
//!
//! Phases that were never bound keep their legacy behaviour unchanged.

use crate::scope_authority::{
    self, ScenarioChanges, ScopeAction, ScopeAuthorityInput, ScopeAuthorityManifest,
    ScopeAuthorityRecord,
};
use crate::{
    ActivePath, CommandKind, EventKind, KbdStateV2, Phase, Result, RuntimeError, WorkStatus,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const DISPOSITION_SCHEMA: &str = "kbd.phase-disposition/v1";
pub const SCENARIO_PASS: &str = "PASS";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PhaseDisposition {
    Pass,
    Fail,
    Blocked,
    Cancelled,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContractBindingInput {
    pub contract_path: String,
    pub contract_digest: String,
    pub scenario_digests: BTreeMap<String, String>,
    pub plan_revision: u64,
    pub adoption_decision_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContractBinding {
    pub contract_path: String,
    pub contract_digest: String,
    pub scenario_digests: BTreeMap<String, String>,
    pub plan_revision: u64,
    pub adoption_decision_id: String,
    pub binding_revision: u32,
    /// Scenarios added or modified by the amendment that produced this
    /// binding; only these require fresh evidence.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub amended_scenarios: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<ScopeAuthorityRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioOutcome {
    pub id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried_from_contract: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseCloseInput {
    pub run_id: String,
    pub contract_digest: String,
    pub evidence_digest: String,
    pub disposition: PhaseDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub scenarios: Vec<ScenarioOutcome>,
    #[serde(default)]
    pub budget_consumption: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseClosure {
    #[serde(flatten)]
    pub input: PhaseCloseInput,
    pub closure_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordedClosure {
    #[serde(flatten)]
    pub closure: PhaseClosure,
    pub recorded_at_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuccessorRecord {
    pub phase_id: String,
    pub manifest_sha256: String,
    pub started_at_revision: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseClosureState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<ContractBinding>,
    #[serde(default)]
    pub bound_at_revision: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_history: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<RecordedClosure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refusal: Option<RecordedClosure>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub consumption_high_water: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor: Option<SuccessorRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseClosureRegistry {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub phases: BTreeMap<String, PhaseClosureState>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub consumed_manifests: BTreeSet<String>,
    /// A bound phase closed while active; until an authorized successor
    /// starts, only an ancestor may become active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awaiting_successor: Option<String>,
}

impl PhaseClosureRegistry {
    pub fn is_empty(&self) -> bool {
        self.phases.is_empty()
            && self.consumed_manifests.is_empty()
            && self.awaiting_successor.is_none()
    }

    fn get(&self, phase_id: &str) -> Option<&PhaseClosureState> {
        self.phases.get(phase_id)
    }

    fn is_bound(&self, phase_id: &str) -> bool {
        self.get(phase_id).is_some_and(|s| s.binding.is_some())
    }

    fn terminal(&self, phase_id: &str) -> Option<&RecordedClosure> {
        self.get(phase_id).and_then(|s| s.terminal.as_ref())
    }
}

/// The single interpretation every reader uses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDispositionView {
    pub schema: String,
    pub project_id: String,
    pub run_id: String,
    pub phase_id: String,
    pub phase_status: WorkStatus,
    pub contract_bound: bool,
    pub disposition: String,
    pub binding: Option<ContractBinding>,
    pub closure: Option<RecordedClosure>,
    pub last_refusal: Option<RecordedClosure>,
    pub successor: Option<SuccessorRecord>,
    pub consumption_high_water: BTreeMap<String, u64>,
    pub awaiting_successor: bool,
    pub dispatch_allowed: bool,
    pub successor_allowed: bool,
}

fn refused(phase_id: &str, reason: impl Into<String>) -> RuntimeError {
    RuntimeError::PhaseCloseRefused {
        phase_id: phase_id.to_string(),
        reason: reason.into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Deterministic idempotency key for one disposition of one phase.
pub fn closure_key(project_id: &str, phase_id: &str, input: &PhaseCloseInput) -> Result<String> {
    let identity = serde_json::json!({
        "projectId": project_id,
        "runId": input.run_id,
        "phaseId": phase_id,
        "contractDigest": input.contract_digest,
        "evidenceDigest": input.evidence_digest,
        "disposition": input.disposition,
        "reasonCode": input.reason_code,
    });
    let bytes = serde_jcs::to_vec(&identity)
        .map_err(|error| RuntimeError::InvalidState(error.to_string()))?;
    Ok(sha256_hex(&bytes))
}

pub fn scenario_diff(
    previous: &BTreeMap<String, String>,
    next: &BTreeMap<String, String>,
) -> ScenarioChanges {
    ScenarioChanges {
        added: next
            .keys()
            .filter(|id| !previous.contains_key(*id))
            .cloned()
            .collect(),
        modified: next
            .iter()
            .filter(|(id, digest)| previous.get(*id).is_some_and(|old| old != *digest))
            .map(|(id, _)| id.clone())
            .collect(),
        removed: previous
            .keys()
            .filter(|id| !next.contains_key(*id))
            .cloned()
            .collect(),
    }
}

// ---------------------------------------------------------------- admission

pub(crate) fn prepare_bind(
    state: &KbdStateV2,
    phase_id: &str,
    input: &ContractBindingInput,
    authority: Option<&ScopeAuthorityInput>,
) -> Result<EventKind> {
    let existing = state
        .phase_closure
        .get(phase_id)
        .and_then(|s| s.binding.as_ref());
    let binding = match (existing, authority) {
        (None, Some(_)) => {
            return Err(refused(phase_id, "the first contract binding needs no amendment authority"))
        }
        (None, None) => new_binding(input, 1, BTreeSet::new(), None),
        (Some(_), None) => {
            return Err(RuntimeError::ScopeAuthority(format!(
                "phase {phase_id} is bound to a different contract digest; a material amendment requires a signed amend-contract manifest"
            )))
        }
        (Some(previous), Some(authority)) => {
            let (_, record) = scope_authority::verify(authority)?;
            let changes = scenario_diff(&previous.scenario_digests, &input.scenario_digests);
            let amended = changes.added.iter().chain(&changes.modified).cloned().collect();
            new_binding(input, previous.binding_revision + 1, amended, Some(record))
        }
    };
    Ok(EventKind::PhaseContractBound {
        phase_id: phase_id.to_string(),
        binding: Box::new(binding),
    })
}

fn new_binding(
    input: &ContractBindingInput,
    binding_revision: u32,
    amended_scenarios: BTreeSet<String>,
    authority: Option<ScopeAuthorityRecord>,
) -> ContractBinding {
    ContractBinding {
        contract_path: input.contract_path.clone(),
        contract_digest: input.contract_digest.clone(),
        scenario_digests: input.scenario_digests.clone(),
        plan_revision: input.plan_revision,
        adoption_decision_id: input.adoption_decision_id.clone(),
        binding_revision,
        amended_scenarios,
        authority,
    }
}

pub(crate) fn prepare_close(
    state: &KbdStateV2,
    phase_id: &str,
    input: &PhaseCloseInput,
) -> Result<EventKind> {
    Ok(EventKind::PhaseDispositionRecorded {
        phase_id: phase_id.to_string(),
        closure: Box::new(PhaseClosure {
            closure_key: closure_key(&state.project_id, phase_id, input)?,
            input: input.clone(),
        }),
    })
}

pub(crate) fn prepare_successor(
    state: &KbdStateV2,
    predecessor_phase_id: &str,
    successor_phase_id: &str,
    title: &str,
    slug: Option<&str>,
    exact_next_work: Option<&String>,
    authority: &ScopeAuthorityInput,
) -> Result<EventKind> {
    let predecessor =
        state
            .phases
            .get(predecessor_phase_id)
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "phase",
                id: predecessor_phase_id.to_string(),
            })?;
    let (_, record) = scope_authority::verify(authority)?;
    let phase = Phase {
        id: successor_phase_id.to_string(),
        slug: slug.unwrap_or(successor_phase_id).to_string(),
        title: title.to_string(),
        parent_phase_id: predecessor.parent_phase_id.clone(),
        status: WorkStatus::InProgress,
        stages: BTreeMap::new(),
        changes: BTreeMap::new(),
        legacy_completion_baseline: None,
        legacy_read_only: false,
    };
    let mut phase_path = match &predecessor.parent_phase_id {
        Some(parent) => state.phase_chain(parent)?,
        None => Vec::new(),
    };
    phase_path.push(successor_phase_id.to_string());
    Ok(EventKind::PhaseSuccessorStarted {
        predecessor_phase_id: predecessor_phase_id.to_string(),
        phase: Box::new(phase),
        active_path: Box::new(ActivePath {
            phase_path,
            phase_id: Some(successor_phase_id.to_string()),
            ..Default::default()
        }),
        exact_next_work: exact_next_work.cloned(),
        authority: Box::new(record),
    })
}

/// Semantic idempotency for the closure commands: `Some(revision)` means the
/// identical outcome is already recorded and nothing must be appended.
pub fn idempotent_replay(state: &KbdStateV2, command: &CommandKind) -> Result<Option<u64>> {
    match command {
        CommandKind::PhaseClose { phase_id, closure } => {
            let key = closure_key(&state.project_id, phase_id, closure)?;
            let Some(entry) = state.phase_closure.get(phase_id) else {
                return Ok(None);
            };
            if let Some(terminal) = &entry.terminal {
                if terminal.closure.closure_key == key {
                    return Ok(Some(terminal.recorded_at_revision));
                }
                return Err(RuntimeError::PhaseAlreadyClosed {
                    phase_id: phase_id.clone(),
                    existing_key: terminal.closure.closure_key.clone(),
                });
            }
            Ok(entry
                .last_refusal
                .as_ref()
                .filter(|refusal| refusal.closure.closure_key == key)
                .map(|refusal| refusal.recorded_at_revision))
        }
        CommandKind::PhaseContractBind {
            phase_id, binding, ..
        } => Ok(state
            .phase_closure
            .get(phase_id)
            .filter(|entry| {
                entry
                    .binding
                    .as_ref()
                    .is_some_and(|b| b.contract_digest == binding.contract_digest)
            })
            .map(|entry| entry.bound_at_revision)),
        CommandKind::PhaseSuccessorStart {
            predecessor_phase_id,
            authority,
            ..
        } => {
            let Some(successor) = state
                .phase_closure
                .get(predecessor_phase_id)
                .and_then(|entry| entry.successor.as_ref())
            else {
                return Ok(None);
            };
            if successor.manifest_sha256 == sha256_hex(authority.manifest_raw.as_bytes()) {
                return Ok(Some(successor.started_at_revision));
            }
            Err(refused(
                predecessor_phase_id,
                format!("a successor ({}) was already started", successor.phase_id),
            ))
        }
        _ => Ok(None),
    }
}

pub fn is_closure_command(command: &CommandKind) -> bool {
    matches!(
        command,
        CommandKind::PhaseClose { .. }
            | CommandKind::PhaseContractBind { .. }
            | CommandKind::PhaseSuccessorStart { .. }
    )
}

/// Command-time refusal for phases defined directly in a terminal state.
pub(crate) fn admit_phase_define(phase: &Phase) -> Result<()> {
    if phase.status.is_terminal() {
        return Err(RuntimeError::InvalidState(format!(
            "phase {} cannot be defined in terminal status {:?}",
            phase.id, phase.status
        )));
    }
    Ok(())
}

// ------------------------------------------------------------------- guards

pub(crate) fn guard_phase_transition(
    state: &KbdStateV2,
    phase_id: &str,
    to: &WorkStatus,
) -> Result<()> {
    if let Some(terminal) = state.phase_closure.terminal(phase_id) {
        return Err(RuntimeError::PhaseAlreadyClosed {
            phase_id: phase_id.to_string(),
            existing_key: terminal.closure.closure_key.clone(),
        });
    }
    if state.phase_closure.is_bound(phase_id) && to.is_terminal() {
        return Err(RuntimeError::ContractBoundPhase {
            phase_id: phase_id.to_string(),
            operation: format!(
                "transition to {to:?}; a contract-bound phase closes only through `phase close`"
            ),
        });
    }
    Ok(())
}

/// Refuses new work (registration, entry, or re-opening transitions) on a
/// terminally closed phase. Bookkeeping transitions to a terminal status
/// stay allowed.
pub(crate) fn guard_work(
    state: &KbdStateV2,
    phase_id: &str,
    to: Option<&WorkStatus>,
) -> Result<()> {
    let creates_work = to.is_none_or(|status| !status.is_terminal());
    match state.phase_closure.terminal(phase_id) {
        Some(terminal) if creates_work => Err(RuntimeError::PhaseAlreadyClosed {
            phase_id: phase_id.to_string(),
            existing_key: terminal.closure.closure_key.clone(),
        }),
        _ => Ok(()),
    }
}

/// Active-path admission. Call after the structural validation passed; it
/// may clear `awaiting_successor` when control returns to an ancestor.
pub(crate) fn admit_active_path(state: &mut KbdStateV2, path: &ActivePath) -> Result<()> {
    if let Some(target) = path.phase_id.as_deref() {
        if let Some(terminal) = state.phase_closure.terminal(target) {
            return Err(RuntimeError::PhaseAlreadyClosed {
                phase_id: target.to_string(),
                existing_key: terminal.closure.closure_key.clone(),
            });
        }
    }
    let Some(closed) = state.phase_closure.awaiting_successor.clone() else {
        return Ok(());
    };
    let ancestors = strict_ancestors(state, &closed);
    match path.phase_id.as_deref() {
        Some(target) if ancestors.contains(target) => {
            state.phase_closure.awaiting_successor = None;
            Ok(())
        }
        _ => Err(RuntimeError::ContractBoundPhase {
            phase_id: closed,
            operation: "activate another phase; a closed phase awaits a signed start-successor authorization (or activate an ancestor)".into(),
        }),
    }
}

fn strict_ancestors(state: &KbdStateV2, phase_id: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut cursor = state
        .phases
        .get(phase_id)
        .and_then(|p| p.parent_phase_id.clone());
    while let Some(id) = cursor {
        if !seen.insert(id.clone()) {
            break;
        }
        cursor = state
            .phases
            .get(&id)
            .and_then(|p| p.parent_phase_id.clone());
    }
    seen
}

// -------------------------------------------------------------------- apply

pub(crate) fn apply_bound(
    state: &mut KbdStateV2,
    revision: u64,
    phase_id: &str,
    binding: &ContractBinding,
) -> Result<()> {
    let phase = state
        .phases
        .get(phase_id)
        .ok_or_else(|| RuntimeError::WorkItemNotFound {
            kind: "phase",
            id: phase_id.to_string(),
        })?;
    if phase.status.is_terminal() {
        return Err(refused(
            phase_id,
            "a phase in a terminal status cannot be bound",
        ));
    }
    validate_binding_shape(phase_id, binding)?;
    let entry = state
        .phase_closure
        .phases
        .get(phase_id)
        .cloned()
        .unwrap_or_default();
    if entry.terminal.is_some() {
        return Err(refused(phase_id, "a closed phase cannot be re-bound"));
    }
    match &entry.binding {
        None => {
            if binding.binding_revision != 1
                || binding.authority.is_some()
                || !binding.amended_scenarios.is_empty()
            {
                return Err(refused(
                    phase_id,
                    "the first binding has revision 1 and no amendment",
                ));
            }
        }
        Some(previous) => {
            if previous.contract_digest == binding.contract_digest {
                return Err(refused(phase_id, "contract digest is already bound"));
            }
            let record = binding.authority.as_ref().ok_or_else(|| {
                RuntimeError::ScopeAuthority("amendment requires a signed manifest".into())
            })?;
            let manifest = scope_authority::verify_record(record)?;
            check_amendment(state, phase_id, previous, binding, &manifest)?;
            if state
                .phase_closure
                .consumed_manifests
                .contains(&record.manifest_sha256)
            {
                return Err(RuntimeError::ScopeAuthority(
                    "scope manifest was already used".into(),
                ));
            }
        }
    }
    let mut entry = entry;
    if let Some(previous) = entry.binding.take() {
        entry.binding_history.push(previous.contract_digest);
    }
    if let Some(record) = &binding.authority {
        state
            .phase_closure
            .consumed_manifests
            .insert(record.manifest_sha256.clone());
    }
    entry.binding = Some(binding.clone());
    entry.bound_at_revision = revision;
    state
        .phase_closure
        .phases
        .insert(phase_id.to_string(), entry);
    Ok(())
}

fn validate_binding_shape(phase_id: &str, binding: &ContractBinding) -> Result<()> {
    if binding.contract_path.trim().is_empty()
        || binding.adoption_decision_id.trim().is_empty()
        || !is_digest(&binding.contract_digest)
        || binding.scenario_digests.is_empty()
        || binding
            .scenario_digests
            .iter()
            .any(|(id, digest)| id.trim().is_empty() || !is_digest(digest))
    {
        return Err(refused(
            phase_id,
            "binding requires a contract path, adoption decision id, 64-hex contract digest and per-scenario digests",
        ));
    }
    Ok(())
}

fn check_amendment(
    state: &KbdStateV2,
    phase_id: &str,
    previous: &ContractBinding,
    next: &ContractBinding,
    manifest: &ScopeAuthorityManifest,
) -> Result<()> {
    let expected = scenario_diff(&previous.scenario_digests, &next.scenario_digests);
    let amended: BTreeSet<String> = expected
        .added
        .iter()
        .chain(&expected.modified)
        .cloned()
        .collect();
    let checks = [
        (
            manifest.action == ScopeAction::AmendContract,
            "action must be amend-contract",
        ),
        (
            manifest.project_id == state.project_id,
            "projectId does not match this project",
        ),
        (
            manifest.run_id == state.run_id,
            "runId does not match the current run",
        ),
        (manifest.phase_id == phase_id, "phaseId does not match"),
        (
            manifest.contract_path.as_deref() == Some(next.contract_path.as_str()),
            "contractPath does not match the new binding",
        ),
        (
            manifest.previous_contract_digest.as_deref() == Some(previous.contract_digest.as_str()),
            "previousContractDigest is not the currently bound digest",
        ),
        (
            manifest.new_contract_digest.as_deref() == Some(next.contract_digest.as_str()),
            "newContractDigest does not match the new binding",
        ),
        (
            manifest
                .scenario_changes
                .as_ref()
                .map(ScenarioChanges::normalized)
                == Some(expected),
            "scenarioChanges does not equal the computed scenario diff",
        ),
        (
            next.binding_revision == previous.binding_revision + 1,
            "binding revision must increase by one",
        ),
        (
            next.amended_scenarios == amended,
            "amended scenarios do not match the diff",
        ),
    ];
    match checks.iter().find(|(ok, _)| !ok) {
        Some((_, reason)) => Err(RuntimeError::ScopeAuthority(format!(
            "amendment rejected: {reason}"
        ))),
        None => Ok(()),
    }
}

pub(crate) fn apply_disposition(
    state: &mut KbdStateV2,
    revision: u64,
    phase_id: &str,
    closure: &PhaseClosure,
) -> Result<()> {
    let input = &closure.input;
    let entry = state
        .phase_closure
        .phases
        .get(phase_id)
        .cloned()
        .unwrap_or_default();
    let binding = entry.binding.clone().ok_or_else(|| {
        refused(
            phase_id,
            "phase is not contract-bound; adopt its contract first",
        )
    })?;
    if let Some(terminal) = &entry.terminal {
        return Err(RuntimeError::PhaseAlreadyClosed {
            phase_id: phase_id.to_string(),
            existing_key: terminal.closure.closure_key.clone(),
        });
    }
    if input.run_id != state.run_id {
        return Err(refused(
            phase_id,
            format!("runId {} is not the current run", input.run_id),
        ));
    }
    if input.contract_digest != binding.contract_digest {
        return Err(refused(
            phase_id,
            "contractDigest is not the bound contract digest",
        ));
    }
    if closure.closure_key != closure_key(&state.project_id, phase_id, input)? {
        return Err(refused(
            phase_id,
            "closureKey does not match the closure identity",
        ));
    }
    let mut high_water = entry.consumption_high_water.clone();
    // A refusal only raises the high-water mark. A PASS must report
    // cumulative consumption at or above every recorded mark, so a success
    // can never be claimed against reset counters.
    let claims_pass = input.disposition == PhaseDisposition::Pass;
    for (dimension, floor) in &entry.consumption_high_water {
        match input.budget_consumption.get(dimension) {
            Some(value) if value >= floor => {}
            _ if !claims_pass => {}
            Some(_) => {
                return Err(refused(
                    phase_id,
                    format!("budget-regressed({dimension}: recorded consumption {floor} cannot decrease)"),
                ))
            }
            None => {
                return Err(refused(
                    phase_id,
                    format!("budget-unreported({dimension}: PASS must report cumulative consumption >= {floor})"),
                ))
            }
        }
    }
    for (dimension, value) in &input.budget_consumption {
        let slot = high_water.entry(dimension.clone()).or_default();
        *slot = (*slot).max(*value);
    }
    let recorded = RecordedClosure {
        closure: closure.clone(),
        recorded_at_revision: revision,
    };
    let mut entry = entry;
    entry.consumption_high_water = high_water;
    match input.disposition {
        PhaseDisposition::Pass => {
            require_pass_evidence(phase_id, &binding, &input.scenarios)?;
            let was_active = state.active_path.phase_id.as_deref() == Some(phase_id);
            let phase = state.phase_mut(phase_id)?;
            if !matches!(phase.status, WorkStatus::InProgress | WorkStatus::Blocked) {
                return Err(refused(
                    phase_id,
                    format!("phase status {:?} cannot close; it must be in progress or blocked", phase.status),
                ));
            }
            phase.status = WorkStatus::Complete;
            entry.terminal = Some(recorded);
            if was_active {
                state.phase_closure.awaiting_successor = Some(phase_id.to_string());
            }
        }
        PhaseDisposition::Fail | PhaseDisposition::Blocked => {
            if input.reason_code.as_deref().is_none_or(str::is_empty) {
                return Err(refused(phase_id, "a refusal requires a reasonCode"));
            }
            entry.last_refusal = Some(recorded);
        }
        PhaseDisposition::Cancelled | PhaseDisposition::Deferred => {
            return Err(refused(
                phase_id,
                "cancelled/deferred dispositions require an authorized path this runtime does not provide",
            ))
        }
    }
    state
        .phase_closure
        .phases
        .insert(phase_id.to_string(), entry);
    Ok(())
}

fn require_pass_evidence(
    phase_id: &str,
    binding: &ContractBinding,
    outcomes: &[ScenarioOutcome],
) -> Result<()> {
    for (scenario, scenario_digest) in &binding.scenario_digests {
        let prefix = format!("{scenario}-");
        let matched: Vec<&ScenarioOutcome> = outcomes
            .iter()
            .filter(|o| o.id == *scenario || o.id.starts_with(&prefix))
            .collect();
        if matched.is_empty() {
            return Err(refused(phase_id, format!("missing-scenario:{scenario}")));
        }
        for outcome in matched {
            if outcome.status != SCENARIO_PASS {
                return Err(refused(
                    phase_id,
                    format!("not-pass:{}({})", outcome.id, outcome.status),
                ));
            }
            if outcome
                .scenario_digest
                .as_deref()
                .is_some_and(|digest| digest != scenario_digest)
            {
                return Err(refused(
                    phase_id,
                    format!("scenario-digest-mismatch:{}", outcome.id),
                ));
            }
            if outcome.carried_from_contract.is_some()
                && (binding.amended_scenarios.contains(scenario)
                    || outcome.scenario_digest.as_deref() != Some(scenario_digest))
            {
                return Err(refused(
                    phase_id,
                    format!(
                        "carried-evidence-invalid:{} (amended or unbound scenario)",
                        outcome.id
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_successor(
    state: &mut KbdStateV2,
    revision: u64,
    predecessor_phase_id: &str,
    phase: &Phase,
    active_path: &ActivePath,
    exact_next_work: &Option<String>,
    authority: &ScopeAuthorityRecord,
) -> Result<()> {
    let entry = state
        .phase_closure
        .phases
        .get(predecessor_phase_id)
        .cloned()
        .unwrap_or_default();
    let terminal = entry
        .terminal
        .as_ref()
        .filter(|t| t.closure.input.disposition == PhaseDisposition::Pass)
        .ok_or_else(|| {
            refused(
                predecessor_phase_id,
                "a successor requires a PASS closure of its predecessor",
            )
        })?;
    if let Some(existing) = &entry.successor {
        return Err(refused(
            predecessor_phase_id,
            format!("a successor ({}) was already started", existing.phase_id),
        ));
    }
    let manifest = scope_authority::verify_record(authority)?;
    let predecessor_parent = state
        .phases
        .get(predecessor_phase_id)
        .and_then(|p| p.parent_phase_id.clone());
    let checks = [
        (
            manifest.action == ScopeAction::StartSuccessor,
            "action must be start-successor",
        ),
        (
            manifest.project_id == state.project_id,
            "projectId does not match this project",
        ),
        (
            manifest.run_id == state.run_id,
            "runId does not match the current run",
        ),
        (
            manifest.phase_id == predecessor_phase_id,
            "phaseId must name the closed predecessor",
        ),
        (
            manifest.predecessor_closure_key.as_deref()
                == Some(terminal.closure.closure_key.as_str()),
            "predecessorClosureKey does not match the recorded closure",
        ),
        (
            manifest.successor_phase_id.as_deref() == Some(phase.id.as_str()),
            "successorPhaseId does not match",
        ),
        (
            manifest.successor_title.as_deref() == Some(phase.title.as_str()),
            "successorTitle does not match",
        ),
        (
            !state
                .phase_closure
                .consumed_manifests
                .contains(&authority.manifest_sha256),
            "scope manifest was already used",
        ),
        (
            phase.parent_phase_id == predecessor_parent,
            "successor must share the predecessor's parent",
        ),
        (
            phase.status == WorkStatus::InProgress,
            "successor starts in progress",
        ),
        (
            !state.phases.contains_key(&phase.id),
            "successor phase id already exists",
        ),
    ];
    if let Some((_, reason)) = checks.iter().find(|(ok, _)| !ok) {
        return Err(RuntimeError::ScopeAuthority(format!(
            "successor rejected: {reason}"
        )));
    }
    state.phase_definition_order.push(phase.id.clone());
    state.phases.insert(phase.id.clone(), phase.clone());
    state.validate_active_path(active_path)?;
    let mut entry = entry;
    entry.successor = Some(SuccessorRecord {
        phase_id: phase.id.clone(),
        manifest_sha256: authority.manifest_sha256.clone(),
        started_at_revision: revision,
    });
    state
        .phase_closure
        .phases
        .insert(predecessor_phase_id.to_string(), entry);
    state
        .phase_closure
        .consumed_manifests
        .insert(authority.manifest_sha256.clone());
    if state.phase_closure.awaiting_successor.as_deref() == Some(predecessor_phase_id) {
        state.phase_closure.awaiting_successor = None;
    }
    state.active_path.clone_from(active_path);
    state.synchronize_active_work_cursor(&phase.id);
    state.exact_next_work.clone_from(exact_next_work);
    Ok(())
}

// --------------------------------------------------------------------- view

pub fn disposition_view(state: &KbdStateV2, phase_id: &str) -> Result<PhaseDispositionView> {
    let phase = state
        .phases
        .get(phase_id)
        .ok_or_else(|| RuntimeError::WorkItemNotFound {
            kind: "phase",
            id: phase_id.to_string(),
        })?;
    let entry = state
        .phase_closure
        .get(phase_id)
        .cloned()
        .unwrap_or_default();
    let token = match (&entry.binding, &entry.terminal, &entry.last_refusal) {
        (None, _, _) => "UNBOUND",
        (Some(_), Some(_), _) => "CLOSED",
        (Some(_), None, Some(refusal))
            if refusal.closure.input.disposition == PhaseDisposition::Fail =>
        {
            "REFUSED_FAIL"
        }
        (Some(_), None, Some(_)) => "REFUSED_BLOCKED",
        (Some(_), None, None) => "OPEN",
    };
    let closed = token == "CLOSED";
    Ok(PhaseDispositionView {
        schema: DISPOSITION_SCHEMA.into(),
        project_id: state.project_id.clone(),
        run_id: state.run_id.clone(),
        phase_id: phase_id.to_string(),
        phase_status: phase.status.clone(),
        contract_bound: entry.binding.is_some(),
        disposition: token.into(),
        successor_allowed: closed && entry.successor.is_none(),
        dispatch_allowed: !closed,
        awaiting_successor: state.phase_closure.awaiting_successor.as_deref() == Some(phase_id),
        binding: entry.binding,
        closure: entry.terminal,
        last_refusal: entry.last_refusal,
        successor: entry.successor,
        consumption_high_water: entry.consumption_high_water,
    })
}
