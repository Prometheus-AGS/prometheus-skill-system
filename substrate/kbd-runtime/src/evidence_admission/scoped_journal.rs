//! Canonical scoped-evidence journal wire shape and replay invariants.
//! This is not an execution grant or an observer verifier. In this release
//! increment no native product observer/writer floor is independently
//! qualified, so operational scoped events are refused unconditionally.
//! The typed journal record makes the future protocol distinct from legacy
//! closure; no caller, raw append, checkpoint flag or document can activate it.

use super::{refused, sha256_hex};
use super::scoped_evidence::{derive_for_close, validate_selection, ScopedPacketSelection, SelectionReviewBinding, BoundObservationIdentity, ObligationKey, Obligations, Observation, ReceiptItemId, RepairResolution, ScopedClosureEvidence};
use crate::phase_closure::{is_digest, ContractBinding, ScenarioOutcome};
use crate::scope_authority::{self, ScopeAuthorityInput, ScopeAuthorityRecord, SCOPE_AUTHORITY_NAMESPACE};
use crate::{ActorKind, Event, EventKind, KbdStateV2, Result};
use serde::{Deserialize, Serialize};

const JOURNAL_SCHEMA: &str = "kbd.scoped-evidence-journal/v1";
const REPAIR_KIND: &str = "kbd.scoped-evidence-repair-authority";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ParentIdentity {
    project_id: String,
    run_id: String,
    phase_id: String,
    contract_digest: String,
    binding_revision: u32,
}

