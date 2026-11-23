//! Admission and deterministic replay of an operator-signed phase exit.
use crate::phase_closure::{
    self, ContractBinding, PhaseCloseInput, PhaseClosure, PhaseDisposition,
};
use crate::scope_authority::{self, ScopeAction, ScopeAuthorityInput, ScopeAuthorityManifest};
use crate::{EventKind, KbdStateV2, Result, RuntimeError, WorkStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

fn invalid(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::ScopeAuthority(reason.into())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperatorDecision {
    pub reference: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exercised {
    pub id: String,
    pub verdict: String,
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Withdrawal {
    pub id: String,
    pub kind: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BudgetAccounting {
    pub class: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<String, u64>,
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn relative(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

fn destination(value: &str, predecessor: &str) -> bool {
    if let Some(url) = value.strip_prefix("https://") {
        return url.split_once('/').is_some_and(|(host, path)| {
            !host.is_empty()
                && !path.trim_matches('/').is_empty()
                && !url.contains(char::is_whitespace)
        });
    }
    !value.is_empty()
        && value != predecessor
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

pub(crate) fn validate_shape(m: &ScopeAuthorityManifest) -> Result<()> {
    if m.bound_contract_digest
        .as_deref()
        .is_none_or(|d| !digest(d))
        || m.operator_decision
            .as_ref()
            .is_none_or(|d| d.reference.trim().is_empty() || d.text.trim().is_empty())
        || m.limits
            .as_ref()
            .is_none_or(|items| items.iter().any(|s| s.trim().is_empty()))
    {
        return Err(invalid("supersede-exit requires boundContractDigest, operatorDecision.reference and text, and limits"));
    }
    let budget = m
        .budget_accounting
        .as_ref()
        .ok_or_else(|| invalid("missing budgetAccounting"))?;
    let valid_budget = match budget.class.as_str() {
        "unmetered" => budget.values.is_empty(),
        "estimated" | "measured" => {
            !budget.values.is_empty() && budget.values.keys().all(|k| !k.trim().is_empty())
        }
        _ => false,
    };
    if !valid_budget {
        return Err(invalid(
            "budgetAccounting requires measured or estimated values, or unmetered with no values",
        ));
    }
    let exercised = m
        .exercised
        .as_ref()
        .ok_or_else(|| invalid("missing exercised list"))?;
    let withdrawn = m
        .withdrawn
        .as_ref()
        .ok_or_else(|| invalid("missing withdrawn list"))?;
    let mut scenarios = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    for item in exercised {
        if item.id.trim().is_empty()
            || !scenarios.insert(&item.id)
            || !matches!(item.verdict.as_str(), "PASS" | "FAIL")
            || item.evidence.is_empty()
            || item
                .evidence
                .iter()
                .any(|e| !relative(&e.path) || !digest(&e.sha256))
        {
            return Err(invalid(format!(
                "invalid or duplicated exercised scenario or evidence: {}",
                item.id
            )));
        }
        for scope in &item.scope_refs {
            if scope.trim().is_empty() {
                return Err(invalid("empty exercised scope reference"));
            }
            scopes.insert(scope);
        }
    }
    for item in withdrawn {
        let unique = match item.kind.as_str() {
            "scenario" => scenarios.insert(&item.id),
            "scope" => scopes.insert(&item.id),
            _ => false,
        };
        if !unique || item.id.trim().is_empty() || !destination(&item.destination, &m.phase_id) {
            return Err(invalid(format!(
                "invalid or overlapping withdrawal {}:{}",
                item.kind, item.id
            )));
        }
    }
    Ok(())
}

fn check_binding(
    state: &KbdStateV2,
    phase: &str,
    binding: &ContractBinding,
    m: &ScopeAuthorityManifest,
) -> Result<()> {
    if m.action != ScopeAction::SupersedeExit
        || m.project_id != state.project_id
        || m.run_id != state.run_id
        || m.phase_id != phase
        || m.bound_contract_digest.as_ref() != Some(&binding.contract_digest)
    {
        return Err(invalid(
            "supersede-exit project, run, phase or boundContractDigest mismatch",
        ));
    }
    let mut accounted: BTreeSet<_> = m.exercised.iter().flatten().map(|e| &e.id).collect();
    for item in m
        .withdrawn
        .iter()
        .flatten()
        .filter(|w| w.kind == "scenario")
    {
        if !binding.scenario_digests.contains_key(&item.id) {
            return Err(invalid(format!(
                "withdrawn scenario {} is outside the bound contract",
                item.id
            )));
        }
        accounted.insert(&item.id);
    }
    if let Some(id) = binding
        .scenario_digests
        .keys()
        .find(|id| !accounted.contains(id))
    {
        return Err(invalid(format!("unaccounted bound scenario {id}")));
    }
    Ok(())
}

fn read_confined(root: &Path, path: &str) -> Result<Vec<u8>> {
    if !relative(path) {
        return Err(invalid(format!("expected project-relative path: {path}")));
    }
    let target = root
        .join(path)
        .canonicalize()
        .map_err(|e| invalid(format!("read {path}: {e}")))?;
    if !target.starts_with(root) || !target.is_file() {
        return Err(invalid(format!("expected project file: {path}")));
    }
    std::fs::read(target).map_err(|e| invalid(format!("read {path}: {e}")))
}

/// Read-only pre-sign validation. The disk-backed command repeats this under the write lock.
pub fn validate_exit(state: &KbdStateV2, phase: &str, raw: &str, root: &Path) -> Result<()> {
    let m = scope_authority::parse_manifest(raw)?;
    let view = phase_closure::disposition_view(state, phase)?;
    let binding = view
        .binding
        .ok_or_else(|| invalid("supersede-exit requires a bound contract"))?;
    if view.closure.is_some()
        || !matches!(
            view.phase_status,
            WorkStatus::InProgress | WorkStatus::Blocked
        )
    {
        return Err(invalid(format!("phase {phase} is not open for closure")));
    }
    check_binding(state, phase, &binding, &m)?;
    let root = root.canonicalize()?;
    let path = format!(
        ".kbd-orchestrator/phases/{phase}/adopted/{}.json",
        binding.contract_digest
    );
    let snapshot = read_confined(&root, &path)?;
    if format!("{:x}", Sha256::digest(&snapshot)) != binding.contract_digest {
        return Err(invalid(
            "adopted snapshot does not match boundContractDigest",
        ));
    }
    let snapshot: Value = serde_json::from_slice(&snapshot)?;
    let mut snapshot_ids = BTreeSet::new();
    for scenario in snapshot["scenarioMatrix"]
        .as_array()
        .ok_or_else(|| invalid("snapshot missing scenarioMatrix"))?
    {
        let id = scenario["id"]
            .as_str()
            .ok_or_else(|| invalid("snapshot scenario missing identity"))?;
        if !snapshot_ids.insert(id) {
            return Err(invalid(format!("duplicate snapshot scenario {id}")));
        }
    }
    if !snapshot_ids
        .iter()
        .copied()
        .eq(binding.scenario_digests.keys().map(String::as_str))
    {
        return Err(invalid(
            "bound scenario identity set differs from adopted snapshot",
        ));
    }
    let mut bound_scope = BTreeSet::new();
    for item in snapshot["functionalScope"]
        .as_array()
        .ok_or_else(|| invalid("snapshot missing functionalScope"))?
    {
        let name = item
            .as_str()
            .or_else(|| item["item"].as_str())
            .ok_or_else(|| invalid("invalid snapshot functionalScope item"))?;
        bound_scope.insert(name);
    }
    let mut accounted = BTreeSet::new();
    for item in m.exercised.iter().flatten() {
        accounted.extend(item.scope_refs.iter().map(String::as_str));
        for evidence in &item.evidence {
            let bytes = read_confined(&root, &evidence.path)?;
            if format!("{:x}", Sha256::digest(bytes)) != evidence.sha256 {
                return Err(invalid(format!("stale evidence: {}", evidence.path)));
            }
        }
    }
    accounted.extend(
        m.withdrawn
            .iter()
            .flatten()
            .filter(|w| w.kind == "scope")
            .map(|w| w.id.as_str()),
    );
    if let Some(id) = bound_scope.symmetric_difference(&accounted).next() {
        return Err(invalid(format!(
            "unaccounted or out-of-scope functionalScope item: {id}"
        )));
    }
    for (dim, value) in m.budget_accounting.iter().flat_map(|b| &b.values) {
        if view
            .consumption_high_water
            .get(dim)
            .is_some_and(|floor| value < floor)
        {
            return Err(invalid(format!("budget-regressed({dim})")));
        }
    }
    Ok(())
}

fn close_input(m: &ScopeAuthorityManifest, manifest_hash: &str) -> Result<PhaseCloseInput> {
    let exercised = m
        .exercised
        .as_ref()
        .ok_or_else(|| invalid("missing exercised list"))?;
    Ok(PhaseCloseInput {
        run_id: m.run_id.clone(),
        contract_digest: m
            .bound_contract_digest
            .clone()
            .ok_or_else(|| invalid("missing boundContractDigest"))?,
        evidence_digest: manifest_hash.into(),
        disposition: if exercised.is_empty() {
            PhaseDisposition::Unmet
        } else {
            PhaseDisposition::Superseded
        },
        reason_code: Some("operator-decided-exit".into()),
        scenarios: Vec::new(),
        budget_consumption: m
            .budget_accounting
            .as_ref()
            .ok_or_else(|| invalid("missing budgetAccounting"))?
            .values
            .clone(),
        evidence_facts: None,
    })
}

pub(crate) fn prepare(
    state: &KbdStateV2,
    phase: &str,
    authority: &ScopeAuthorityInput,
) -> Result<EventKind> {
    let (manifest, record) = scope_authority::verify(authority)?;
    let input = close_input(&manifest, &record.manifest_sha256)?;
    Ok(EventKind::PhaseDispositionRecorded {
        phase_id: phase.into(),
        closure: Box::new(PhaseClosure {
            closure_key: phase_closure::closure_key(&state.project_id, phase, &input)?,
            input,
            admission_version: Some(crate::evidence_admission::EVIDENCE_ADMISSION_VERSION),
            operator_exit: Some(manifest),
            exit_authority: Some(record),
        }),
    })
}

pub(crate) fn check_record(
    state: &KbdStateV2,
    phase: &str,
    binding: &ContractBinding,
    closure: &PhaseClosure,
) -> Result<()> {
    let record = closure
        .exit_authority
        .as_ref()
        .ok_or_else(|| invalid("missing signed exit authority"))?;
    let m = scope_authority::verify_record(record)?;
    if closure.operator_exit.as_ref() != Some(&m)
        || closure.input != close_input(&m, &record.manifest_sha256)?
    {
        return Err(invalid("closure does not match signed operator exit"));
    }
    check_binding(state, phase, binding, &m)
}
