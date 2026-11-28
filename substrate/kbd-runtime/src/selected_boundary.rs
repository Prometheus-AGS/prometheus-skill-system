//! Selected execution is a signed *metadata-only* handoff on the existing
//! full adopted contract. It does not replace the contract or admit evidence.
//! Existing GateStarted/GateFinished records provide single-use accounting.
use crate::{GateKind, GateRun, KbdStateV2, Result, RuntimeError, WorkStatus};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Component, Path, PathBuf}};

pub(crate) mod selected_exec;
use selected_exec::ExecBinding;

pub const REQUEST_PREFIX: &str = "selected:";
pub const RESERVED_PREFIX: &str = "selected-boundary:";
const KEY: &str = "selectedExecution";
// Static review rejected activation until the authoritative host owns the
// dispatch fence through POST and the running profile identity is qualified.
// This refusal is not an operator-overridable flag or a second authority.
fn require_reviewed_dispatch_fence()->Result<()> {
    Err(fail("selected dispatch remains unqualified: authoritative host fence through POST and running profile binding require independent review and proof"))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PinnedFile { path: String, sha256: String }

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Start {
    id: String,
    command_id: String,
    argv: Vec<String>,
    cwd: String,
    scenario_ids: Vec<String>,
    requires: Vec<String>,
    max_seconds: u64,
    // The existing, independently reviewed product launcher is responsible
    // for target-specific effect admission and physical settlement.
    guard: PinnedFile,
    exec: ExecBinding,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Selection {
    schema_version: String,
    boundary: String,
    expires_at: DateTime<Utc>,
    selected_scenario_ids: Vec<String>,
    candidate_manifest: PinnedFile,
    source_digests: BTreeMap<String, String>,
    schedule: PinnedFile,
    previous_evidence: Option<PinnedFile>,
    starts: Vec<Start>,
}

#[derive(Debug)]
pub struct SelectedInvocation {
    pub scope: String,
    pub gate_id: String,
    pub exec_request_identity: String,
}

fn fail(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ScopeAuthority(format!("selected boundary: {}", message.into()))
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(fail(message)) }
}
fn sha(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn id_valid(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/'))
}
fn argv_digest(args: &[String]) -> String {
    let mut h = Sha256::new();
    for arg in args { h.update(arg.as_bytes()); h.update([0]); }
    format!("{:x}", h.finalize())
}
fn confined(root: &Path, relative: &str) -> Result<PathBuf> {
    let p = Path::new(relative);
    require(!relative.is_empty() && p.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)), "path outside project")?;
    let actual = root.join(p).canonicalize().map_err(|_| fail(format!("missing pinned path {relative}")))?;
    require(actual.starts_with(root), "pinned path resolves outside project")?;
    Ok(actual)
}
fn read_pin(root: &Path, path: &str, digest: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(confined(root, path)?)?;
    require(sha(&bytes) == digest, &format!("stale pinned file {path}"))?;
    Ok(bytes)
}
fn remove_selection(snapshot: &mut Value) -> Result<Option<Value>> {
    let packet = snapshot.get_mut("boundaryPacket").and_then(Value::as_object_mut).ok_or_else(|| fail("parent has no boundary packet"))?;
    Ok(packet.remove(KEY))
}
fn scope_id(digest: &str, start: &str) -> String { format!("{RESERVED_PREFIX}{digest}:{start}") }
pub(crate) fn consumption_id(scope: &str) -> String {
    sha(format!("kbd.selected-start/v1\0{scope}").as_bytes())
}
pub(crate) fn is_reserved(scope: &str) -> bool { scope.starts_with(RESERVED_PREFIX) || scope.starts_with(REQUEST_PREFIX) }

fn completed_owner(state: &KbdStateV2, phase: &str, owner: &str) -> bool {
    let Some(p) = state.phases.get(phase) else { return false; };
    let matches: Vec<_> = p.changes.iter().flat_map(|(change, c)| {
        c.tasks.values().filter(move |t| t.id == owner || format!("{change}/{}", t.id) == owner)
            .map(move |t| (c, t))
    }).collect();
    matches.len() == 1 && matches[0].1.status == WorkStatus::Complete
        && matches[0].0.depends_on.iter().all(|id| p.changes.get(id).is_some_and(|c| c.implementation_status == WorkStatus::Complete))
}

fn snapshot(root: &Path,state: &KbdStateV2,phase: &str,digest: &str) -> Result<Value> {
    let phase_ref = state.phases.get(phase).ok_or_else(|| fail("unknown phase"))?;
    let directory = crate::phase_projection_directory(Path::new(".kbd-orchestrator"), state, phase_ref)?;
    let file = directory.join("adopted").join(format!("{digest}.json"));
    let bytes = read_pin(root,&file.to_string_lossy(),digest)?;
    serde_json::from_slice(&bytes).map_err(|_| fail("invalid adopted snapshot"))
}
fn selected_documents(root: &Path,state: &KbdStateV2,phase: &str,previous: &str,next: &str) -> Result<(Value,Selection)> {
    let mut current = snapshot(root,state,phase,next)?;
    let mut old = snapshot(root,state,phase,previous)?;
    let selection_value = remove_selection(&mut current)?.ok_or_else(|| fail("one-time signed selectedExecution handoff required"))?;
    remove_selection(&mut old)?;
    require(current == old, "handoff changes non-selection contract content")?;
    require(current["contractVersion"] == "2" && current["kind"] == "kbd.adopted-contract", "only adopted v2 contracts are supported")?;
    let selection = serde_json::from_value(selection_value).map_err(|e| fail(format!("invalid selectedExecution: {e}")))?;
    Ok((current,selection))
}
struct BoundScope<'a> {
    digest: &'a str,
    scenarios: &'a BTreeMap<String,String>,
    history: &'a [String],
    bound_at_revision: u64,
    signed_at: DateTime<Utc>,
}
fn signed_at(record: &crate::scope_authority::ScopeAuthorityManifest) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&record.timestamp).map(|t| t.with_timezone(&Utc)).map_err(|_| fail("invalid signature timestamp"))
}