impl ParentIdentity {
    fn matches(&self, state: &KbdStateV2, phase_id: &str, binding: &ContractBinding) -> bool {
        self.project_id == state.project_id
            && self.run_id == state.run_id
            && self.phase_id == phase_id
            && self.contract_digest == binding.contract_digest
            && self.binding_revision == binding.binding_revision
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReceiptItemIdentity {
    receipt_sha256: String,
    item_sha256: String,
}

impl ReceiptItemIdentity {
    fn valid(&self) -> bool {
        is_digest(&self.receipt_sha256) && is_digest(&self.item_sha256)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductObservationRecord {
    scenario_id: String,
    scenario_digest: String,
    target_id: String,
    target_identity_sha256: String,
    requirements_sha256: String,
    candidate_sha256: String,
    attempt_id: String,
    attempt_scope_sha256: String,
    receipt: ReceiptItemIdentity,
    producer_profile_sha256: String,
    // Exact existing independently verified observer manifest, not a
    // process-exit status or arbitrary harness-authored boundary-results.
    observer_manifest_sha256: String,
    outcome: ScenarioOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BoundObserver {
    target_id: String,
    target_identity_sha256: String,
    profile_sha256: String,
    observer_key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnattributedObservationRecord {
    receipt_sha256: Option<String>,
    recorded_outcome: ScenarioOutcome,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ActivationRecord {
    interpretation_sha256: String,
    writer_compatibility_sha256: String,
    // Independently qualified target observers; never infer a native target
    // identity from the same receipt's platform label or reuse one target
    // as proof of another.
    observers: Vec<BoundObserver>,
    // Distinct signed operator-approved interpretation/activation; no
    // scope or execution permission is created by a document.
    authority: ScopeAuthorityRecord,
}

/// Observation-accounting reference to an existing independently authorized
/// attempt, not an execution reservation or permission. The future producer
/// must verify the real authority and durable effect reservation separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AttemptRecord {
    attempt_id: String,
    candidate_sha256: String,
    attempt_scope_sha256: String,
    existing_authority_sha256: String,
    selection_review: SelectionReviewBinding,
    selection: ScopedPacketSelection,
    repair_authority_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RepairRequest {
    schema_version: String,
    kind: String,
    namespace: String,
    action: String,
    parent: ParentIdentity,
    failed_receipt: ReceiptItemIdentity,
    failed_attempt_id: String,
    replacement_attempt_id: String,
    replacement_candidate_sha256: String,
    approver: String,
    timestamp: String,
    nonce: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RepairResolutionRecord {
    authority_manifest_sha256: String,
    replacement_receipt: ReceiptItemIdentity,
}

/// Untrusted wire payload until the existing signed journal admission and
/// real qualified observer/authority verifier have authenticated it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "payload", rename_all = "kebab-case")]
enum Entry {
    ProtocolActivated(ActivationRecord),
    AttemptRecorded(AttemptRecord),
    ObservationRecorded(ProductObservationRecord),
    UnattributedObservationRecorded(UnattributedObservationRecord),
    RepairAuthorityRecorded(ScopeAuthorityRecord),
    RepairResolutionRecorded(RepairResolutionRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopedJournalRecord {
    schema: String,
    parent: ParentIdentity,
    entry: Entry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecordedItem {
    event_id: String,
    event_integrity_hash: String,
    revision: u64,
    record: ScopedJournalRecord,
}

/// Projection rebuilt from the existing canonical journal, not a second
/// persisted authority. The new event kind is mandatory for activation;
/// checkpoint/default fields never create a scoped protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopedJournalState {
    parent: ParentIdentity,
    activation: ActivationRecord,
    // Activation itself is retained as the first authenticated journal item.
    items: Vec<RecordedItem>,
}

fn valid_name(s: &str) -> bool {
    !s.is_empty() && s.trim() == s && !s.chars().any(char::is_control)
}

fn parse_repair(raw: &str) -> Result<RepairRequest> {
    let fail = |why: &str| crate::RuntimeError::ScopeAuthority(format!("scoped-repair:{why}"));
    if raw.chars().any(char::is_control) {
        return Err(fail("nonprintable-manifest"));
    }
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|_| fail("invalid-json"))?;
    if serde_jcs::to_string(&value).map_err(|_| fail("canonicalization"))? != raw {
        return Err(fail("noncanonical-manifest"));
    }
    let request: RepairRequest = serde_json::from_value(value).map_err(|_| fail("invalid-shape"))?;
    if request.schema_version != "1" || request.kind != REPAIR_KIND
        || request.namespace != SCOPE_AUTHORITY_NAMESPACE || request.action != "repair-evidence-debt"
        || !request.failed_receipt.valid()
        || !is_digest(&request.parent.contract_digest)
        || request.parent.binding_revision == 0
        || !is_digest(&request.replacement_candidate_sha256)
        || request.failed_attempt_id == request.replacement_attempt_id
        || request.reason.trim().chars().count() < 10
        || chrono::DateTime::parse_from_rfc3339(&request.timestamp).is_err()
        || ![request.approver.as_str(), request.nonce.as_str(),
            request.parent.project_id.as_str(), request.parent.run_id.as_str(),
            request.parent.phase_id.as_str(), request.failed_attempt_id.as_str(),
            request.replacement_attempt_id.as_str()].into_iter().all(valid_name)
    {
        return Err(fail("invalid-exact-repair-authority"));
    }
    Ok(request)
}

/// Admission reuses the existing operator-owned SSH verifier. This authorizes
/// resolving one exact evidence debt after an already separately authorized
/// repair, not B4/B5 execution, fresh work, retries or cleanup. No command
/// or signing endpoint is introduced by this code.
fn verify_repair_authority(input: &ScopeAuthorityInput) -> Result<(RepairRequest, ScopeAuthorityRecord)> {
    let request = parse_repair(&input.manifest_raw)?;
    let record = scope_authority::verify_canonical_payload(input, &request.approver)?;
    Ok((request, record))
}

fn replay_repair_authority(record: &ScopeAuthorityRecord) -> Result<RepairRequest> {
    scope_authority::verify_record_identity(record)?;
    let request = parse_repair(&record.manifest_raw)?;
    if request.approver != record.approver || !is_digest(&record.allowed_signers_sha256)
        || record.signature.is_empty()
    {
        return Err(crate::RuntimeError::ScopeAuthority("scoped-repair:stored-authority-mismatch".into()));
    }
    Ok(request)
}

/// Only future canonical qualified host verification and signed replay may
/// construct this witness. It is deliberately not deserializable, and there
/// is no producer before the actual native observer and writer are qualified.
struct VerifiedJournalEvent<'a> {
    event: &'a Event,
    record: &'a ScopedJournalRecord,
}

impl ScopedJournalState {
    /// Pure append-only fold of already authenticated events. Exact item
    /// history, distinct attempts, explicit repair pre-authorization and
    /// replacement ordering must be established; no proof is inferred from
    /// a newly supplied PASS string. The producer is the missing prerequisite.
    fn retain(&mut self, verified: VerifiedJournalEvent<'_>) -> Result<()> {
        let record = verified.record;
        let event = verified.event;
        let fail = |why: &str| refused(&self.parent.phase_id, format!("scoped-history:{why}"));
        let matching_payload = matches!(&event.kind, EventKind::ScopedEvidenceJournalRecorded {
            phase_id, record: actual
        } if phase_id == &self.parent.phase_id && actual.as_ref() == record);
        if !matching_payload || record.schema != JOURNAL_SCHEMA || record.parent != self.parent
            || event.project_id != self.parent.project_id || event.run_id != self.parent.run_id
            || event.schema_version == "1" || !is_digest(&event.integrity_hash)
            || !valid_name(&event.event_id)
            || self.items.last().is_some_and(|item| event.revision <= item.revision)
            || self.items.iter().any(|item| item.event_id == event.event_id)
        {
            return Err(fail("journal-event-identity"));
        }
        match &record.entry {
            Entry::ProtocolActivated(_) => return Err(fail("duplicate-activation")),
            Entry::AttemptRecorded(attempt) => {
                attempt.selection_review.check_packet(&attempt.selection)?;
                if !valid_name(&attempt.attempt_id)
                    || !is_digest(&attempt.candidate_sha256)
                    || !is_digest(&attempt.attempt_scope_sha256)
                    || !is_digest(&attempt.existing_authority_sha256)
                    || attempt.selection.digest()? != attempt.attempt_scope_sha256
                    || attempt.selection.candidate() != attempt.candidate_sha256
                    || attempt.selection.parent_contract() != self.parent.contract_digest
                    || attempt.selection.interpretation() != self.activation.interpretation_sha256
                    || self.items.iter().any(|item| matches!(&item.record.entry, Entry::AttemptRecorded(old)
                        if old.attempt_id == attempt.attempt_id))
                {
                    return Err(fail("attempt-identity-or-replay"));
                }
                if let Some(repair_digest) = &attempt.repair_authority_sha256 {
                    let repair = self.items.iter().find_map(|item| match &item.record.entry {
                        Entry::RepairAuthorityRecorded(authority) if &authority.manifest_sha256 == repair_digest =>
                            Some(replay_repair_authority(authority)),
                        _ => None,
                    }).ok_or_else(|| fail("repair-authority-before-attempt-missing"))??;
                    if repair.replacement_attempt_id != attempt.attempt_id
                        || repair.replacement_candidate_sha256 != attempt.candidate_sha256
                    {
                        return Err(fail("repair-attempt-identity-mismatch"));
                    }
                }
            }
            Entry::ObservationRecorded(observation) => {
                let attempt = self.items.iter().find_map(|item| match &item.record.entry {
                    Entry::AttemptRecorded(attempt) if attempt.attempt_id == observation.attempt_id => Some(attempt),
                    _ => None,
                }).ok_or_else(|| fail("observation-before-attempt-record"))?;
                if observation.candidate_sha256 != attempt.candidate_sha256
                    || observation.attempt_scope_sha256 != attempt.attempt_scope_sha256
                    || !attempt.selection.contains(&observation.scenario_id, &observation.target_id)
                {
                    return Err(fail("observation-attempt-mismatch"));
                }
                let mut observers = self.activation.observers.iter().filter(|observer| observer.target_id == observation.target_id);
                let observer = observers.next().ok_or_else(|| fail("target-observer-not-bound"))?;
                if observers.next().is_some()
                    || event.signer_key_id.as_deref() != Some(observer.observer_key_id.as_str())
                    || observation.target_identity_sha256 != observer.target_identity_sha256
                    || observation.producer_profile_sha256 != observer.profile_sha256
                    || !observation.receipt.valid() || !is_digest(&observation.observer_manifest_sha256)
                    || !is_digest(&observation.candidate_sha256)
                    || !is_digest(&observation.scenario_digest)
                    || !is_digest(&observation.target_identity_sha256)
                    || !is_digest(&observation.requirements_sha256)
                    || !is_digest(&observation.attempt_scope_sha256)
                    || ![observation.scenario_id.as_str(), observation.target_id.as_str(),
                        observation.attempt_id.as_str()].into_iter().all(valid_name)
                    || observation.outcome.id != observation.scenario_id
                    || observation.outcome.scenario_digest.as_deref() != Some(observation.scenario_digest.as_str())
                    || observation.outcome.carried_from_contract.is_some()
                    || !matches!(observation.outcome.status.as_str(), "PASS" | "FAIL" | "BLOCKED" | "NOT_EXERCISED")
                {
                    return Err(fail("observation-attribution"));
                }
                // Preserve a conflicting observation in the future verified
                // canonical history; it can never be silently overwritten.
                // Full exact parent target/candidate/receipt evaluation is
                // still performed by the scoped coverage fold at closure.
            }
            Entry::UnattributedObservationRecorded(observation) => {
                if !self.activation.observers.iter().any(|observer|
                        event.signer_key_id.as_deref() == Some(observer.observer_key_id.as_str()))
                    || !valid_name(&observation.reason)
                    || observation.receipt_sha256.as_ref().is_some_and(|digest| !is_digest(digest))
                {
                    return Err(fail("unattributed-observation-record"));
                }
                // Keep historical claim even if it says PASS; it has no
                // canonical obligation/candidate attribution or current credit.
            }
            Entry::RepairAuthorityRecorded(authority) => {
                if event.actor.kind != ActorKind::Operator {
                    return Err(fail("repair-operator-authority"));
                }
                let request = replay_repair_authority(authority)?;
                if request.parent != self.parent
                    || !self.has_observation(&request.failed_receipt, &request.failed_attempt_id, "FAIL", None)
                    || self.items.iter().any(|item| matches!(&item.record.entry, Entry::RepairAuthorityRecorded(old)
                        if old.manifest_sha256 == authority.manifest_sha256))
                {
                    return Err(fail("repair-authority-binding-or-replay"));
                }
                // The exact signed repair authorization must precede the
                // recorded replacement attempt, not just its final outcome.
                if self.items.iter().any(|item| match &item.record.entry {
                    Entry::AttemptRecorded(old) => old.attempt_id == request.replacement_attempt_id,
                    Entry::ObservationRecorded(old) => old.attempt_id == request.replacement_attempt_id,
                    _ => false,
                }) {
                    return Err(fail("repair-authority-after-attempt"));
                }
            }
            Entry::RepairResolutionRecorded(resolution) => {
                let mut authorization = None;
                for (index, item) in self.items.iter().enumerate() {
                    if let Entry::RepairAuthorityRecorded(authority) = &item.record.entry {
                        if authority.manifest_sha256 == resolution.authority_manifest_sha256 {
                            authorization = Some((index, replay_repair_authority(authority)?));
                        }
                    }
                    if matches!(&item.record.entry, Entry::RepairResolutionRecorded(old)
                        if old.authority_manifest_sha256 == resolution.authority_manifest_sha256) {
                        return Err(fail("repair-resolution-replay"));
                    }
                }
                let (authority_index, request) = authorization.ok_or_else(|| fail("repair-authority-missing"))?;
                let failed = self.unique_observation(&request.failed_receipt).ok_or_else(|| fail("repair-failed-item-ambiguous"))?;
                let replacement_attempt = self.items.iter().skip(authority_index + 1)
                    .find_map(|item| match &item.record.entry {
                        Entry::AttemptRecorded(attempt) if attempt.attempt_id == request.replacement_attempt_id => Some(attempt),
                        _ => None,
                    }).ok_or_else(|| fail("repair-replacement-attempt-missing"))?;
                let replacement = self.unique_observation(&resolution.replacement_receipt)
                    .ok_or_else(|| fail("repair-replacement-item-ambiguous"))?;
                if replacement_attempt.repair_authority_sha256.as_deref() != Some(resolution.authority_manifest_sha256.as_str())
                    || failed.outcome.status != "FAIL"
                    || failed.attempt_id != request.failed_attempt_id
                    || replacement.outcome.status != "PASS"
                    || replacement.attempt_id != request.replacement_attempt_id
                    || replacement.candidate_sha256 != request.replacement_candidate_sha256
                    || replacement.scenario_id != failed.scenario_id
                    || replacement.target_id != failed.target_id
                    || replacement.target_identity_sha256 != failed.target_identity_sha256
                    || replacement.requirements_sha256 != failed.requirements_sha256
                {
                    return Err(fail("repair-replacement-not-exact"));
                }
            }
        }
        self.items.push(RecordedItem {
            event_id: event.event_id.clone(),
            event_integrity_hash: event.integrity_hash.clone(),
            revision: event.revision,
            record: record.clone(),
        });
        Ok(())
    }

    fn unique_observation(&self, identity: &ReceiptItemIdentity) -> Option<&ProductObservationRecord> {
        let mut rows = self.items.iter().filter_map(|item| {
            match &item.record.entry {
                Entry::ObservationRecorded(observation) if &observation.receipt == identity => Some(observation),
                _ => None,
            }
        });
        let first = rows.next()?;
        rows.all(|row| row == first).then_some(first)
    }

    fn has_observation(&self, identity: &ReceiptItemIdentity, attempt: &str,
        status: &str, candidate: Option<&str>) -> bool {
        self.unique_observation(identity).is_some_and(|row| row.attempt_id == attempt
            && row.outcome.status == status
            && candidate.is_none_or(|value| row.candidate_sha256 == value))
    }
}

/// Witness of the complete authenticated scoped journal reconstructed by
/// the future qualified native replay producer. Cannot be deserialized or
/// derived from a checkpoint flag or a caller-supplied projection.
struct VerifiedJournalHistory<'a> {
    state: &'a ScopedJournalState,
}

fn derive_verified_scoped_closure(
    parent: &Obligations<'_>, verified: VerifiedJournalHistory<'_>,
) -> Result<ScopedClosureEvidence> {
    let journal = verified.state;
    let fail = |why: &str| refused(parent.phase_id, format!("scoped-closure-history:{why}"));
    if journal.parent.project_id != parent.project_id || journal.parent.run_id != parent.run_id
        || journal.parent.phase_id != parent.phase_id
        || journal.parent.contract_digest != parent.binding.contract_digest
        || journal.parent.binding_revision != parent.binding.binding_revision
        || journal.activation.interpretation_sha256 != parent.interpretation_sha256
        || !is_digest(&journal.activation.writer_compatibility_sha256)
        || !journal.items.first().is_some_and(|item| matches!(&item.record.entry,
            Entry::ProtocolActivated(activation) if activation == &journal.activation))
    {
        return Err(fail("parent-or-activation-mismatch"));
    }
    // Each required target identity and observer profile must be bound
    // exactly once by this authenticated activation, with no caller-selected
    // subset or additional target. Target requirements remain parent-derived.
    let mut targets = std::collections::BTreeMap::new();
    for row in parent.required {
        // The current canonical journal path is for the exact existing
        // native product observer. Artifact transport groundwork is not
        // accepted as a substitute for this proof.
        let profile = row.native_observer_profile_sha256.as_ref()
            .ok_or_else(|| fail("native-observer-profile-required"))?;
        let value = (&row.target_identity_sha256, profile);
        if let Some(previous) = targets.insert(row.key.target_id.as_str(), value) {
            if previous != value { return Err(fail("inconsistent-parent-target")); }
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for observer in &journal.activation.observers {
        let (identity, profile) = targets.get(observer.target_id.as_str())
            .ok_or_else(|| fail("unexpected-target-observer"))?;
        if !seen.insert(observer.target_id.as_str())
            || *identity != &observer.target_identity_sha256
            || *profile != &observer.profile_sha256
            || !is_digest(&observer.profile_sha256) || !valid_name(&observer.observer_key_id)
        {
            return Err(fail("target-observer-binding"));
        }
    }
    if !seen.iter().copied().eq(targets.keys().copied()) {
        return Err(fail("target-observer-denominator-incomplete"));
    }
    let mut observations = Vec::new();
    let mut repairs = Vec::new();
    let mut prior_authorities = std::collections::BTreeMap::new();
    for item in &journal.items {
        match &item.record.entry {
            Entry::ObservationRecorded(row) => observations.push(Observation {
                identity: Some(BoundObservationIdentity {
                    project_id: journal.parent.project_id.clone(),
                    run_id: journal.parent.run_id.clone(),
                    phase_id: journal.parent.phase_id.clone(),
                    contract_digest: journal.parent.contract_digest.clone(),
                    binding_revision: journal.parent.binding_revision,
                    candidate_sha256: row.candidate_sha256.clone(),
                    key: ObligationKey { scenario_id: row.scenario_id.clone(), target_id: row.target_id.clone() },
                    target_identity_sha256: row.target_identity_sha256.clone(),
                    requirements_sha256: row.requirements_sha256.clone(),
                    attempt_id: row.attempt_id.clone(),
                    receipt_sha256: row.receipt.receipt_sha256.clone(),
                    receipt_item_sha256: row.receipt.item_sha256.clone(),
                    producer_profile_sha256: row.producer_profile_sha256.clone(),
                    observer_manifest_sha256: row.observer_manifest_sha256.clone(),
                }),
                outcome: row.outcome.clone(),
            }),
            Entry::UnattributedObservationRecorded(row) => observations.push(Observation {
                identity: None,
                outcome: row.recorded_outcome.clone(),
            }),
            Entry::RepairAuthorityRecorded(authority) => {
                let request = replay_repair_authority(authority)?;
                if prior_authorities.insert(authority.manifest_sha256.clone(), request).is_some() {
                    return Err(fail("duplicate-repair-authority"));
                }
            }
            Entry::RepairResolutionRecorded(resolution) => {
                let authority = prior_authorities.get(&resolution.authority_manifest_sha256)
                    .ok_or_else(|| fail("repair-authorization-missing"))?;
                repairs.push(RepairResolution {
                    failed: ReceiptItemId { receipt_sha256: authority.failed_receipt.receipt_sha256.clone(),
                        item_sha256: authority.failed_receipt.item_sha256.clone() },
                    replacement: ReceiptItemId { receipt_sha256: resolution.replacement_receipt.receipt_sha256.clone(),
                        item_sha256: resolution.replacement_receipt.item_sha256.clone() },
                    authority_record_sha256: resolution.authority_manifest_sha256.clone(),
                    replacement_candidate_sha256: authority.replacement_candidate_sha256.clone(),
                });
            }
            Entry::AttemptRecorded(attempt) => {
                attempt.selection_review.check_packet(&attempt.selection)?;
                if attempt.selection.digest()? != attempt.attempt_scope_sha256
                    || attempt.selection.candidate() != attempt.candidate_sha256 {
                    return Err(fail("attempt-selection-identity"));
                }
                validate_selection(parent, &attempt.selection)?;
            }
            Entry::ProtocolActivated(_) => {},
        }
    }
    // The existing pure exact whole-parent coverage/debt and PASS checks
    // operate on the authenticated journal observations, not an editable
    // aggregate. Full effect reconciliation and budgets remain mandatory.
    let journal_bytes = serde_jcs::to_vec(journal).map_err(|_| fail("journal-canonicalization"))?;
    derive_for_close(parent, &observations, &repairs, &sha256_hex(&journal_bytes))
}

/// Canonical reducer entry. The activation gate is a deliberate refusal,
/// not an editable config flag or a claim that current old writers are safe.
/// No actual qualified native product observer/receipt and compatible writer
/// floor has yet been established. No scope grant or history may be accepted
/// until a later independently reviewed canonical producer is implemented.
pub(crate) fn apply_scoped_journal(
    state: &KbdStateV2, event: &Event, phase_id: &str, record: &ScopedJournalRecord,
) -> Result<()> {
    let binding = state.phase_closure.phases.get(phase_id)
        .and_then(|entry| entry.binding.as_ref())
        .ok_or_else(|| refused(phase_id, "scoped-history:parent-not-bound".into()))?;
    if record.schema != JOURNAL_SCHEMA || !record.parent.matches(state, phase_id, binding)
        || event.schema_version == "1" {
        return Err(refused(phase_id, "scoped-history:unsupported-version-or-parent-identity".into()));
    }
    Err(refused(phase_id, "scoped-history:activation-unqualified-no-producer-or-writer-proof".into()))
}
