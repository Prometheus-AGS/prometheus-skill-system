use fs2::FileExt;
use loro::{ExportMode, LoroDoc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    CausalFrontier, ConflictCandidate, ConflictKind, ConflictRecord, Event, EventKind, KbdStateV2,
    ReplicaHead, Result, RuntimeError,
};

pub const PROJECT_DOCUMENT_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone)]
pub struct ProjectDocument {
    root: PathBuf,
    project_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocumentStatus {
    pub path: PathBuf,
    pub bytes: u64,
    pub event_count: usize,
    pub snapshot_sha256: Option<String>,
}

impl ProjectDocument {
    pub fn open(root: impl AsRef<Path>, project_id: impl Into<String>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            project_id: project_id.into(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.root.join("project.loro")
    }

    pub fn lock_path(&self) -> PathBuf {
        self.root.join("project.loro.lock")
    }

    pub fn events(&self) -> Result<Vec<Event>> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_shared()?;
        let result = (|| {
            let doc = self.load_locked()?;
            let mut events = events_from_doc(&doc)?;
            sort_events(&mut events);
            validate_unique_events(&events, &self.project_id)?;
            Ok(events)
        })();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn fold(&self) -> Result<KbdStateV2> {
        fold_project_events(&self.events()?)
    }

    /// Insert signed events into the grow-only event map and fsync the Loro
    /// snapshot. Existing identical event IDs are idempotent; an event ID with
    /// different bytes is integrity corruption and is rejected.
    pub fn ingest_events(&self, events: &[Event]) -> Result<usize> {
        if events.is_empty() {
            return Ok(0);
        }
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = (|| {
            let doc = self.load_locked()?;
            let existing = events_from_doc(&doc)?
                .into_iter()
                .map(|event| (event.event_id.clone(), event))
                .collect::<BTreeMap<_, _>>();
            let event_map = doc.get_map("events");
            let metadata = doc.get_map("metadata");
            metadata
                .insert("schemaVersion", PROJECT_DOCUMENT_SCHEMA_VERSION)
                .map_err(loro_error)?;
            metadata
                .insert("projectId", self.project_id.as_str())
                .map_err(loro_error)?;

            let mut inserted = 0;
            let mut seen = HashSet::new();
            for event in events {
                validate_event(event, &self.project_id)?;
                if !seen.insert(&event.event_id) {
                    continue;
                }
                if let Some(committed) = existing.get(&event.event_id) {
                    if committed != event {
                        return Err(RuntimeError::DuplicateEvent(event.event_id.clone()));
                    }
                    continue;
                }
                let canonical = if event.schema_version == "1" {
                    serde_json::to_string(event)?
                } else {
                    String::from_utf8(
                        serde_jcs::to_vec(event)
                            .map_err(|error| RuntimeError::InvalidState(error.to_string()))?,
                    )
                    .map_err(|error| RuntimeError::InvalidState(error.to_string()))?
                };
                event_map
                    .insert(event.event_id.as_str(), canonical)
                    .map_err(loro_error)?;
                inserted += 1;
            }
            if inserted > 0 || !self.path().exists() {
                doc.commit();
                self.persist_locked(&doc)?;
            }
            Ok(inserted)
        })();
        FileExt::unlock(&lock)?;
        result
    }

    pub fn status(&self) -> Result<ProjectDocumentStatus> {
        let events = self.events()?;
        let path = self.path();
        let snapshot = fs::read(&path).ok();
        Ok(ProjectDocumentStatus {
            path,
            bytes: snapshot
                .as_ref()
                .map(|bytes| bytes.len() as u64)
                .unwrap_or(0),
            event_count: events.len(),
            snapshot_sha256: snapshot.map(|bytes| format!("{:x}", Sha256::digest(bytes))),
        })
    }

    pub fn export_snapshot(&self) -> Result<Vec<u8>> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_shared()?;
        let result = self
            .load_locked()?
            .export(ExportMode::Snapshot)
            .map_err(|error| RuntimeError::InvalidState(format!("project.loro: {error}")));
        FileExt::unlock(&lock)?;
        result
    }

    /// Export the complete operation history as a Loro update bundle. Peers
    /// may import this repeatedly; Loro deduplicates operations by ID.
    pub fn export_updates(&self) -> Result<Vec<u8>> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_shared()?;
        let result = self
            .load_locked()?
            .export(ExportMode::all_updates())
            .map_err(|error| RuntimeError::InvalidState(format!("project.loro: {error}")));
        FileExt::unlock(&lock)?;
        result
    }