fn load_selection(root: &Path, state: &KbdStateV2, phase: &str, allow_completed_summary: bool) -> Result<(String, Selection)> {
    let entry = state.phase_closure.phases.get(phase).ok_or_else(|| fail("phase is not bound"))?;
    require(entry.terminal.is_none(), "phase already closed")?;
    let binding = entry.binding.as_ref().ok_or_else(|| fail("phase is not bound"))?;
    let authority = binding.authority.as_ref().ok_or_else(|| fail("one-time signed selection-only amendment required"))?;
    let record = crate::scope_authority::verify_record(authority)?;
    let previous = record.previous_contract_digest.as_deref().ok_or_else(|| fail("selection is not an amendment"))?;
    require(record.action == crate::scope_authority::ScopeAction::AmendSelectedExecution
        && record.new_contract_digest.as_deref() == Some(binding.contract_digest.as_str())
        && record.project_id == state.project_id && record.run_id == state.run_id && record.phase_id == phase
        && record.scenario_changes.as_ref().is_some_and(|c| c.added.is_empty() && c.modified.is_empty() && c.removed.is_empty())
        && binding.amended_scenarios.is_empty() && entry.binding_history.last().map(String::as_str) == Some(previous),
        "signed amend-selected-execution handoff must preserve every scenario and bind the current parent")?;
    // Use the same trusted operator verification at admission so a removed
    // signer cannot provide a fresh physical start.
    crate::scope_authority::verify(&crate::ScopeAuthorityInput {
        manifest_raw: authority.manifest_raw.clone(), signature: authority.signature.clone(),
    })?;
    let (current,selection) = selected_documents(root,state,phase,previous,&binding.contract_digest)?;
    let scope = BoundScope { digest: &binding.contract_digest, scenarios: &binding.scenario_digests,
        history: &entry.binding_history, bound_at_revision: entry.bound_at_revision, signed_at: signed_at(&record)? };
    validate_selection(root,state,phase,&current,&selection,&scope,allow_completed_summary)?;
    Ok((binding.contract_digest.clone(),selection))
}

