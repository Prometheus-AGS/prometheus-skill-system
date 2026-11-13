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
//! * source freshness and the bound budget ceilings (`admit_close`) read the
//!   project tree once, at command admission on a host that has it. The
//!   reducer never reads files, so a later edit cannot change how a recorded
//!   closure replays.
//!
//! Events without the stamp were written before this change and replay with
//! the semantics they were recorded under.

use crate::phase_closure::{self, PhaseCloseInput, PhaseDisposition, ScenarioOutcome};
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
    /// `budgetConsumption.source`: absent when the evidence reports no
    /// budget consumption at all, empty when it names no source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_source: Option<String>,
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

/// Enforced budget dimension in the evidence, and its ceiling key in the
/// bound contract snapshot.
const ENFORCED_BUDGETS: [(&str, &str); 4] = [
    ("noProgress", "noProgressLimit"),
    ("repairAttempts", "repairAttempts"),
    ("timeMinutes", "timeMinutes"),
    ("toolCalls", "toolCalls"),
];
const NO_PROGRESS_CEILING: &str = "noProgressLimit";

/// Command-time admission of a PASS closure, in the evaluator's order: bound
/// snapshot, identity, declared sources, source freshness, scenario
/// evidence, budget ceilings. `project_root` is `None` on a host without the
/// project tree, which therefore cannot admit a PASS closure.
/// Conditions the reducer already reports (unbound, closed, run or contract
/// mismatch) are left to it so their existing refusals are unchanged.
pub(crate) fn admit_close(
    state: &KbdStateV2,
    phase_id: &str,
    input: &PhaseCloseInput,
    project_root: Option<&Path>,
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
    let snapshot = project_root
        .map(|root| bound_snapshot(root, phase_id, &binding.contract_digest))
        .transpose()?;
    check_identity(&state.project_id, phase_id, input)?;
    if input.run_id != state.run_id || input.contract_digest != binding.contract_digest {
        return Ok(());
    }
    check_sources_declared(phase_id, input)?;
    let stale: Vec<String> = facts(input)
        .source_digests
        .iter()
        .filter_map(|(path, recorded)| {
            let observed = project_root.map_or(SourceObservation::Unverifiable, |root| {
                observe_source(root, path)
            });
            match observed {
                SourceObservation::Digest(current) if current == *recorded => None,
                SourceObservation::Digest(_) => Some(format!("stale({path})")),
                SourceObservation::Missing => Some(format!("missing({path})")),
                SourceObservation::Unverifiable => Some(format!("unverifiable({path})")),
            }
        })
        .collect();
    if !stale.is_empty() {
        return Err(refused(
            phase_id,
            format!("evidence-stale-sources({})", stale.join(",")),
        ));
    }
    check_observations(phase_id, binding.scenario_digests.keys(), &input.scenarios)?;
    phase_closure::require_pass_evidence(phase_id, binding, &input.scenarios)?;
    match snapshot {
        Some(snapshot) => check_budget(phase_id, input, &snapshot),
        None => Ok(()),
    }
}

fn read_confined(project_root: &Path, relative: &str) -> Option<Vec<u8>> {
    let candidate = Path::new(relative);
    let confined = !relative.is_empty()
        && candidate
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
    confined
        .then(|| std::fs::read(project_root.join(candidate)).ok())
        .flatten()
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Hash one declared source under the project root. Paths that leave the
/// root (absolute, `..`) are reported missing rather than read.
fn observe_source(project_root: &Path, relative: &str) -> SourceObservation {
    read_confined(project_root, relative).map_or(SourceObservation::Missing, |bytes| {
        SourceObservation::Digest(sha256_hex(&bytes))
    })
}

/// The adopted contract snapshot the binding names. It defines the budget
/// ceilings; the mutable working contract is never read.
fn bound_snapshot(project_root: &Path, phase_id: &str, contract_digest: &str) -> Result<Value> {
    let relative = format!(".kbd-orchestrator/phases/{phase_id}/adopted/{contract_digest}.json");
    read_confined(project_root, &relative)
        .filter(|bytes| sha256_hex(bytes) == contract_digest)
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| {
            refused(
                phase_id,
                format!(
                    "snapshot-invalid({relative} does not hash to the bound digest {contract_digest})"
                ),
            )
        })
}

/// Base budget plus additive extensions; the latest extension's explicit
/// `resultingCeilings` wins when present. A missing ceiling is 0.
fn ceiling(snapshot: &Value, key: &str) -> f64 {
    let budgets = &snapshot["budgets"];
    let extensions = budgets["extensions"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let number = |value: &Value| value.as_f64().unwrap_or(0.0);
    let resulting = extensions
        .iter()
        .rev()
        .map(|extension| &extension["resultingCeilings"])
        .find(|ceilings| !ceilings.is_null());
    match resulting {
        Some(ceilings) => number(&ceilings[key]),
        None if key == NO_PROGRESS_CEILING => number(&budgets[key]),
        None => {
            number(&budgets[key])
                + extensions
                    .iter()
                    .map(|extension| number(&extension[key]))
                    .sum::<f64>()
        }
    }
}

/// Self-reported cumulative consumption against the bound ceilings: above a
/// ceiling refuses, an enforced dimension without a metered value refuses,
/// exactly at a ceiling is admitted.
fn check_budget(phase_id: &str, input: &PhaseCloseInput, snapshot: &Value) -> Result<()> {
    match facts(input).budget_source.as_deref() {
        None => {
            return Err(refused(
                phase_id,
                "budget-consumption-missing(evidence needs budgetConsumption with source and measured/estimate/unknown classes)".into(),
            ))
        }
        Some("") => return Err(refused(phase_id, "evidence-budget-source-missing".into())),
        Some(_) => {}
    }
    let unmeasured: Vec<&str> = ENFORCED_BUDGETS
        .iter()
        .map(|(dimension, _)| *dimension)
        .filter(|dimension| !input.budget_consumption.contains_key(*dimension))
        .collect();
    if !unmeasured.is_empty() {
        return Err(refused(
            phase_id,
            format!("budget-unmeasured({})", unmeasured.join(",")),
        ));
    }
    let exceeded: Vec<String> = ENFORCED_BUDGETS
        .iter()
        .filter_map(|(dimension, ceiling_key)| {
            let value = input.budget_consumption.get(*dimension).copied()?;
            let limit = ceiling(snapshot, ceiling_key);
            #[allow(clippy::cast_precision_loss)]
            let over = value as f64 > limit;
            over.then(|| format!("exceeded:{dimension}({value}>{limit})"))
        })
        .collect();
    if exceeded.is_empty() {
        Ok(())
    } else {
        Err(refused(
            phase_id,
            format!("budget-exceeded({})", exceeded.join(",")),
        ))
    }
}