    /// Validate and merge a peer's Loro updates, then fsync the authoritative
    /// project document. Validation happens against an isolated document
    /// before the on-disk authority is changed.
    pub fn import_updates(&self, updates: &[u8]) -> Result<(usize, KbdStateV2)> {
        fs::create_dir_all(&self.root)?;
        let lock = self.open_lock()?;
        lock.lock_exclusive()?;
        let result = (|| {
            let incoming = LoroDoc::new();
            incoming.import(updates).map_err(loro_error)?;
            let incoming_events = events_from_doc(&incoming)?;
            validate_unique_events(&incoming_events, &self.project_id)?;

            let local = self.load_locked()?;
            let before = events_from_doc(&local)?;
            let before_by_id = before
                .iter()
                .map(|event| (event.event_id.as_str(), event))
                .collect::<BTreeMap<_, _>>();
            for event in &incoming_events {
                if let Some(existing) = before_by_id.get(event.event_id.as_str()) {
                    if *existing != event {
                        return Err(RuntimeError::DuplicateEvent(event.event_id.clone()));
                    }
                }
            }
            local.import(updates).map_err(loro_error)?;
            local.commit();
            let merged = events_from_doc(&local)?;
            let merged_by_id = merged
                .iter()
                .map(|event| (event.event_id.as_str(), event))
                .collect::<BTreeMap<_, _>>();
            if before_by_id
                .iter()
                .any(|(event_id, event)| merged_by_id.get(event_id).copied() != Some(*event))
            {
                return Err(RuntimeError::InvalidState(
                    "project.loro updates attempted to mutate or delete committed events".into(),
                ));
            }
            let state = fold_project_events(&merged)?;
            let inserted = incoming_events
                .iter()
                .filter(|event| !before_by_id.contains_key(event.event_id.as_str()))
                .count();
            if inserted > 0 || !self.path().exists() {
                self.persist_locked(&local)?;
            }
            Ok((inserted, state))
        })();
        FileExt::unlock(&lock)?;
        result
    }

    fn open_lock(&self) -> Result<File> {
        Ok(OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?)
    }

    fn load_locked(&self) -> Result<LoroDoc> {
        let path = self.path();
        if !path.exists() {
            return Ok(LoroDoc::new());
        }
        let bytes = fs::read(path)?;
        LoroDoc::from_snapshot(&bytes)
            .or_else(|_| {
                let doc = LoroDoc::new();
                doc.import(&bytes)?;
                Ok::<_, loro::LoroError>(doc)
            })
            .map_err(loro_error)
    }

    fn persist_locked(&self, doc: &LoroDoc) -> Result<()> {
        let bytes = doc
            .export(ExportMode::Snapshot)
            .map_err(|error| RuntimeError::InvalidState(format!("project.loro: {error}")))?;
        let temporary = self
            .root
            .join(format!(".project.loro.{}.tmp", Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, self.path())?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}

fn events_from_doc(doc: &LoroDoc) -> Result<Vec<Event>> {
    let value = serde_json::to_value(doc.get_map("events").get_deep_value())?;
    value
        .as_object()
        .into_iter()
        .flat_map(|object| object.values())
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    RuntimeError::InvalidState(
                        "project.loro event map contains a non-string value".into(),
                    )
                })
                .and_then(|json| serde_json::from_str(json).map_err(Into::into))
        })
        .collect()
}

fn validate_unique_events(events: &[Event], project_id: &str) -> Result<()> {
    let mut ids = HashSet::new();
    for event in events {
        if !ids.insert(&event.event_id) {
            return Err(RuntimeError::DuplicateEvent(event.event_id.clone()));
        }
        validate_event(event, project_id)?;
    }
    Ok(())
}

fn validate_event(event: &Event, project_id: &str) -> Result<()> {
    if event.project_id != project_id {
        return Err(RuntimeError::ProjectMismatch {
            supplied: event.project_id.clone(),
            current: project_id.into(),
        });
    }
    if !event.actor_id.is_empty() && event.actor_id != event.actor.id {
        return Err(RuntimeError::InvalidState(format!(
            "event {} actorId does not match actor.id",
            event.event_id
        )));
    }
    event.verify_signature(&BTreeMap::new())?;
    if event.integrity_hash != event.calculate_hash()? {
        return Err(RuntimeError::Integrity {
            revision: event.revision,
        });
    }
    Ok(())
}