// The exact same checks run at proposed handoff and before a physical start.
fn validate_selection(root: &Path,state: &KbdStateV2,phase: &str,current: &Value,selection: &Selection,
    scope: &BoundScope<'_>,allow_completed_summary: bool) -> Result<()> {
    let entry = state.phase_closure.phases.get(phase).ok_or_else(|| fail("phase is not bound"))?;
    require(state.conflicts.values().all(|conflict| conflict.resolved_by_event_id.is_some()),
        "canonical conflict must be reconciled before selected execution")?;
    let signed_at = scope.signed_at;
    let now = Utc::now();
    // Expiry limits physical admission, not the ability to reconcile already
    // recorded evidence or close. No completed scope can authorize a new start.
    let completed_summary = allow_completed_summary && !selection.starts.is_empty() && selection.starts.iter().all(|start| {
        let scope = scope_id(scope.digest,&start.id);
        state.latest_gate_receipts.values().any(|r| r.scope == scope &&
            r.kind == GateKind::SelectedIntegration && r.gate_id == consumption_id(&scope)
            && r.outcome == crate::GateOutcome::Passed
            && selected_exec::settled(state,r,&start.exec,&selection.candidate_manifest))
    });
    require(selection.schema_version == "1" && signed_at <= now && (now < selection.expires_at || completed_summary || (allow_completed_summary && state.active_gates.values().any(|gate| gate.phase_id.as_deref() == Some(phase) && gate.scope.starts_with(&format!("{RESERVED_PREFIX}{}:",scope.digest)))))
        && signed_at < selection.expires_at, "selected authority expired or invalid")?;
    require(matches!(selection.boundary.as_str(), "B4" | "B5"), "unknown boundary")?;
    let selected: BTreeSet<_> = selection.selected_scenario_ids.iter().map(String::as_str).collect();
    require(!selected.is_empty() && selected.len() == selection.selected_scenario_ids.len(), "empty or duplicate scenario selection")?;
    let rows = current["scenarioMatrix"].as_array().ok_or_else(|| fail("missing scenario matrix"))?;
    let row_ids: BTreeSet<_> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    require(rows.len() == row_ids.len() && row_ids == scope.scenarios.keys().map(String::as_str).collect()
        && selected.is_subset(&row_ids), "selection does not match immutable complete parent")?;
    let parent_commands = current["boundaryPacket"]["commands"].as_array().ok_or_else(|| fail("missing parent commands"))?;
    let parent_by_id: BTreeMap<_,_> = parent_commands.iter().filter_map(|c| c["id"].as_str().map(|id| (id,c))).collect();
    require(parent_by_id.len() == parent_commands.len(), "duplicate or missing parent command identity")?;
    let mut required = BTreeSet::new();
    for row in rows.iter().filter(|r| r["id"].as_str().is_some_and(|id| selected.contains(id))) {
        let id = row["id"].as_str().ok_or_else(|| fail("invalid scenario"))?;
        let owner = row["ownerTask"].as_str().ok_or_else(|| fail("missing scenario owner"))?;
        require(owner.split('+').all(|o| completed_owner(state, phase, o.trim())), &format!("selected owner not implementation-complete: {id}"))?;
        let maps = |c: &Value| c["scenarioIds"].as_array().is_some_and(|ids| ids.iter().any(|s| s.as_str() == Some(id)));
        let mut case_count = 0;
        for (cid,c) in &parent_by_id {
            if c["kind"] == "scenario" && maps(c) { required.insert((*cid,id)); case_count += 1; }
            if c["kind"] == "negativeControl" && (maps(c) || row["negativeControl"]["commandRef"].as_str() == Some(*cid)) {
                required.insert((*cid,id));
            }
        }
        require(case_count > 0 && row["negativeControl"]["commandRef"].as_str().is_some_and(|cid| parent_by_id.get(cid).is_some_and(|c| c["kind"] == "negativeControl")), "required case or control absent")?;
    }
    let mut seen = BTreeSet::new();
    let mut request_hashes = BTreeSet::new();
    let mut receipt_paths = BTreeSet::new();
    let mut case_starts: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut offered = BTreeSet::new();
    let mut total = 0u64;
    for start in &selection.starts {
        require(id_valid(&start.id) && !seen.contains(start.id.as_str()) && start.requires.iter().all(|p| seen.contains(p.as_str()))
            && start.max_seconds > 0 && !start.argv.is_empty(), "invalid physical start or prerequisite order")?;
        let command = parent_by_id.get(start.command_id.as_str()).ok_or_else(|| fail("unknown parent command"))?;
        let kind = command["kind"].as_str().ok_or_else(|| fail("unknown command kind"))?;
        require(command["cwd"].as_str().unwrap_or(".") == start.cwd, "working directory differs from parent command")?;
        let scope: BTreeSet<_> = start.scenario_ids.iter().map(String::as_str).collect();
        require(scope.len() == start.scenario_ids.len() && scope.is_subset(&selected), "unselected or duplicate scenario command coverage")?;
        match kind {
            "build" | "run" => {
                require(command["argv"] == serde_json::to_value(&start.argv).map_err(|_| fail("invalid argv"))?, "build/run argv differs from parent")?;
            }
            "scenario" | "negativeControl" => {
                require(!scope.is_empty() && scope.iter().all(|id| required.contains(&(start.command_id.as_str(), *id))), "behavior command not mapped to selection")?;
                for id in scope {
                    if kind == "scenario" { case_starts.entry(id).or_default().push(start.id.as_str()); }
                    else {
                        require(case_starts.get(id).is_some_and(|prior| !prior.is_empty() && prior.iter().all(|p| start.requires.iter().any(|r| r == p))),
                            "control must depend on its selected case starts")?;
                    }
                    offered.insert((start.command_id.as_str(),id));
                }
            }
            _ => return Err(fail("unsupported command kind")),
        }
        // The exact narrowed argv (including the existing guard), rather than
        // an inferred shell string, is part of the owner-signed snapshot.
        require(selection.source_digests.get(&start.guard.path) == Some(&start.guard.sha256), "guard is not a pinned source")?;
        read_pin(root, &start.guard.path, &start.guard.sha256)?;
        confined(root, &start.cwd)?;
        start.exec.validate(root,phase,&start.id,&selection.candidate_manifest,&start.guard,&selection.source_digests,&start.argv,&start.cwd,start.max_seconds)?;
        require(request_hashes.insert(start.exec.request_hash.as_str()) && receipt_paths.insert(&start.exec.receipt_path), "duplicate selected Exec request or receipt destination")?;
        total = total.checked_add(start.max_seconds).ok_or_else(|| fail("time budget overflow"))?;
        seen.insert(start.id.as_str());
    }
    require(offered == required && !selection.starts.is_empty(), "missing selected case or mandatory negative control")?;
    require(!selection.source_digests.is_empty(), "candidate source digests missing")?;
    for (path,digest) in &selection.source_digests { read_pin(root,path,digest)?; }
    read_pin(root,&selection.candidate_manifest.path,&selection.candidate_manifest.sha256)?;
    let schedule_bytes = read_pin(root,&selection.schedule.path,&selection.schedule.sha256)?;
    let schedule: Value = serde_json::from_slice(&schedule_bytes).map_err(|_| fail("invalid pinned physical schedule"))?;
    let scheduled = schedule["starts"].as_array().ok_or_else(|| fail("physical start roster missing"))?;
    require(scheduled.len() == selection.starts.len(), "scope must name every start in its pinned physical schedule")?;
    for (start,physical) in selection.starts.iter().zip(scheduled) {
        let argv = serde_json::to_vec(&start.argv).map_err(|_| fail("invalid scheduled argv"))?;
        require(physical["id"] == start.id && physical["definition"] == start.command_id
            && physical["argvSha256"].as_str() == Some(sha(&argv).as_str())
            && physical["totalSeconds"].as_u64() == Some(start.max_seconds)
            && physical["dependsOn"] == serde_json::to_value(&start.requires).map_err(|_| fail("invalid schedule dependencies"))?,
            "selected invocation differs from the pinned physical schedule")?;
    }
    let finalization = schedule["packetFinalization"]["totalSeconds"].as_u64().ok_or_else(|| fail("missing terminal settlement reserve"))?;
    total = total.checked_add(finalization).ok_or_else(|| fail("budget overflow"))?;
    require(schedule["budget"]["serialCeilingSeconds"].as_u64().is_some_and(|n| n >= total)
        && schedule["budget"]["toolCalls"].as_u64() == Some(selection.starts.len() as u64), "physical schedule budget is inconsistent")?;
    // Keep prior signed scope reservations. An unaccounted prior scope never
    // makes room for additional effects; high-water accounting is conservative.
    let mut prior_digests = BTreeSet::new();
    for gate_scope in state.active_gates.values().filter(|g| g.phase_id.as_deref() == Some(phase)).map(|g| g.scope.as_str())
        .chain(state.latest_gate_receipts.values().filter(|g| g.phase_id.as_deref() == Some(phase)).map(|g| g.scope.as_str())) {
        if let Some((digest,_)) = gate_scope.strip_prefix(RESERVED_PREFIX).and_then(|scope| scope.split_once(':')) {
            if digest != scope.digest { prior_digests.insert(digest); }
        }
    }
    let mut reserved_time = 0u64;
    let mut reserved_calls = 0u64;
    let mut prior_repairs = 0u64;
    for digest in prior_digests {
        require(scope.history.iter().any(|d| d == digest), "prior selected start has unknown binding")?;
        let mut historical = snapshot(root,state,phase,digest)?;
        let prior_value = remove_selection(&mut historical)?.ok_or_else(|| fail("prior selected scope missing"))?;
        let prior: Selection = serde_json::from_value(prior_value).map_err(|_| fail("prior selected scope invalid"))?;
        if selection.boundary == "B4" {
            require(!prior.selected_scenario_ids.iter().any(|id| selected.contains(id.as_str())), "selected journey already attempted; use separately signed repair after recording its disposition")?;
        }
        let prior_seconds = prior.starts.iter().try_fold(0u64, |sum,start| sum.checked_add(start.max_seconds)).ok_or_else(|| fail("historical budget overflow"))?;
        let prior_schedule = read_pin(root,&prior.schedule.path,&prior.schedule.sha256)?;
        let prior_schedule: Value = serde_json::from_slice(&prior_schedule).map_err(|_| fail("prior schedule invalid"))?;
        let prior_finalization = prior_schedule["packetFinalization"]["totalSeconds"].as_u64().ok_or_else(|| fail("prior terminal reserve missing"))?;
        reserved_time = reserved_time.checked_add(prior_seconds).and_then(|n| n.checked_add(prior_finalization)).ok_or_else(|| fail("historical budget overflow"))?;
        reserved_calls = reserved_calls.checked_add(prior.starts.len() as u64).ok_or_else(|| fail("historical budget overflow"))?;
        if prior.boundary == "B5" { prior_repairs += 1; }
    }
    let budgets = &current["budgets"];
    if !completed_summary {
        require(entry.consumption_high_water.get("noProgress").copied().unwrap_or(0) < budgets["noProgressLimit"].as_u64().unwrap_or(0), "no-progress budget exhausted")?;
    }
    let used_time = entry.consumption_high_water.get("timeMinutes").copied().unwrap_or(0);
    let used_calls = entry.consumption_high_water.get("toolCalls").copied().unwrap_or(0);
    // Conservative base ceilings: a signed additive extension is never
    // implicitly spent by this first scoped path.
    require(completed_summary || (total <= budgets["timeMinutes"].as_u64().unwrap_or(0).saturating_sub(used_time).saturating_mul(60).saturating_sub(reserved_time)
        && selection.starts.len() as u64 <= budgets["toolCalls"].as_u64().unwrap_or(0).saturating_sub(used_calls).saturating_sub(reserved_calls)), "selected packet exceeds remaining parent budget")?;
    if selection.boundary == "B5" {
        let prior = selection.previous_evidence.as_ref().ok_or_else(|| fail("B5 requires prior immutable evidence"))?;
        let bytes = read_pin(root,&prior.path,&prior.sha256)?;
        let evidence: Value = serde_json::from_slice(&bytes).map_err(|_| fail("invalid prior evidence"))?;
        // Reuse the canonical refusal recorded by the existing close adapter,
        // not a newly authored standalone JSON claim of failure.
        let recorded = entry.last_refusal.as_ref().ok_or_else(|| fail("B5 requires a recorded canonical refusal"))?;
        let refusal = &recorded.closure.input;
        require(recorded.recorded_at_revision < scope.bound_at_revision
            && refusal.run_id == state.run_id && refusal.evidence_digest == prior.sha256
            && evidence["contractDigest"].as_str() == Some(refusal.contract_digest.as_str())
            && matches!(refusal.disposition, crate::PhaseDisposition::Fail | crate::PhaseDisposition::Blocked)
            && refusal.evidence_facts.as_ref().is_some_and(|facts| facts.phase == phase)
            && selection.selected_scenario_ids.iter().all(|id| refusal.scenarios.iter().any(|row| row.id == *id
                && matches!(row.status.as_str(), "FAIL" | "BLOCKED" | "NOT_EXERCISED"))),
            "B5 evidence is not the canonical affected refusal")?;
        require(evidence["phase"] == phase && evidence["contractDigest"].as_str().is_some_and(|digest| scope.history.iter().any(|d| d == digest))
            && evidence["scenarios"].as_array().is_some_and(|rows| selection.selected_scenario_ids.iter().all(|id| rows.iter().any(|r| r["id"].as_str() == Some(id.as_str()) && matches!(r["status"].as_str(),Some("FAIL" | "BLOCKED" | "NOT_EXERCISED"))))),
            "B5 prior affected outcomes are missing")?;
        require(completed_summary || budgets["repairAttempts"].as_u64().unwrap_or(0) > entry.consumption_high_water.get("repairAttempts").copied().unwrap_or(0).saturating_add(prior_repairs), "repair budget exhausted")?;
    } else {
        require(selection.previous_evidence.is_none(), "B4 cannot claim repair evidence")?;
    }
    Ok(())
}

