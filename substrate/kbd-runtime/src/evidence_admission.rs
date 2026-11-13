//! Evidence admission for canonical phase closure (issue #219).
//!
//! The close command carries the evidence facts the shell evaluator checks:
//! the phase and project the evidence names, its source digests, and each
//! scenario entry's observation fields. A PASS closure is admitted only when
//! those facts hold.
//!
//! Two layers, split so replay stays pure:
//! * the fact checks (`check_identity`, `check_sources_declared`,
//!   `check_observations`) read only the event and the state; the reducer
//!   runs them for events stamped [`EVIDENCE_ADMISSION_VERSION`] or later;
//! * source freshness (`admit_close`) hashes the files once, at command
//!   admission on a host that has the project tree. The reducer never reads
//!   files, so a later edit cannot change how a recorded closure replays.
//!
//! Events without the stamp were written before this change and replay with
//! the semantics they were recorded under.

use crate::phase_closure::{PhaseCloseInput, PhaseDisposition, ScenarioOutcome};
use crate::{KbdStateV2, Result, RuntimeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path};

/// Stamped by the runtime on every disposition event it admits. Absent on
/// events written before evidence admission existed.
pub const EVIDENCE_ADMISSION_VERSION: u32 = 2;

/// Evidence-file facts passed through unchanged by the close adapter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceFacts {
    #[serde(default)]
    pub phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Project-relative path to the sha256 the evidence was produced against.
    #[serde(default)]
    pub source_digests: BTreeMap<String, String>,
}

/// What the admitting host found at one declared source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceObservation {
    Digest(String),
    Missing,
    /// The host has no project tree to hash (filesystem-free replica).
    Unverifiable,
}

pub fn enforces_evidence(admission_version: Option<u32>) -> bool {
    admission_version.is_some_and(|version| version >= EVIDENCE_ADMISSION_VERSION)
}

fn refused(phase_id: &str, reason: String) -> RuntimeError {
    RuntimeError::PhaseCloseRefused {
        phase_id: phase_id.to_string(),
        reason,
    }
}

fn facts(input: &PhaseCloseInput) -> EvidenceFacts {
    input.evidence_facts.clone().unwrap_or_default()
}

/// The evidence names the phase being closed and, when it names a project,
/// this project.
pub(crate) fn check_identity(
    project_id: &str,
    phase_id: &str,
    input: &PhaseCloseInput,
) -> Result<()> {
    let facts = facts(input);
    if facts.phase != phase_id {
        return Err(refused(
            phase_id,
            format!(
                "evidence-phase-mismatch(evidence={} gate={phase_id})",
                facts.phase
            ),
        ));
    }
    match facts.project_id.as_deref().filter(|id| !id.is_empty()) {
        Some(supplied) if supplied != project_id => Err(refused(
            phase_id,
            format!("evidence-project-mismatch(evidence={supplied} canonical={project_id})"),
        )),
        _ => Ok(()),
    }
}

pub(crate) fn check_sources_declared(phase_id: &str, input: &PhaseCloseInput) -> Result<()> {
    if facts(input).source_digests.is_empty() {
        return Err(refused(phase_id, "evidence-source-digests-missing".into()));
    }
    Ok(())
}

/// Mirrors the evaluator's `(.field // "" | length) == 0`.
fn is_blank(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null | Value::Bool(false)) => true,
        Some(Value::Bool(true)) => false,
        Some(Value::String(text)) => text.is_empty(),
        Some(Value::Array(items)) => items.is_empty(),
        Some(Value::Object(fields)) => fields.is_empty(),
        Some(Value::Number(number)) => number.as_f64() == Some(0.0),
    }
}

fn owns(scenario: &str, outcome_id: &str) -> bool {
    outcome_id == scenario
        || outcome_id
            .strip_prefix(scenario)
            .is_some_and(|rest| rest.starts_with('-'))
}

/// Every bound scenario has at least one entry (`S` or `S-<sub>`), and every
/// entry owned by a bound scenario carries its observation fields. Entries
/// for scenarios outside the bound contract are not examined.
pub(crate) fn check_observations<'a>(
    phase_id: &str,
    bound_scenarios: impl Iterator<Item = &'a String> + Clone,
    outcomes: &[ScenarioOutcome],
) -> Result<()> {
    if let Some(scenario) = bound_scenarios
        .clone()
        .find(|scenario| !outcomes.iter().any(|outcome| owns(scenario, &outcome.id)))
    {
        return Err(refused(phase_id, format!("missing-scenario:{scenario}")));
    }
    let incomplete = outcomes.iter().find(|outcome| {
        bound_scenarios
            .clone()
            .any(|scenario| owns(scenario, &outcome.id))
            && [&outcome.attempted, &outcome.observed, &outcome.evidence]
                .iter()
                .any(|field| is_blank(field.as_ref()))
    });
    match incomplete {
        Some(outcome) => Err(refused(phase_id, format!("empty-evidence:{}", outcome.id))),
        None => Ok(()),
    }
}

/// Command-time admission of a PASS closure, in the evaluator's order:
/// identity, declared sources, then source freshness through `observe`.
/// Conditions the reducer already reports (unbound, closed, run or contract
/// mismatch) are left to it so their existing refusals are unchanged.
pub(crate) fn admit_close(
    state: &KbdStateV2,
    phase_id: &str,
    input: &PhaseCloseInput,
    observe: &dyn Fn(&str) -> SourceObservation,
) -> Result<()> {
    if input.disposition != PhaseDisposition::Pass {
        return Ok(());
    }
    let Some(entry) = state.phase_closure.phases.get(phase_id) else {
        return Ok(());
    };
    let Some(binding) = entry.binding.as_ref().filter(|_| entry.terminal.is_none()) else {
        return Ok(());
    };
    check_identity(&state.project_id, phase_id, input)?;
    if input.run_id != state.run_id || input.contract_digest != binding.contract_digest {
        return Ok(());
    }
    check_sources_declared(phase_id, input)?;
    let stale: Vec<String> = facts(input)
        .source_digests
        .iter()
        .filter_map(|(path, recorded)| match observe(path) {
            SourceObservation::Digest(current) if current == *recorded => None,
            SourceObservation::Digest(_) => Some(format!("stale({path})")),
            SourceObservation::Missing => Some(format!("missing({path})")),
            SourceObservation::Unverifiable => Some(format!("unverifiable({path})")),
        })
        .collect();
    if stale.is_empty() {
        Ok(())
    } else {
        Err(refused(
            phase_id,
            format!("evidence-stale-sources({})", stale.join(",")),
        ))
    }
}

/// Hash one declared source under the project root. Paths that leave the
/// root (absolute, `..`) are reported missing rather than read.
pub(crate) fn observe_source(project_root: &Path, relative: &str) -> SourceObservation {
    let candidate = Path::new(relative);
    let confined = !relative.is_empty()
        && candidate
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
    if !confined {
        return SourceObservation::Missing;
    }
    match std::fs::read(project_root.join(candidate)) {
        Ok(bytes) => SourceObservation::Digest(format!("{:x}", Sha256::digest(&bytes))),
        Err(_) => SourceObservation::Missing,
    }
}