pub fn fold_project_events(events: &[Event]) -> Result<KbdStateV2> {
    if events.is_empty() {
        return Ok(KbdStateV2::default());
    }
    let project_id = events[0].project_id.clone();
    validate_unique_events(events, &project_id)?;
    let ordered = causal_order(events)?;
    let mut conflicts = detect_conflicts(events)?;
    conflicts.extend(detect_claim_conflicts(events)?);

    let resolutions = events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::ConflictResolved {
                conflict_id,
                winner_event_id,
                reason,
            } if event.actor.kind == crate::ActorKind::Operator => Some((
                conflict_id.clone(),
                (event, winner_event_id.clone(), reason.clone()),
            )),
            _ => None,
        })
        .fold(
            BTreeMap::<String, (&Event, String, String)>::new(),
            |mut selected, (conflict_id, resolution)| {
                let replace = selected
                    .get(&conflict_id)
                    .map(|current| event_order_key(resolution.0) > event_order_key(current.0))
                    .unwrap_or(true);
                if replace {
                    selected.insert(conflict_id, resolution);
                }
                selected
            },
        );
    for (conflict_id, (resolution, winner_event_id, reason)) in resolutions {
        let Some(conflict) = conflicts.get_mut(&conflict_id) else {
            continue;
        };
        if conflict
            .candidates
            .iter()
            .any(|candidate| candidate.event_id == winner_event_id)
        {
            conflict.winner_event_id = winner_event_id;
            conflict.resolved_by_event_id = Some(resolution.event_id.clone());
            conflict.resolution_reason = Some(reason);
        }
    }

    let losers = conflicts
        .values()
        .flat_map(|conflict| {
            conflict
                .candidates
                .iter()
                .filter(|candidate| candidate.event_id != conflict.winner_event_id)
                .map(|candidate| candidate.event_id.clone())
        })
        .collect::<HashSet<_>>();

    let mut state = KbdStateV2::default();
    let mut authority_frontier = CausalFrontier::empty();
    let mut authority_heads = BTreeMap::new();
    let mut run_ids = HashSet::new();
    for event in ordered {
        let replica_id = event_replica_id(event);
        let lamport = event_lamport(event);
        authority_frontier.advance(replica_id.clone(), lamport);
        authority_heads.insert(
            replica_id,
            ReplicaHead {
                event_id: event.event_id.clone(),
                integrity_hash: event.integrity_hash.clone(),
                lamport,
            },
        );
        run_ids.insert(event.run_id.clone());
        if losers.contains(&event.event_id)
            || matches!(event.kind, EventKind::ConflictResolved { .. })
        {
            continue;
        }
        if let Err(error) = state.apply_folded(event) {
            if matches!(error, RuntimeError::DuplicateCommand(_)) {
                continue;
            }
            let conflict = fold_error_conflict(event, &error.to_string())?;
            conflicts.insert(conflict.id.clone(), conflict);
        }
    }
    state.frontier = authority_frontier;
    state.replica_heads = authority_heads;
    state.revision = state.frontier.derived_revision();
    state.conflicts = conflicts;
    if run_ids.len() > 1 {
        let frontier = serde_jcs::to_vec(&state.frontier)
            .map_err(|error| RuntimeError::InvalidState(error.to_string()))?;
        state.run_id = format!("merge:{:x}", Sha256::digest(frontier));
    }
    Ok(state)
}

fn detect_conflicts(events: &[Event]) -> Result<BTreeMap<String, ConflictRecord>> {
    let mut slots = BTreeMap::<String, Vec<&Event>>::new();
    for event in events {
        if let Some((slot, _)) = event_slot(event) {
            slots.entry(slot).or_default().push(event);
        }
    }
    let mut conflicts = BTreeMap::new();
    for (slot, candidates) in slots {
        let maximal = candidates
            .iter()
            .copied()
            .filter(|candidate| {
                !candidates.iter().copied().any(|other| {
                    candidate.event_id != other.event_id && event_observes(other, candidate)
                })
            })
            .collect::<Vec<_>>();
        if maximal.len() < 2 {
            continue;
        }
        let mut conflict_candidates = maximal
            .iter()
            .map(|event| conflict_candidate(event))
            .collect::<Result<Vec<_>>>()?;
        conflict_candidates.sort_by(|left, right| {
            (left.lamport, &left.event_id).cmp(&(right.lamport, &right.event_id))
        });
        let winner_event_id = conflict_candidates
            .last()
            .expect("two conflict candidates")
            .event_id
            .clone();
        let kind = event_slot(maximal[0])
            .map(|(_, kind)| kind)
            .unwrap_or(ConflictKind::Fold);
        let id = conflict_id(
            &slot,
            conflict_candidates
                .iter()
                .map(|candidate| candidate.event_id.as_str()),
        );
        conflicts.insert(
            id.clone(),
            ConflictRecord {
                id,
                slot,
                kind,
                candidates: conflict_candidates,
                winner_event_id,
                resolved_by_event_id: None,
                resolution_reason: None,
            },
        );
    }
    Ok(conflicts)
}