/// The existing canonical bind command is the writer floor. Admission of
/// the new action occurs with the host snapshot and current signed owner
/// authority, before the existing binding event is appended.
pub(crate) fn admit_handoff(root: &Path,state: &KbdStateV2,phase: &str,input: &crate::ContractBindingInput,
    authority: Option<&crate::ScopeAuthorityInput>) -> Result<()> {
    let Some(authority) = authority else { return Ok(()); };
    let parsed = crate::scope_authority::parse_manifest(&authority.manifest_raw)?;
    if parsed.action != crate::scope_authority::ScopeAction::AmendSelectedExecution { return Ok(()); }
    require_reviewed_dispatch_fence()?;
    let (manifest,_) = crate::scope_authority::verify(authority)?;
    let root = root.canonicalize()?;
    let entry = state.phase_closure.phases.get(phase).ok_or_else(|| fail("selected handoff requires an already adopted phase"))?;
    let previous = entry.binding.as_ref().ok_or_else(|| fail("selected handoff requires existing binding"))?;
    require(state.active_path.phase_id.as_deref() == Some(phase) && entry.terminal.is_none()
        && !state.active_gates.values().any(|gate| gate.phase_id.as_deref() == Some(phase)),
        "selected handoff requires active phase and settled prior gates")?;
    require(manifest.project_id == state.project_id && manifest.run_id == state.run_id && manifest.phase_id == phase
        && manifest.contract_path.as_deref() == Some(input.contract_path.as_str())
        && manifest.previous_contract_digest.as_deref() == Some(previous.contract_digest.as_str())
        && manifest.new_contract_digest.as_deref() == Some(input.contract_digest.as_str())
        && input.scenario_digests == previous.scenario_digests,
        "selected handoff differs from signed parent identity or obligations")?;
    let (current,selection) = selected_documents(&root,state,phase,&previous.contract_digest,&input.contract_digest)?;
    let mut history = entry.binding_history.clone();
    history.push(previous.contract_digest.clone());
    let scope = BoundScope { digest: &input.contract_digest, scenarios: &input.scenario_digests, history: &history,
        bound_at_revision: state.revision.saturating_add(1), signed_at: signed_at(&manifest)? };
    validate_selection(&root,state,phase,&current,&selection,&scope,false)?;
    let first=selection.starts.first().ok_or_else(|| fail("empty selected starts"))?;
    require(selection.starts.iter().all(|s| s.exec.same_runner(&first.exec)), "selected packet spans unsupported runner identities")?;
    first.exec.verify_installed_identity()
}

