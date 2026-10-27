//! Identity inspection and explicit, history-preserving change repair.

use crate::{BoundaryKind, BoundaryObligation, Change, Phase, Result, RuntimeError, RuntimeState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskMapping {
    pub source_key: String,
    pub source_task_key: String,
    pub target_task_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiptMapping {
    pub source_key: String,
    pub target_subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resolution {
    /// Every participating key, including the survivor, bound to JCS SHA-256.
    pub sources: BTreeMap<String, String>,
    pub survivor_key: String,
    pub target_key: String,
    pub task_mappings: Vec<TaskMapping>,
    /// Exact outstanding obligation keys; no receipt may be created or dropped.
    pub receipt_mappings: Vec<ReceiptMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepairInput {
    pub expected_revision: u64,
    pub reason: String,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepairRecord {
    pub phase_id: String,
    pub input: RepairInput,
    pub original_changes: BTreeMap<String, Change>,
    pub original_obligations: BTreeMap<String, BoundaryObligation>,
    pub resulting_change: Change,
    pub resulting_obligations: BTreeMap<String, BoundaryObligation>,
}

pub fn change_digest(change: &Change) -> Result<String> {
    let bytes = serde_jcs::to_vec(change).map_err(|e| RuntimeError::InvalidState(e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn invalid(message: impl Into<String>) -> RuntimeError {
    RuntimeError::InvalidState(format!("change identity repair refused: {}", message.into()))
}

pub(crate) fn admit_registration(state: &RuntimeState, phase_id: &str, change: &Change) -> Result<()> {
    let phase = state.phases.get(phase_id).ok_or_else(|| invalid(format!("phase {phase_id:?} missing")))?;
    if phase.changes.contains_key(&change.id) {
        return Err(RuntimeError::WorkItemExists { kind: "change", id: change.id.clone() });
    }
    if let Some((key, _)) = phase.changes.iter().find(|(_, existing)| existing.id == change.id) {
        return Err(invalid(format!("phase {phase_id} change ID {:?} already exists under imported key {key:?}; inspect and explicitly repair, do not register a duplicate", change.id)));
    }
    if state.change_identity_repairs.values().any(|r| r.phase_id == phase_id && r.original_changes.contains_key(&change.id) && r.input.resolution.target_key != change.id) {
        return Err(invalid(format!("change ID {:?} is a retired alias; resolve its canonical target instead of recreating it", change.id)));
    }
    Ok(())
}

/// Prepare the entire operation without mutation. Replay repeats this admission
/// against the preceding state, so an event cannot smuggle a fabricated result.
pub(crate) fn prepare(state: &RuntimeState, phase_id: &str, input: &RepairInput) -> Result<RepairRecord> {
    prepare_with(state, phase_id, input, None)
}

/// Replay re-derives the result from the recorded originals instead of re-checking the
/// admission-time revision (a fold can lag the authoritative revision) and live digests
/// (a later struct field must not invalidate journaled history).
pub(crate) fn prepare_replay(state: &RuntimeState, phase_id: &str, input: &RepairInput, recorded: &BTreeMap<String, Change>) -> Result<RepairRecord> {
    prepare_with(state, phase_id, input, Some(recorded))
}

fn prepare_with(state: &RuntimeState, phase_id: &str, input: &RepairInput, recorded: Option<&BTreeMap<String, Change>>) -> Result<RepairRecord> {
    if recorded.is_none() && state.revision != input.expected_revision {
        return Err(RuntimeError::RevisionConflict { expected: input.expected_revision, actual: state.revision });
    }
    if input.reason.trim().is_empty() { return Err(RuntimeError::ReasonRequired); }
    let phase = state.phases.get(phase_id).ok_or_else(|| invalid(format!("phase {phase_id:?} missing")))?;
    let resolution = &input.resolution;
    if resolution.sources.is_empty() || resolution.target_key.trim().is_empty() {
        return Err(invalid("explicit nonempty sources and targetKey required"));
    }
    if !resolution.sources.contains_key(&resolution.survivor_key) {
        return Err(invalid("survivorKey must be one of the explicitly bound sources"));
    }
    if phase.changes.contains_key(&resolution.target_key) && !resolution.sources.contains_key(&resolution.target_key) {
        return Err(invalid("target key exists outside the explicit sources"));
    }
    let mut originals = BTreeMap::new();
    for (key, digest) in &resolution.sources {
        let change = phase.changes.get(key).ok_or_else(|| invalid(format!("phase {phase_id} source key {key:?} missing")))?;
        match recorded {
            Some(originals) if originals.get(key) != Some(change) => return Err(invalid(format!("source content differs from the recorded original at key {key:?}"))),
            None if digest != &change_digest(change)? => return Err(invalid(format!("source content digest changed at key {key:?}"))),
            _ => {}
        }
        if change.id != resolution.target_key {
            return Err(invalid(format!("source {key:?} embedded ID {:?} does not establish target identity {:?}", change.id, resolution.target_key)));
        }
        originals.insert(key.clone(), change.clone());
    }
    let source_damaged = originals.len() > 1 || originals.iter().any(|(key, change)| {
        key != &change.id || change.tasks.iter().any(|(task_key, task)| task_key != &task.id)
    });
    if !source_damaged {
        return Err(invalid("no damaged identity was identified in the listed sources; ordinary transitions must use existing commands"));
    }
    if let Some((key, _)) = phase.changes.iter().find(|(key, change)| change.id == resolution.target_key && !resolution.sources.contains_key(*key)) {
        return Err(invalid(format!("key {key:?} also carries target ID {:?} but is not a listed source", resolution.target_key)));
    }
    let mut result = originals.get(&resolution.survivor_key).cloned().ok_or_else(|| invalid("survivor missing"))?;
    result.id.clone_from(&resolution.target_key);
    result.tasks.clear();
    let mut mapped = BTreeSet::new();
    for mapping in &resolution.task_mappings {
        if mapping.target_task_key.trim().is_empty() || !mapped.insert((mapping.source_key.clone(), mapping.source_task_key.clone())) {
            return Err(invalid("empty target task or repeated source task mapping"));
        }
        let mut task = originals.get(&mapping.source_key).and_then(|c| c.tasks.get(&mapping.source_task_key))
            .cloned().ok_or_else(|| invalid(format!("task {}/{} missing from explicit sources", mapping.source_key, mapping.source_task_key)))?;
        task.id.clone_from(&mapping.target_task_key);
        if let Some(existing) = result.tasks.get(&mapping.target_task_key) {
            if existing != &task {
                return Err(invalid(format!("conflicting outcomes/content mapped to task {:?}; resolve separately with real evidence", mapping.target_task_key)));
            }
        } else { result.tasks.insert(mapping.target_task_key.clone(), task); }
    }
    if mapped.len() != originals.values().map(|c| c.tasks.len()).sum::<usize>() {
        return Err(invalid("every source task requires an explicit mapping; no task may be discarded"));
    }
    if result.status.is_complete() && result.tasks.values().any(|t| !t.status.is_complete() && t.status != crate::WorkStatus::Cancelled) {
        return Err(invalid("survivor is complete but merged work remains unfinished; choose an existing unfinished survivor instead of fabricating completion"));
    }
    let mut old_obligations = BTreeMap::new();
    for (key, obligation) in &state.boundary_obligations {
        if resolution.sources.keys().any(|source| obligation.subject == *source || obligation.subject.strip_prefix(source)
            .is_some_and(|suffix| suffix.starts_with('/') || suffix.starts_with(':'))) {
            old_obligations.insert(key.clone(), obligation.clone());
        }
    }
    let mut new_obligations = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for mapping in &resolution.receipt_mappings {
        let original = old_obligations.get(&mapping.source_key).ok_or_else(|| invalid(format!("outstanding receipt {:?} not associated with source identity", mapping.source_key)))?;
        if !seen.insert(mapping.source_key.clone()) { return Err(invalid("repeated receipt source mapping")); }
        let expected_subject = match original.boundary {
            BoundaryKind::Change => Some(resolution.target_key.clone()),
            BoundaryKind::Task => resolution.task_mappings.iter().find(|task| {
                original.subject == format!("{}/{}", task.source_key, task.source_task_key)
                    || original.subject == format!("{}:{}", task.source_key, task.source_task_key)
                    || original.subject == format!("{}::{}", task.source_key, task.source_task_key)
            }).map(|task| format!("{}/{}", resolution.target_key, task.target_task_key)),
            BoundaryKind::Phase | BoundaryKind::Zeespec => None,
        };
        let valid_subject = expected_subject.as_deref() == Some(mapping.target_subject.as_str());
        if !valid_subject { return Err(invalid("receipt target must resolve to an existing resulting change/task")); }
        let target_key = format!("{:?}:{}", original.boundary, mapping.target_subject).to_ascii_lowercase();
        if new_obligations.contains_key(&target_key) || (state.boundary_obligations.contains_key(&target_key) && !old_obligations.contains_key(&target_key)) {
            return Err(invalid(format!("outstanding receipt collision at {target_key:?}")));
        }
        let mut effective = original.clone();
        effective.key.clone_from(&target_key);
        effective.subject.clone_from(&mapping.target_subject);
        new_obligations.insert(target_key, effective);
    }
    if seen.len() != old_obligations.len() { return Err(invalid("every outstanding source receipt requires an explicit mapping; none may be dropped or synthesized")); }
    Ok(RepairRecord { phase_id: phase_id.to_string(), input: input.clone(), original_changes: originals,
        original_obligations: old_obligations, resulting_change: result, resulting_obligations: new_obligations })
}

/// Inspect keys separately from embedded IDs; a map cannot expose repeated keys,
/// but imported records can still carry repeated or mismatched embedded IDs.
pub fn phase_issues(phase: &Phase) -> Vec<String> {
    let mut issues = Vec::new();
    let mut ids: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (key, change) in &phase.changes {
        ids.entry(&change.id).or_default().push(key);
        if key != &change.id {
            issues.push(format!("phase {} change key {key:?} differs from embedded ID {:?}", phase.id, change.id));
        }
        for (task_key, task) in &change.tasks {
            if task_key != &task.id {
                issues.push(format!("phase {} change {key:?} task key {task_key:?} differs from embedded ID {:?}", phase.id, task.id));
            }
        }
    }
    for (id, keys) in ids {
        if keys.len() > 1 {
            issues.push(format!("phase {} duplicate change ID {id:?} at keys {keys:?}", phase.id));
        }
    }
    issues
}

/// Reject lossy legacy conversion before collecting rows into a keyed map.
/// The caller retains the original input/backup; this function never mutates it.
pub(crate) fn validate_legacy(phase_id: &str, progress: &Value) -> Result<()> {
    let Some(changes) = super::legacy_changes(progress) else { return Ok(()); };
    let rows: Vec<(String, &Value)> = match changes {
        Value::Array(rows) => rows.iter().enumerate().map(|(i, row)| (i.to_string(), row)).collect(),
        Value::Object(rows) => rows.iter().map(|(key, row)| (key.clone(), row)).collect(),
        _ => return Ok(()),
    };
    let mut seen = BTreeMap::new();
    let mut issues = Vec::new();
    for (index, row) in rows {
        // The converter reads an embedded ID only from array strings and object rows.
        let id = row.get("id").and_then(Value::as_str).or_else(|| if changes.is_array() { row.as_str() } else { None });
        if let Some(id) = id {
            if let Some(previous) = seen.insert(id.to_owned(), index.clone()) {
                issues.push(format!("duplicate change ID {id:?} at rows/keys {previous:?} and {index:?}"));
            }
            if changes.is_object() && index != id {
                issues.push(format!("change key {index:?} differs from embedded ID {id:?}"));
            }
        }
        if let Some(tasks) = row.get("tasks").and_then(Value::as_array) {
            let mut task_ids = BTreeMap::new();
            for (ordinal, task) in tasks.iter().enumerate() {
                if let Some(task_id) = task.get("id").and_then(Value::as_str).or_else(|| task.as_str()) {
                    if let Some(previous) = task_ids.insert(task_id, ordinal) {
                        issues.push(format!("change row/key {index:?} duplicate task ID {task_id:?} at rows {previous} and {ordinal}"));
                    }
                }
            }
        }
    }
    if issues.is_empty() { Ok(()) } else {
        Err(RuntimeError::InvalidState(format!("phase {phase_id}: legacy identity conversion refused without altering input: {}", issues.join("; "))))
    }
}
