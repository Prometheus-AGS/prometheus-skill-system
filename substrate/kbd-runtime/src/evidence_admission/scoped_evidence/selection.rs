//! Scoped admission selection against an immutable complete parent. The
//! packet is not a shortened adopted contract, a grant, or a closure claim.
//! The actual canonical producer must authenticate the independently reviewed
//! exact variant, prerequisites, candidate and existing finite authority.

use super::Obligations;
use super::super::{refused, sha256_hex};
use crate::{phase_closure::is_digest, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CommandRole { Prerequisite, Case, NegativeControl }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectedTarget {
    scenario_id: String,
    target_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommandSelection {
    command_id: String,
    role: CommandRole,
    // Exact independently reviewed narrowed invocation identity, including
    // actual argv/cwd/guard and physical schedule. No dynamic dispatch here.
    variant_sha256: String,
    scenario_ids: Vec<String>,
    argv: Vec<String>,
    cwd: Option<String>,
    guard_sha256: String,
    physical_schedule_sha256: String,
}

impl CommandSelection {
    fn computed_variant_digest(&self) -> Result<String> {
        let value = serde_json::json!({
            "schema": "kbd.scoped-command-variant/v1",
            "commandId": self.command_id,
            "role": self.role,
            "scenarioIds": self.scenario_ids,
            "argv": self.argv,
            "cwd": self.cwd,
            "guardSha256": self.guard_sha256,
            "physicalScheduleSha256": self.physical_schedule_sha256
        });
        let bytes = serde_jcs::to_vec(&value).map_err(|error|
            crate::RuntimeError::InvalidState(format!("scoped-command-variant:canonicalization:{error}")))?;
        Ok(sha256_hex(&bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::evidence_admission) struct ScopedPacketSelection {
    schema: String,
    parent_contract_sha256: String,
    interpretation_sha256: String,
    candidate_sha256: String,
    selected_scenario_ids: Vec<String>,
    targets: Vec<SelectedTarget>,
    commands: Vec<CommandSelection>,
    safety_check_ids: Vec<String>,
}

/// Exact input/output of the independently authenticated selection review.
/// The receipt itself must be verified by the future canonical producer; a
/// deserialized binding or its digest alone cannot establish that review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::evidence_admission) struct SelectionReviewBinding {
    review_receipt_sha256: String,
    reviewed_packet_sha256: String,
    reviewed_parent_sha256: String,
    reviewed_interpretation_sha256: String,
    reviewed_candidate_sha256: String,
}

impl SelectionReviewBinding {
    pub(in crate::evidence_admission) fn check_packet(&self, packet: &ScopedPacketSelection) -> Result<()> {
        if !is_digest(&self.review_receipt_sha256)
            || self.reviewed_packet_sha256 != packet.digest()?
            || self.reviewed_parent_sha256 != packet.parent_contract_sha256
            || self.reviewed_interpretation_sha256 != packet.interpretation_sha256
            || self.reviewed_candidate_sha256 != packet.candidate_sha256
        {
            return Err(crate::RuntimeError::InvalidState("scoped selection review does not bind exact packet".into()));
        }
        Ok(())
    }
}

impl ScopedPacketSelection {
    pub(in crate::evidence_admission) fn digest(&self) -> Result<String> {
        let bytes = serde_jcs::to_vec(self).map_err(|error|
            crate::RuntimeError::InvalidState(format!("scoped-packet:canonicalization:{error}")))?;
        Ok(sha256_hex(&bytes))
    }

    pub(in crate::evidence_admission) fn candidate(&self) -> &str { &self.candidate_sha256 }
    pub(in crate::evidence_admission) fn parent_contract(&self) -> &str { &self.parent_contract_sha256 }
    pub(in crate::evidence_admission) fn interpretation(&self) -> &str { &self.interpretation_sha256 }

    pub(in crate::evidence_admission) fn contains(&self, scenario: &str, target: &str) -> bool {
        self.targets.iter().any(|row| row.scenario_id == scenario && row.target_id == target)
    }
}

/// Exact denominator comparison; selected packet admission intentionally
/// does NOT require commands for unrelated parent obligations. Whole-parent
/// contract adoption and final coverage still use the full denominator.
/// Verified producer must separately authenticate variant semantics, actual
/// prerequisites and authority/qualification; this function creates none.
pub(in crate::evidence_admission) fn validate_selection(
    parent: &Obligations<'_>, packet: &ScopedPacketSelection,
) -> Result<()> {
    let fail = |why: &str| refused(parent.phase_id, format!("scoped-packet:{why}"));
    if packet.schema != "kbd.scoped-packet-selection/v1"
        || packet.parent_contract_sha256 != parent.binding.contract_digest
        || packet.interpretation_sha256 != parent.interpretation_sha256
        || !is_digest(&packet.candidate_sha256)
    {
        // Historical packets remain valid references to their own candidate;
        // current admission must additionally compare the candidate at use.
        return Err(fail("parent-or-candidate-identity"));
    }
    let selected: BTreeSet<_> = packet.selected_scenario_ids.iter().map(String::as_str).collect();
    if selected.is_empty() || selected.len() != packet.selected_scenario_ids.len()
        || selected.iter().any(|id| !parent.binding.scenario_digests.contains_key(*id))
    {
        return Err(fail("unknown-duplicate-or-empty-scenario-selection"));
    }

    let mut targets = BTreeSet::new();
    let mut commands = BTreeMap::<(&str,CommandRole),BTreeSet<&str>>::new();
    let mut safety = BTreeSet::new();
    for row in parent.required.iter().filter(|row| selected.contains(row.key.scenario_id.as_str())) {
        let scenario = row.key.scenario_id.as_str();
        targets.insert((scenario, row.key.target_id.as_str()));
        for (role, ids) in [
            (CommandRole::Prerequisite, &row.prerequisite_command_ids),
            (CommandRole::Case, &row.case_command_ids),
            (CommandRole::NegativeControl, &row.control_command_ids),
        ] {
            for id in ids { commands.entry((id.as_str(), role)).or_default().insert(scenario); }
        }
        safety.extend(row.safety_check_ids.iter().map(String::as_str));
    }
    let offered_targets: BTreeSet<_> = packet.targets.iter()
        .map(|row| (row.scenario_id.as_str(), row.target_id.as_str())).collect();
    if targets.is_empty() || offered_targets != targets || offered_targets.len() != packet.targets.len() {
        return Err(fail("selected-target-denominator-mismatch"));
    }
    let offered_safety: BTreeSet<_> = packet.safety_check_ids.iter().map(String::as_str).collect();
    if offered_safety != safety || offered_safety.len() != packet.safety_check_ids.len() {
        return Err(fail("selected-safety-denominator-mismatch"));
    }
    let mut offered_commands = BTreeSet::new();
    for command in &packet.commands {
        let key = (command.command_id.as_str(), command.role);
        let expected = commands.get(&key).ok_or_else(|| fail("unmapped-selected-command"))?;
        let scope: BTreeSet<_> = command.scenario_ids.iter().map(String::as_str).collect();
        if !offered_commands.insert(key) || !is_digest(&command.variant_sha256)
            || !is_digest(&command.guard_sha256) || !is_digest(&command.physical_schedule_sha256)
            || command.argv.is_empty()
            || command.computed_variant_digest()? != command.variant_sha256
            || scope.len() != command.scenario_ids.len() || &scope != expected
        {
            return Err(fail("command-variant-or-scope-mismatch"));
        }
    }
    if !offered_commands.iter().copied().eq(commands.keys().copied()) {
        return Err(fail("selected-case-control-or-prerequisite-missing"));
    }
    Ok(())
}