pub(crate) fn selected_authority(input: &crate::ScopeAuthorityInput) -> Result<bool> {
    Ok(crate::scope_authority::parse_manifest(&input.manifest_raw)?.action == crate::scope_authority::ScopeAction::AmendSelectedExecution)
}

fn selected_start<'a>(state: &KbdStateV2, phase: &str, digest: &str, selection: &'a Selection, start_id: &str) -> Result<&'a Start> {
    let start = selection.starts.iter().find(|s| s.id == start_id).ok_or_else(|| fail("unknown selected start"))?;
    let scope = scope_id(digest,start_id);
    require(!state.phase_closure.consumed_manifests.contains(&consumption_id(&scope))
        && !state.active_gates.values().any(|g| g.scope == scope)
        && !state.latest_gate_receipts.values().any(|g| g.scope == scope),
        "selected start already consumed; no automatic retry")?;
    // The gate receipt orders physical starts only. The product launcher
    // independently requires its actual build, containment and case evidence.
    for prior in selection.starts.iter().take_while(|p| p.id != start.id) {
        let scope = scope_id(digest,&prior.id);
        require(state.latest_gate_receipts.values().any(|r| r.scope == scope && r.phase_id.as_deref() == Some(phase)
            && r.kind == GateKind::SelectedIntegration && r.gate_id == consumption_id(&scope)
            && r.outcome == crate::GateOutcome::Passed && selected_exec::settled(state,r,&prior.exec,&selection.candidate_manifest)), "prior selected start not successfully settled")?;
    }
    require(!state.active_gates.values().any(|g| g.phase_id.as_deref() == Some(phase) && is_reserved(&g.scope)), "an earlier selected start is unsettled")?;
    Ok(start)
}

