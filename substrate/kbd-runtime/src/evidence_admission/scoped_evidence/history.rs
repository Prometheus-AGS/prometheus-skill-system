//! Scoped transport history payload and deterministic append/replay groundwork.
//! Uses the existing canonical signed event ledger as its intended durability
//! boundary. No second file/ledger, operational event kind, writer or mutation
//! is introduced. A verified transport fact grants no product evidence credit.

use super::transport::{AttemptEnvelope, ReplayVerifiedTransport, TransportItemKey, TransportRecord};
use super::super::{refused, sha256_hex};
use crate::{phase_closure::is_digest, Result};
use std::collections::{BTreeMap, BTreeSet};

/// The intended position comes from the existing canonical signed journal
/// after signature, authority, ordering and compatibility checks. This type
/// is private and has no caller-controlled deserialization.
#[derive(Clone)]
struct VerifiedEventPosition {
    revision: u64,
    event_id: String,
    integrity_hash: String,
}

/// Only the future canonical event reducer may construct this after the
/// existing journal has authenticated the signed event, accepted protocol,
/// exact payload and prior host artifact-verification fact. No wire input or
/// raw TransportRecord can be promoted to this type.
pub(super) struct AuthenticatedTransportEvent {
    position: VerifiedEventPosition,
    record: TransportRecord,
}
impl AuthenticatedTransportEvent {
    pub(super) fn record(&self) -> &TransportRecord { &self.record }
}

struct HistoryEntry {
    position: VerifiedEventPosition,
    record: TransportRecord,
    record_sha256: String,
    replay_of: Option<usize>,
}

/// The canonical replay projection must be rebuilt from the complete
/// authenticated event stream, not an editable checkpoint or partial packet.
/// Its stored record payload is serializable for future signed journal
/// integration. The projection itself is not an authoritative wire format.
struct ScopedTransportHistory {
    project_id: String,
    run_id: String,
    phase_id: String,
    entries: Vec<HistoryEntry>,
    event_ids: BTreeSet<String>,
    attempt_envelopes: BTreeMap<String, AttemptEnvelope>,
    first_item: BTreeMap<TransportItemKey, usize>,
    // An exact signed request/receipt can cover multiple requirements in
    // the same reserved attempt, but never transfer into another attempt.
    request_attempts: BTreeMap<String, String>,
    receipt_attempts: BTreeMap<String, String>,
    conflicting_attempts: BTreeSet<String>,
}

impl ScopedTransportHistory {
    /// Adds one verified record. Only distinct authenticated journal events
    /// may append; exact repeats retain reference history, while a conflicting
    /// attempt preserves both records and cannot be treated as a fresh attempt.
    fn append(&mut self, verified: ReplayVerifiedTransport<'_>) -> Result<()> {
        let event = verified.event();
        let position = event.position.clone();
        let record = event.record.clone();
        let reject = |why: &str| refused(&self.phase_id, format!("scoped-history:{why}"));
        if record.project_id() != self.project_id || record.run_id() != self.run_id
            || record.phase_id() != self.phase_id
            || !super::name_is_valid(&position.event_id) || !is_digest(&position.integrity_hash)
            || position.revision == 0
            || self.entries.last().is_some_and(|entry| entry.position.revision >= position.revision)
            || self.event_ids.contains(&position.event_id)
        {
            return Err(reject("event-or-run-identity-mismatch"));
        }
        let bytes = serde_jcs::to_vec(&record).map_err(|_| reject("record-canonicalization"))?;
        let record_sha256 = sha256_hex(&bytes);
        let attempt_id = record.attempt_id().to_owned();
        let envelope = record.attempt_envelope();
        if let Some(previous) = self.attempt_envelopes.get(&attempt_id) {
            if previous != &envelope { self.conflicting_attempts.insert(attempt_id.clone()); }
        } else { self.attempt_envelopes.insert(attempt_id.clone(), envelope); }

        // Distinct applicable checks are not conflicting merely because they
        // are in the same finite attempt. Reuse of the same check with different
        // exact content remains a conflict, never a fresh permission.
        let item = record.item_key();
        let mut replay_of = None;
        if let Some(&previous) = self.first_item.get(&item) {
            if self.entries[previous].record_sha256 == record_sha256 && self.entries[previous].record == record {
                replay_of = Some(previous);
            } else {
                self.conflicting_attempts.insert(attempt_id.clone());
            }
        } else { self.first_item.insert(item, self.entries.len()); }

        for (index, digest) in [
            (&mut self.request_attempts, record.request_hash()),
            (&mut self.receipt_attempts, record.receipt_hash()),
        ] {
            let key = digest.as_str().to_owned();
            if let Some(previous) = index.get(&key) {
                if previous != &attempt_id {
                    self.conflicting_attempts.insert(previous.clone());
                    self.conflicting_attempts.insert(attempt_id.clone());
                }
            } else { index.insert(key, attempt_id.clone()); }
        }
        self.event_ids.insert(position.event_id.clone());
        self.entries.push(HistoryEntry { position, record, record_sha256, replay_of });
        Ok(())
    }

    fn require_reconciled(&self) -> Result<()> {
        if self.conflicting_attempts.is_empty() { Ok(()) }
        else {
            Err(refused(&self.phase_id, format!("scoped-history:conflicting-attempts:{:?}",
                self.conflicting_attempts)))
        }
    }
}