fn detect_claim_conflicts(events: &[Event]) -> Result<BTreeMap<String, ConflictRecord>> {
    let mut scopes = BTreeMap::<String, Vec<&Event>>::new();
    for event in events {
        if let EventKind::ClaimAcquired { scope, .. } = &event.kind {
            scopes.entry(scope.clone()).or_default().push(event);
        }
    }
    let mut conflicts = BTreeMap::new();
    for (scope, candidates) in scopes {
        let maximal = candidates
            .iter()
            .copied()
            .filter(|candidate| {
                !candidates.iter().copied().any(|other| {
                    candidate.event_id != other.event_id && event_observes(other, candidate)
                })
            })
            .collect::<Vec<_>>();
        let incompatible = maximal.len() > 1
            && maximal.iter().any(|event| {
                matches!(
                    event.kind,
                    EventKind::ClaimAcquired {
                        mode: crate::ClaimMode::Exclusive,
                        ..
                    }
                )
            });
        if !incompatible {
            continue;
        }
        let mut claim_candidates = maximal
            .iter()
            .map(|event| conflict_candidate(event))
            .collect::<Result<Vec<_>>>()?;
        claim_candidates.sort_by(|left, right| {
            let left_holder = left
                .value
                .get("payload")
                .and_then(|payload| payload.get("holderId"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&left.actor_id);
            let right_holder = right
                .value
                .get("payload")
                .and_then(|payload| payload.get("holderId"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&right.actor_id);
            (left.lamport, left_holder, &left.event_id).cmp(&(
                right.lamport,
                right_holder,
                &right.event_id,
            ))
        });
        let winner_event_id = claim_candidates
            .last()
            .expect("incompatible claims have candidates")
            .event_id
            .clone();
        let slot = format!("claim:{scope}");
        let id = conflict_id(
            &slot,
            claim_candidates
                .iter()
                .map(|candidate| candidate.event_id.as_str()),
        );
        conflicts.insert(
            id.clone(),
            ConflictRecord {
                id,
                slot,
                kind: ConflictKind::Claim,
                candidates: claim_candidates,
                winner_event_id,
                resolved_by_event_id: None,
                resolution_reason: None,
            },
        );
    }
    Ok(conflicts)
}

fn event_slot(event: &Event) -> Option<(String, ConflictKind)> {
    match &event.kind {
        EventKind::RunInitialized { .. }
        | EventKind::LifecycleTransition { .. }
        | EventKind::PauseCheckpointed { .. } => {
            Some(("singleton:lifecycle".into(), ConflictKind::Lifecycle))
        }
        EventKind::ActivePathChanged { .. } => {
            Some(("singleton:active_path".into(), ConflictKind::ActivePath))
        }
        EventKind::PhaseDefined { phase } => {
            Some((format!("phase:{}", phase.id), ConflictKind::Phase))
        }
        EventKind::PhaseTransitioned { phase_id, .. } => {
            Some((format!("phase:{phase_id}"), ConflictKind::Phase))
        }
        EventKind::CompletionUpdated { dimension, .. } => Some((
            format!("completion:{dimension:?}"),
            ConflictKind::Completion,
        )),
        EventKind::DecisionRecorded { decision } => {
            Some((format!("decision:{}", decision.id), ConflictKind::Decision))
        }
        EventKind::BlockerRecorded { blocker } => {
            Some((format!("blocker:{}", blocker.id), ConflictKind::Blocker))
        }
        EventKind::BlockerCleared { blocker_id, .. } => {
            Some((format!("blocker:{blocker_id}"), ConflictKind::Blocker))
        }
        _ => None,
    }
}

fn event_observes(observer: &Event, observed: &Event) -> bool {
    if observed.replica_id.is_empty() {
        observer.replica_id.is_empty() && observer.revision > observed.revision
    } else {
        observer.frontier.contains_event(observed)
    }
}

fn causal_order(events: &[Event]) -> Result<Vec<&Event>> {
    let mut remaining = events.iter().collect::<Vec<_>>();
    let mut ordered = Vec::with_capacity(events.len());
    let mut frontier = CausalFrontier::empty();
    while !remaining.is_empty() {
        remaining.sort_by(|left, right| event_order_key(left).cmp(&event_order_key(right)));
        let Some(index) = remaining.iter().position(|event| {
            if event.replica_id.is_empty() {
                event.revision == frontier.next_lamport("legacy")
            } else {
                frontier.dominates(&event.frontier)
                    && event.lamport == frontier.next_lamport(&event.replica_id)
            }
        }) else {
            return Err(RuntimeError::InvalidState(
                "project.loro contains an event whose causal frontier cannot be satisfied".into(),
            ));
        };
        let event = remaining.remove(index);
        frontier.advance(event_replica_id(event), event_lamport(event));
        ordered.push(event);
    }
    Ok(ordered)
}

fn event_order_key(event: &Event) -> (u64, u64, &str, &str) {
    (
        if event.frontier.is_empty() {
            event.revision
        } else {
            event.frontier.derived_revision().saturating_add(1)
        },
        event_lamport(event),
        event_replica_id_ref(event),
        &event.event_id,
    )
}

fn event_replica_id(event: &Event) -> String {
    event_replica_id_ref(event).to_owned()
}

fn event_replica_id_ref(event: &Event) -> &str {
    if event.replica_id.is_empty() {
        "legacy"
    } else {
        &event.replica_id
    }
}

fn event_lamport(event: &Event) -> u64 {
    if event.lamport == 0 {
        event.revision
    } else {
        event.lamport
    }
}

fn conflict_candidate(event: &Event) -> Result<ConflictCandidate> {
    Ok(ConflictCandidate {
        event_id: event.event_id.clone(),
        replica_id: event_replica_id(event),
        lamport: event_lamport(event),
        actor_id: if event.actor_id.is_empty() {
            event.actor.id.clone()
        } else {
            event.actor_id.clone()
        },
        value: serde_json::to_value(&event.kind)?,
    })
}

fn fold_error_conflict(event: &Event, error: &str) -> Result<ConflictRecord> {
    let candidate = ConflictCandidate {
        value: serde_json::json!({
            "event": event.kind,
            "foldError": error,
        }),
        ..conflict_candidate(event)?
    };
    let slot = format!("fold:{}", event.event_id);
    let id = conflict_id(&slot, [event.event_id.as_str()]);
    Ok(ConflictRecord {
        id: id.clone(),
        slot,
        kind: ConflictKind::Fold,
        winner_event_id: event.event_id.clone(),
        candidates: vec![candidate],
        resolved_by_event_id: None,
        resolution_reason: None,
    })
}

fn conflict_id<'a>(slot: &str, event_ids: impl IntoIterator<Item = &'a str>) -> String {
    let mut input = slot.as_bytes().to_vec();
    for event_id in event_ids {
        input.push(0);
        input.extend_from_slice(event_id.as_bytes());
    }
    format!("conflict:{:x}", Sha256::digest(input))
}

fn sort_events(events: &mut [Event]) {
    events.sort_by(|left, right| {
        let left_order = if left.frontier.is_empty() {
            left.revision
        } else {
            left.frontier.derived_revision().saturating_add(1)
        };
        let right_order = if right.frontier.is_empty() {
            right.revision
        } else {
            right.frontier.derived_revision().saturating_add(1)
        };
        (left_order, left.lamport, &left.replica_id, &left.event_id).cmp(&(
            right_order,
            right.lamport,
            &right.replica_id,
            &right.event_id,
        ))
    });
}

fn loro_error(error: loro::LoroError) -> RuntimeError {
    RuntimeError::InvalidState(format!("project.loro: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, ActorKind, ClaimMode, DeviceSigner, EventKind, MigrationProvenance, Phase, Runtime,
        WorkStatus, EVENT_SCHEMA_VERSION,
    };
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn grow_only_document_is_idempotent_and_persistent() {
        let fixture = tempdir().unwrap();
        let runtime_root = fixture.path().join("runtime");
        let runtime = Runtime::open(&runtime_root);
        let state = runtime
            .initialize(
                "project-a",
                "run-a",
                Actor {
                    kind: ActorKind::Operator,
                    id: "operator-a".into(),
                    device: "device-a".into(),
                    harness: "test".into(),
                    session: "session-a".into(),
                },
            )
            .unwrap();
        assert_eq!(state.revision, 1);
        let events = runtime.events().unwrap();
        let document = ProjectDocument::open(fixture.path().join("document"), "project-a");
        assert_eq!(document.ingest_events(&events).unwrap(), 1);
        assert_eq!(document.ingest_events(&events).unwrap(), 0);
        assert_eq!(document.events().unwrap(), events);
        let status = document.status().unwrap();
        assert_eq!(status.event_count, 1);
        assert!(status.bytes > 0);
        assert!(status.snapshot_sha256.is_some());
    }

    #[test]
    fn authoritative_updates_validate_merge_fsync_and_replay_idempotently() {
        let fixture = tempdir().unwrap();
        let runtime = Runtime::open(fixture.path().join("runtime"));
        runtime
            .initialize(
                "project-a",
                "run-a",
                Actor {
                    kind: ActorKind::Operator,
                    id: "operator-a".into(),
                    device: "device-a".into(),
                    harness: "test".into(),
                    session: "session-a".into(),
                },
            )
            .unwrap();
        let source = ProjectDocument::open(fixture.path().join("source"), "project-a");
        source.ingest_events(&runtime.events().unwrap()).unwrap();
        let updates = source.export_updates().unwrap();

        let target = ProjectDocument::open(fixture.path().join("target"), "project-a");
        let (inserted, state) = target.import_updates(&updates).unwrap();
        assert_eq!(inserted, 1);
        assert_eq!(state.project_id, "project-a");
        assert_eq!(state.revision, 1);
        assert_eq!(target.import_updates(&updates).unwrap().0, 0);

        let before = fs::read(target.path()).unwrap();
        let mut corrupt = updates;
        corrupt.truncate(corrupt.len() / 2);
        assert!(target.import_updates(&corrupt).is_err());
        assert_eq!(fs::read(target.path()).unwrap(), before);
    }

    fn signed_branch_event(
        project_id: &str,
        replica_id: &str,
        event_id: &str,
        frontier: CausalFrontier,
        kind: EventKind,
        actor: Actor,
    ) -> Event {
        let signer = DeviceSigner::generate();
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: project_id.into(),
            replica_id: replica_id.into(),
            run_id: "run-a".into(),
            event_id: event_id.into(),
            command_id: Some(format!("command-{event_id}")),
            revision: frontier.derived_revision().saturating_add(1),
            expected_revision: frontier.derived_revision(),
            lamport: frontier.next_lamport(replica_id),
            frontier,
            causal_parent: None,
            actor_id: actor.id.clone(),
            actor,
            timestamp: Utc::now(),
            kind,
            previous_hash: None,
            migration_provenance: None::<MigrationProvenance>,
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        event.seal(&signer).unwrap();
        event
    }

    fn phase(id: &str, title: &str) -> Phase {
        Phase {
            id: id.into(),
            slug: id.into(),
            title: title.into(),
            parent_phase_id: None,
            status: WorkStatus::Pending,
            stages: BTreeMap::new(),
            changes: BTreeMap::new(),
            legacy_read_only: false,
        }
    }

    #[test]
    fn divergent_phases_union_conflicts_stay_visible_and_resolution_is_authoritative() {
        let fixture = tempdir().unwrap();
        let runtime = Runtime::open(fixture.path().join("source"));
        runtime
            .initialize("project-a", "run-a", Actor::operator("operator-a", "test"))
            .unwrap();
        let genesis = runtime.events().unwrap().remove(0);
        let base = crate::replay_events(std::slice::from_ref(&genesis)).unwrap();
        let branch_actor = |id: &str| Actor {
            kind: ActorKind::Harness,
            id: id.into(),
            device: format!("device-{id}"),
            harness: "test".into(),
            session: format!("session-{id}"),
        };
        let phase_a = signed_branch_event(
            "project-a",
            "replica-a",
            "event-a",
            base.frontier.clone(),
            EventKind::PhaseDefined {
                phase: phase("phase-1", "candidate A"),
            },
            branch_actor("actor-a"),
        );
        let phase_b = signed_branch_event(
            "project-a",
            "replica-b",
            "event-b",
            base.frontier.clone(),
            EventKind::PhaseDefined {
                phase: phase("phase-1", "candidate B"),
            },
            branch_actor("actor-b"),
        );
        let distinct = signed_branch_event(
            "project-a",
            "replica-c",
            "event-c",
            base.frontier,
            EventKind::PhaseDefined {
                phase: phase("phase-2", "independent"),
            },
            branch_actor("actor-c"),
        );
        let document = ProjectDocument::open(fixture.path().join("document"), "project-a");
        document
            .ingest_events(&[genesis.clone(), phase_a.clone(), phase_b.clone(), distinct])
            .unwrap();

        let folded = document.fold().unwrap();
        assert_eq!(folded.phases.len(), 2);
        assert_eq!(folded.phases["phase-1"].title, "candidate B");
        assert_eq!(folded.phases["phase-2"].title, "independent");
        assert_eq!(folded.conflicts.len(), 1);
        let conflict = folded.conflicts.values().next().unwrap().clone();
        assert_eq!(conflict.candidates.len(), 2);
        assert_eq!(conflict.winner_event_id, "event-b");
        assert!(conflict.resolved_by_event_id.is_none());

        let resolution = signed_branch_event(
            "project-a",
            "operator-replica",
            "resolution-1",
            folded.frontier.clone(),
            EventKind::ConflictResolved {
                conflict_id: conflict.id.clone(),
                winner_event_id: "event-a".into(),
                reason: "operator selected the canonical phase definition".into(),
            },
            Actor::operator("operator-a", "test"),
        );
        document.ingest_events(&[resolution]).unwrap();
        let resolved = document.fold().unwrap();
        assert_eq!(resolved.phases["phase-1"].title, "candidate A");
        let conflict = &resolved.conflicts[&conflict.id];
        assert_eq!(conflict.winner_event_id, "event-a");
        assert_eq!(
            conflict.resolved_by_event_id.as_deref(),
            Some("resolution-1")
        );
    }

    #[test]
    fn concurrent_exclusive_claims_select_lamport_then_holder_and_keep_loser_visible() {
        let fixture = tempdir().unwrap();
        let document = ProjectDocument::open(fixture.path(), "project-a");
        let operator = Actor {
            kind: ActorKind::Operator,
            id: "operator".into(),
            device: "device".into(),
            harness: "test".into(),
            session: "session".into(),
        };
        let genesis = signed_branch_event(
            "project-a",
            "origin",
            "genesis",
            CausalFrontier::empty(),
            EventKind::RunInitialized {
                initial_state: crate::LifecycleState::Ready,
                exact_next_work: None,
                plan_revision: 1,
            },
            operator,
        );
        let mut frontier = CausalFrontier::empty();
        frontier.advance("origin", 1);
        let expires_at = Utc::now() + chrono::Duration::minutes(10);
        let claim_a = signed_branch_event(
            "project-a",
            "replica-a",
            "claim-a",
            frontier.clone(),
            EventKind::ClaimAcquired {
                claim_id: "claim-a".into(),
                scope: "phase:recovery".into(),
                holder_id: "holder-a".into(),
                mode: ClaimMode::Exclusive,
                expires_at,
                monotonic_token: 1,
            },
            Actor {
                kind: ActorKind::Harness,
                id: "holder-a".into(),
                device: "device-a".into(),
                harness: "test".into(),
                session: "a".into(),
            },
        );
        let claim_b = signed_branch_event(
            "project-a",
            "replica-b",
            "claim-b",
            frontier,
            EventKind::ClaimAcquired {
                claim_id: "claim-b".into(),
                scope: "phase:recovery".into(),
                holder_id: "holder-b".into(),
                mode: ClaimMode::Exclusive,
                expires_at,
                monotonic_token: 1,
            },
            Actor {
                kind: ActorKind::Harness,
                id: "holder-b".into(),
                device: "device-b".into(),
                harness: "test".into(),
                session: "b".into(),
            },
        );
        document
            .ingest_events(&[genesis, claim_a, claim_b])
            .unwrap();
        let state = document.fold().unwrap();
        let conflict = state
            .conflicts
            .values()
            .find(|conflict| conflict.kind == ConflictKind::Claim)
            .unwrap();
        assert_eq!(conflict.winner_event_id, "claim-b");
        assert_eq!(conflict.candidates.len(), 2);
        assert!(state.claims.contains_key("claim-b"));
        assert!(!state.claims.contains_key("claim-a"));
    }
}