/// Normal resume path: give the next executable signed start, or the exact
/// refusal, from the very same canonical gate admission checks.
pub fn next(root: &Path,state: &KbdStateV2) -> Result<Value> {
    let phase = state.active_path.phase_id.as_deref().ok_or_else(|| fail("no active phase"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,true)?;
    for (index,start) in selection.starts.iter().enumerate() {
        let scope = scope_id(&digest,&start.id);
        if state.active_gates.values().any(|g| g.scope == scope) {
            return Err(fail(format!("selected start {} is unsettled; use gate selected-complete only with its independently signed receipt; no retry", start.id)));
        }
        if let Some(receipt) = state.latest_gate_receipts.values().find(|r| r.scope == scope) {
            require(receipt.kind == GateKind::SelectedIntegration && receipt.gate_id == consumption_id(&scope)
                && receipt.outcome == crate::GateOutcome::Passed && selected_exec::settled(state,receipt,&start.exec,&selection.candidate_manifest), &format!("selected start {} did not settle successfully; no automatic retry", start.id))?;
            continue;
        }
        selected_start(state,phase,&digest,&selection,&start.id)?;
        return Ok(serde_json::json!({"status":"READY_SELECTED_START","phaseId":phase,"contractDigest":digest,
            "selectedScenarioIds":selection.selected_scenario_ids,"startId":start.id,
            "scope":format!("{REQUEST_PREFIX}{}",start.id),"argv":start.argv,"cwd":start.cwd,
            "remainingStartIds":selection.starts[index..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            "requestPath":start.exec.request.path,"requestHash":start.exec.request_hash,"receiptPath":start.exec.receipt_path,"artifactRoot":start.exec.artifact_root,
            "note":"Reserve with kbd gate selected-reserve --start <start-id>, then use kbd gate selected-dispatch --start <start-id>. Only the connected authoritative gate may submit the exact prepared request. Reconcile with --reconcile-only if uncertain, then selected-complete."}));
    }
    Ok(serde_json::json!({"status":"SELECTED_STARTS_RECORDED","phaseId":phase,"contractDigest":digest,
        "note":"No selected start remains. Reconcile actual boundary evidence and the complete parent before canonical closure; gate exits are not product PASS."}))
}

/// Existing project launchers use the active reservation, not a caller's
/// proposed packet. This does not create a reservation or grant another start.
pub fn active(root: &Path,state: &KbdStateV2,start_id: &str) -> Result<Value> {
    let phase = state.active_path.phase_id.as_deref().ok_or_else(|| fail("no active phase"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,true)?;
    let start = selection.starts.iter().find(|s| s.id == start_id).ok_or_else(|| fail("unknown selected start"))?;
    let scope = scope_id(&digest,start_id);
    let gate = state.active_gates.get(&consumption_id(&scope)).ok_or_else(|| fail("no active canonical selected reservation"))?;
    check_replay_gate(state,gate)?;
    require(gate.command_sha256 == argv_digest(&start.argv) && state.phase_closure.consumed_manifests.contains(&gate.id),
        "active gate is not the signed reserved command")?;
    Ok(serde_json::json!({"status":"ACTIVE_SELECTED_START","phaseId":phase,"contractDigest":digest,
        "gateId":gate.id,"scope":scope,"startId":start.id,"commandId":start.command_id,"argv":start.argv,
        "cwd":start.cwd,"scenarioIds":start.scenario_ids,"selectedScenarioIds":selection.selected_scenario_ids,
        "boundary":selection.boundary,"maxSeconds":start.max_seconds,
        "candidateManifest":{"path":selection.candidate_manifest.path,"sha256":selection.candidate_manifest.sha256},
        "schedule":{"path":selection.schedule.path,"sha256":selection.schedule.sha256},
        "guard":{"path":start.guard.path,"sha256":start.guard.sha256},
        "requestPath":start.exec.request.path,"requestHash":start.exec.request_hash,"receiptPath":start.exec.receipt_path,"artifactRoot":start.exec.artifact_root,
        "note":"Use gate selected-dispatch --start <start-id>; --reconcile-only never dispatches. The exact signed terminal receipt and artifacts must be independently verified before selected-complete."}))
}

/// Replay validates the canonical binding identity and one-use namespace
/// without filesystem or signature-process side effects.
pub(crate) fn check_replay_gate(state: &KbdStateV2,gate: &GateRun) -> Result<()> {
    let phase = gate.phase_id.as_deref().ok_or_else(|| fail("missing selected phase"))?;
    let tail = gate.scope.strip_prefix(RESERVED_PREFIX).ok_or_else(|| fail("invalid selected scope"))?;
    let (digest,start) = tail.split_once(':').ok_or_else(|| fail("invalid selected scope"))?;
    let entry = state.phase_closure.phases.get(phase).ok_or_else(|| fail("missing selected parent"))?;
    let binding = entry.binding.as_ref().ok_or_else(|| fail("missing selected binding"))?;
    let authority = binding.authority.as_ref().ok_or_else(|| fail("missing selected binding authority"))?;
    let manifest = crate::scope_authority::verify_record(authority)?;
    require(id_valid(start) && gate.kind == GateKind::SelectedIntegration
        && gate.id == consumption_id(&gate.scope)
        && state.active_path.phase_id.as_deref() == Some(phase) && entry.terminal.is_none()
        && binding.contract_digest == digest && manifest.action == crate::scope_authority::ScopeAction::AmendSelectedExecution,
        "selected gate does not bind the signed active parent and writer floor")
}

/// Used by the existing CLI gate, and rechecked under the canonical journal
/// lock on GateStart. No effect is launched by this function.
pub fn prepare(root: &Path,state: &KbdStateV2,start_id: &str,argv: &[String]) -> Result<SelectedInvocation> {
    let phase = state.active_path.phase_id.as_deref().ok_or_else(|| fail("no active phase"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,false)?;
    let start = selected_start(state,phase,&digest,&selection,start_id)?;
    require(start.argv == argv, "argv differs from exact signed selected start")?;
    let scope = scope_id(&digest,start_id);
    let request = start.exec.validate(&root,phase,start_id,&selection.candidate_manifest,&start.guard,&selection.source_digests,&start.argv,&start.cwd,start.max_seconds)?;
    start.exec.admit_start(&root,&request,Utc::now(),selection.expires_at,start.max_seconds)?;
    Ok(SelectedInvocation { gate_id: consumption_id(&scope),scope,exec_request_identity: selected_exec::executable(&start.exec) })
}

// A selected handoff must not be circumvented by asking the ordinary gate
// to execute its argv (or an unrelated command). Verify the current bound
// snapshot, not a mutable contract view. Whole-scope gates in other phases
// retain their existing path; missing or stale bound snapshots fail closed.
pub fn refuse_stock_gate_in_selected_scope(root: &Path,state: &KbdStateV2) -> Result<()> {
    let Some(phase) = state.active_path.phase_id.as_deref() else { return Ok(()); };
    let Some(binding) = state.phase_closure.phases.get(phase).and_then(|entry| entry.binding.as_ref()) else { return Ok(()); };
    let root = root.canonicalize()?;
    let contract = snapshot(&root,state,phase,&binding.contract_digest)?;
    require(contract["identity"]["phaseId"] == phase && contract["identity"]["runId"] == state.run_id
        && contract["identity"]["projectId"] == state.project_id, "bound contract identity mismatch")?;
    require(contract["boundaryPacket"].get(KEY).is_none(),
        "current signed selected handoff requires gate selected-next and the exact selected gate; stock gate cannot launch it")
}

/// Host-side canonical admission, not a CLI-only check.
pub(crate) fn admit_gate(root: &Path,state: &KbdStateV2,gate: &GateRun) -> Result<()> {
    if !is_reserved(&gate.scope) && gate.kind != GateKind::SelectedIntegration { return refuse_stock_gate_in_selected_scope(root,state); }
    let phase = gate.phase_id.as_deref().ok_or_else(|| fail("missing gate phase"))?;
    require(gate.kind == GateKind::SelectedIntegration && state.active_path.phase_id.as_deref() == Some(phase), "selected start is not an active integration gate")?;
    let tail = gate.scope.strip_prefix(RESERVED_PREFIX).ok_or_else(|| fail("unresolved selected scope"))?;
    let (bound,start_id) = tail.split_once(':').ok_or_else(|| fail("invalid selected scope"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,false)?;
    require(bound == digest && gate.id == consumption_id(&gate.scope), "stale scope or gate identity")?;
    let start = selected_start(state,phase,&digest,&selection,start_id)?;
    require(gate.command_sha256 == argv_digest(&start.argv) && gate.executable == selected_exec::executable(&start.exec), "gate does not bind signed argv and Exec request")?;
    require(!state.phase_closure.consumed_manifests.contains(&selected_exec::start_token(gate)?), "Exec request already consumed")?;
    let request = start.exec.validate(&root,phase,start_id,&selection.candidate_manifest,&start.guard,&selection.source_digests,&start.argv,&start.cwd,start.max_seconds)?;
    require(gate.started_at <= Utc::now() && (Utc::now()-gate.started_at).num_seconds() <= 30,"stale selected reservation timestamp")?;
    start.exec.verify_installed_identity()?;
    start.exec.admit_start(&root,&request,gate.started_at,selection.expires_at,start.max_seconds)
}

/// Recheck current signed identity and source bytes immediately before spawn;
/// a refused or interrupted reservation remains consumed.
pub fn recheck_active(root: &Path,state: &KbdStateV2,gate: &GateRun,argv: &[String]) -> Result<()> {
    let phase = gate.phase_id.as_deref().ok_or_else(|| fail("missing gate phase"))?;
    require(state.active_gates.get(&gate.id) == Some(gate) && state.active_path.phase_id.as_deref() == Some(phase),
        "selected reservation is not current")?;
    let tail = gate.scope.strip_prefix(RESERVED_PREFIX).ok_or_else(|| fail("invalid selected scope"))?;
    let (bound,start_id) = tail.split_once(':').ok_or_else(|| fail("invalid selected scope"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,false)?;
    require(bound == digest && selection.starts.iter().any(|s| s.id == start_id && s.argv == argv),
        "selected candidate or argv changed before dispatch")?;
    Ok(())
}


/// Construct a reservation for the existing GateStart command. This does
/// not dispatch an execution or authorize an arbitrary child process.
pub fn reservation(root: &Path,state: &KbdStateV2,start_id: &str) -> Result<GateRun> {
    let phase = state.active_path.phase_id.as_deref().ok_or_else(|| fail("no active phase"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,false)?;
    let start = selected_start(state,phase,&digest,&selection,start_id)?;
    let invocation = prepare(&root,state,start_id,&start.argv)?;
    Ok(GateRun {
        id: invocation.gate_id, kind: GateKind::SelectedIntegration,
        scope: invocation.scope, phase_id: Some(phase.to_owned()), source_revision: state.revision,
        executable: invocation.exec_request_identity, command_sha256: argv_digest(&start.argv),
        worktree: root.to_string_lossy().into_owned(), target_dir: None, sccache_available: false,
        started_at: Utc::now(),
    })
}
/// Build the only admissible selected finish from the independent terminal
/// receipt. Reconciliation never dispatches another effect.
pub fn verified_completion(root: &Path,state: &KbdStateV2,start_id: &str) -> Result<crate::GateReceipt> {
    let phase = state.active_path.phase_id.as_deref().ok_or_else(|| fail("no active phase"))?;
    let root = root.canonicalize()?;
    let (digest,selection) = load_selection(&root,state,phase,true)?;
    let start = selection.starts.iter().find(|s| s.id == start_id).ok_or_else(|| fail("unknown selected start"))?;
    let scope = scope_id(&digest,start_id);
    let gate = state.active_gates.get(&consumption_id(&scope)).ok_or_else(|| fail("no consumed active selected attempt"))?;
    check_replay_gate(state,gate)?;
    require(gate.command_sha256 == argv_digest(&start.argv) && state.phase_closure.consumed_manifests.contains(&gate.id)
        && state.phase_closure.consumed_manifests.contains(&selected_exec::start_token(gate)?),
        "selected gate does not bind consumed attempt and exact signed start")?;
    let request = start.exec.validate(&root,phase,start_id,&selection.candidate_manifest,&start.guard,&selection.source_digests,&start.argv,&start.cwd,start.max_seconds)?;
    start.exec.completion(&root,gate,&request,&selection.candidate_manifest,selection.expires_at,start.max_seconds)
}
pub(crate) fn admit_finish(root: &Path,state: &KbdStateV2,receipt: &crate::GateReceipt) -> Result<()> {
    if !is_reserved(&receipt.scope) && receipt.kind != GateKind::SelectedIntegration { return Ok(()); }
    let (_,start_id) = receipt.scope.strip_prefix(RESERVED_PREFIX).and_then(|s| s.split_once(':'))
        .ok_or_else(|| fail("invalid selected finish scope"))?;
    let verified = verified_completion(root,state,start_id)?;
    require(&verified == receipt, "selected gate finish differs from independently verified terminal receipt")
}

/// First supported lane is the connected authoritative host. A new selected
/// event discovered in a peer import is not proof of host verification.
pub(crate) fn requires_host_admission(event: &crate::Event) -> Result<bool> {
    use crate::EventKind;
    match &event.kind {
        EventKind::PhaseContractBound { binding, .. } => {
            if let Some(record) = &binding.authority {
                Ok(crate::scope_authority::verify_record(record)?.action == crate::scope_authority::ScopeAction::AmendSelectedExecution)
            } else { Ok(false) }
        }
        EventKind::GateStarted { gate } => Ok(is_reserved(&gate.scope) || gate.kind == GateKind::SelectedIntegration),
        EventKind::GateFinished { receipt } => Ok(is_reserved(&receipt.scope) || receipt.kind == GateKind::SelectedIntegration),
        _ => Ok(false),
    }
}
pub(crate) fn refuse_unverified_import(event: &crate::Event) -> Result<()> {
    require(!requires_host_admission(event)?, "new selected authority or gate event cannot be imported as a peer assertion; use the connected authoritative host with verified evidence")
}

/// Dispatch parameters derived exclusively from the canonical active reservation.
/// No caller-selected socket, input, deadline, signer or argv is accepted.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SelectedDispatch {
    pub socket:PathBuf,pub state_dir:PathBuf,pub identity:PathBuf,pub request:PathBuf,
    pub expected_request_hash:String,pub code:PathBuf,pub inputs:Vec<String>,pub receipt_out:PathBuf,
    pub artifact_root:PathBuf,pub reservation_started_at:DateTime<Utc>,pub complete_before:DateTime<Utc>,
    pub expected_runner_key:String,pub reconcile_only:bool,
}
pub fn dispatch_plan(root:&Path,state:&KbdStateV2,start_id:&str,reconcile_only:bool)->Result<SelectedDispatch>{
    if !reconcile_only { require_reviewed_dispatch_fence()?; }
    let phase=state.active_path.phase_id.as_deref().ok_or_else(||fail("no active phase"))?;
    let root=root.canonicalize()?;
    let (digest,selection)=load_selection(&root,state,phase,reconcile_only)?;
    let start=selection.starts.iter().find(|s|s.id==start_id).ok_or_else(||fail("unknown selected start"))?;
    let scope=scope_id(&digest,start_id);
    let gate=state.active_gates.get(&consumption_id(&scope)).ok_or_else(||fail("no consumed selected reservation"))?;
    check_replay_gate(state,gate)?;
    require(gate.command_sha256==argv_digest(&start.argv)
        && state.phase_closure.consumed_manifests.contains(&gate.id)
        && state.phase_closure.consumed_manifests.contains(&selected_exec::start_token(gate)?),
        "selected dispatch is not the exact consumed request")?;
    let request=start.exec.validate(&root,phase,start_id,&selection.candidate_manifest,&start.guard,&selection.source_digests,&start.argv,&start.cwd,start.max_seconds)?;
    start.exec.dispatch(&root,gate,&request,selection.expires_at,start.max_seconds,reconcile_only)
}
