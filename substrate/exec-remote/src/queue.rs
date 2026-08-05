use std::{
    fs::{self, File, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use fs2::FileExt;
use prometheus_exec_contracts::{hash_serializable, Digest, ExecutionReceipt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    verify_dispatch, EnrollmentSnapshot, PeerDispatchState, RemoteError, Result,
    SignedRemoteDispatch, REMOTE_SCHEMA_VERSION,
};

const MAX_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRecord {
    pub dispatch: SignedRemoteDispatch,
    pub dispatch_hash: Digest,
    pub state: PeerDispatchState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ExecutionReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sequence: u64,
    pub event_hash: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptDispatchResult {
    pub record: DispatchRecord,
    pub replayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DispatchEvent {
    schema_version: String,
    dispatch_id: Uuid,
    sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    previous_event_hash: Option<Digest>,
    occurred_at: DateTime<Utc>,
    #[serde(flatten)]
    data: DispatchEventData,
    event_hash: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
enum DispatchEventData {
    Accepted {
        dispatch: Box<SignedRemoteDispatch>,
    },
    Transition {
        state: PeerDispatchState,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        run_id: Option<Uuid>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt: Option<Box<ExecutionReceipt>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        failure: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct DispatchQueue {
    root: PathBuf,
}

impl DispatchQueue {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let queue = Self { root: root.into() };
        create_private_dir(&queue.segment_root())?;
        let _lock = queue.operation_lock()?;
        queue.records_unlocked()?;
        Ok(queue)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Inspect an existing queue without creating directories, lock files, or
    /// repair state. Intended for doctor/certification surfaces.
    pub fn inspect_read_only(root: impl Into<PathBuf>) -> Result<Vec<DispatchRecord>> {
        let queue = Self { root: root.into() };
        let root_metadata =
            fs::symlink_metadata(&queue.root).map_err(|source| io_error(&queue.root, source))?;
        let segment_metadata = fs::symlink_metadata(queue.segment_root())
            .map_err(|source| io_error(queue.segment_root(), source))?;
        if root_metadata.file_type().is_symlink()
            || !root_metadata.is_dir()
            || segment_metadata.file_type().is_symlink()
            || !segment_metadata.is_dir()
        {
            return Err(RemoteError::CorruptSegment(
                "remote queue root or segment directory is unsafe".into(),
            ));
        }
        let lock_path = queue.root.join("queue.lock");
        let lock = OpenOptions::new()
            .read(true)
            .open(&lock_path)
            .map_err(|source| io_error(&lock_path, source))?;
        FileExt::lock_shared(&lock).map_err(|source| io_error(&lock_path, source))?;
        queue.records_unlocked()
    }

    pub fn accept(
        &self,
        dispatch: SignedRemoteDispatch,
        enrollment: &EnrollmentSnapshot,
        now: DateTime<Utc>,
    ) -> Result<AcceptDispatchResult> {
        verify_dispatch(&dispatch, enrollment)?;
        if now > dispatch.expires_at()? {
            return Err(RemoteError::Expired);
        }
        let _lock = self.operation_lock()?;
        let supplied = dispatch.dispatch_hash()?;
        if let Some(existing) = self.get_unlocked(dispatch.dispatch_id)? {
            if existing.dispatch_hash == supplied {
                return Ok(AcceptDispatchResult {
                    record: existing,
                    replayed: true,
                });
            }
            return Err(RemoteError::DispatchHashConflict {
                dispatch_id: dispatch.dispatch_id,
                existing: existing.dispatch_hash,
                supplied,
            });
        }
        for existing in self.records_unlocked()? {
            if existing.dispatch.request.request_id == dispatch.request.request_id {
                return Err(RemoteError::RequestReplay {
                    request_id: dispatch.request.request_id,
                    existing_dispatch_id: existing.dispatch.dispatch_id,
                });
            }
        }
        let event = self.new_event(
            dispatch.dispatch_id,
            1,
            None,
            now,
            DispatchEventData::Accepted {
                dispatch: Box::new(dispatch),
            },
        )?;
        self.append_event_unlocked(&event)?;
        Ok(AcceptDispatchResult {
            record: record_from_events(&[event])?,
            replayed: false,
        })
    }

    pub fn get(&self, dispatch_id: Uuid) -> Result<Option<DispatchRecord>> {
        let _lock = self.shared_operation_lock()?;
        self.get_unlocked(dispatch_id)
    }

    pub fn records(&self) -> Result<Vec<DispatchRecord>> {
        let _lock = self.shared_operation_lock()?;
        self.records_unlocked()
    }

    pub fn transition(
        &self,
        dispatch_id: Uuid,
        state: PeerDispatchState,
        run_id: Option<Uuid>,
        receipt: Option<ExecutionReceipt>,
        failure: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<DispatchRecord> {
        let _lock = self.operation_lock()?;
        let current = self
            .get_unlocked(dispatch_id)?
            .ok_or(RemoteError::DispatchNotFound(dispatch_id))?;
        validate_transition(
            &current,
            state,
            run_id,
            receipt.as_ref(),
            failure.as_deref(),
        )?;
        if current.state == state
            && current.run_id == run_id
            && current.receipt == receipt
            && current.failure == failure
        {
            return Ok(current);
        }
        let event = self.new_event(
            dispatch_id,
            current.sequence + 1,
            Some(current.event_hash),
            now,
            DispatchEventData::Transition {
                state,
                run_id,
                receipt: receipt.map(Box::new),
                failure,
            },
        )?;
        self.append_event_unlocked(&event)?;
        self.get_unlocked(dispatch_id)?
            .ok_or(RemoteError::DispatchNotFound(dispatch_id))
    }

    pub fn reconcile_expired(&self, now: DateTime<Utc>) -> Result<Vec<DispatchRecord>> {
        let mut expired = Vec::new();
        for record in self.records()? {
            if !record.state.is_terminal() && now > record.dispatch.expires_at()? {
                expired.push(self.transition(
                    record.dispatch.dispatch_id,
                    PeerDispatchState::Expired,
                    record.run_id,
                    None,
                    Some("remote dispatch validity window expired".into()),
                    now,
                )?);
            }
        }
        Ok(expired)
    }

    fn records_unlocked(&self) -> Result<Vec<DispatchRecord>> {
        let mut records = Vec::new();
        for entry in fs::read_dir(self.segment_root())
            .map_err(|source| io_error(self.segment_root(), source))?
        {
            let entry = entry.map_err(|source| io_error(self.segment_root(), source))?;
            let file_type = entry
                .file_type()
                .map_err(|source| io_error(entry.path(), source))?;
            if file_type.is_symlink() || !file_type.is_dir() {
                return Err(RemoteError::CorruptSegment(format!(
                    "unexpected queue entry {}",
                    entry.path().display()
                )));
            }
            let dispatch_id = entry
                .file_name()
                .to_str()
                .ok_or_else(|| RemoteError::CorruptSegment("non-UTF-8 dispatch directory".into()))?
                .parse::<Uuid>()
                .map_err(|error| RemoteError::CorruptSegment(error.to_string()))?;
            records.push(record_from_events(
                &self.load_events_unlocked(dispatch_id)?,
            )?);
        }
        records.sort_by_key(|record| record.dispatch.dispatch_id);
        Ok(records)
    }

    fn get_unlocked(&self, dispatch_id: Uuid) -> Result<Option<DispatchRecord>> {
        let directory = self.dispatch_dir(dispatch_id);
        if !directory.exists() {
            return Ok(None);
        }
        Ok(Some(record_from_events(
            &self.load_events_unlocked(dispatch_id)?,
        )?))
    }

    fn load_events_unlocked(&self, dispatch_id: Uuid) -> Result<Vec<DispatchEvent>> {
        let directory = self.dispatch_dir(dispatch_id);
        let mut paths = fs::read_dir(&directory)
            .map_err(|source| io_error(&directory, source))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|source| io_error(&directory, source))?;
        paths.sort();
        if paths.is_empty() {
            return Err(RemoteError::CorruptSegment(format!(
                "dispatch {dispatch_id} has no events"
            )));
        }
        let mut events = Vec::with_capacity(paths.len());
        for (index, path) in paths.into_iter().enumerate() {
            let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.len() > MAX_SEGMENT_BYTES
            {
                return Err(RemoteError::CorruptSegment(format!(
                    "unsafe event segment {}",
                    path.display()
                )));
            }
            let event: DispatchEvent = serde_json::from_slice(
                &fs::read(&path).map_err(|source| io_error(&path, source))?,
            )?;
            let expected_sequence = index as u64 + 1;
            if event.dispatch_id != dispatch_id || event.sequence != expected_sequence {
                return Err(RemoteError::CorruptSegment(format!(
                    "event sequence mismatch at {}",
                    path.display()
                )));
            }
            let expected_hash = compute_event_hash(
                event.dispatch_id,
                event.sequence,
                event.previous_event_hash.as_ref(),
                event.occurred_at,
                &event.data,
            )?;
            if event.event_hash != expected_hash {
                return Err(RemoteError::CorruptSegment(format!(
                    "event hash mismatch at {}",
                    path.display()
                )));
            }
            let expected_previous = events.last().map(|prior: &DispatchEvent| &prior.event_hash);
            if event.previous_event_hash.as_ref() != expected_previous {
                return Err(RemoteError::CorruptSegment(format!(
                    "event chain mismatch at {}",
                    path.display()
                )));
            }
            let expected_name = event_filename(event.sequence, &event.event_hash);
            if path.file_name().and_then(|name| name.to_str()) != Some(expected_name.as_str()) {
                return Err(RemoteError::CorruptSegment(format!(
                    "event filename mismatch at {}",
                    path.display()
                )));
            }
            events.push(event);
        }
        Ok(events)
    }

    fn new_event(
        &self,
        dispatch_id: Uuid,
        sequence: u64,
        previous_event_hash: Option<Digest>,
        occurred_at: DateTime<Utc>,
        data: DispatchEventData,
    ) -> Result<DispatchEvent> {
        let event_hash = compute_event_hash(
            dispatch_id,
            sequence,
            previous_event_hash.as_ref(),
            occurred_at,
            &data,
        )?;
        Ok(DispatchEvent {
            schema_version: REMOTE_SCHEMA_VERSION.into(),
            dispatch_id,
            sequence,
            previous_event_hash,
            occurred_at,
            data,
            event_hash,
        })
    }

    fn append_event_unlocked(&self, event: &DispatchEvent) -> Result<()> {
        let directory = self.dispatch_dir(event.dispatch_id);
        create_private_dir(&directory)?;
        let destination = directory.join(event_filename(event.sequence, &event.event_hash));
        let mut bytes = serde_json::to_vec(event)?;
        bytes.push(b'\n');
        let mut temporary = tempfile::NamedTempFile::new_in(&directory)
            .map_err(|source| io_error(&directory, source))?;
        set_private_file(temporary.as_file())?;
        temporary
            .write_all(&bytes)
            .map_err(|source| io_error(temporary.path(), source))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| io_error(temporary.path(), source))?;
        temporary
            .persist_noclobber(&destination)
            .map_err(|error| io_error(&destination, error.error))?;
        sync_directory(&directory)?;
        Ok(())
    }

    fn operation_lock(&self) -> Result<File> {
        let lock = open_lock(&self.root.join("queue.lock"))?;
        FileExt::lock_exclusive(&lock)
            .map_err(|source| io_error(self.root.join("queue.lock"), source))?;
        Ok(lock)
    }

    fn shared_operation_lock(&self) -> Result<File> {
        let lock = open_lock(&self.root.join("queue.lock"))?;
        FileExt::lock_shared(&lock)
            .map_err(|source| io_error(self.root.join("queue.lock"), source))?;
        Ok(lock)
    }

    fn segment_root(&self) -> PathBuf {
        self.root.join("segments")
    }

    fn dispatch_dir(&self, dispatch_id: Uuid) -> PathBuf {
        self.segment_root().join(dispatch_id.to_string())
    }
}

fn record_from_events(events: &[DispatchEvent]) -> Result<DispatchRecord> {
    let first = events
        .first()
        .ok_or_else(|| RemoteError::CorruptSegment("empty dispatch event chain".into()))?;
    let DispatchEventData::Accepted { dispatch } = &first.data else {
        return Err(RemoteError::CorruptSegment(
            "first dispatch event is not accepted".into(),
        ));
    };
    let mut record = DispatchRecord {
        dispatch: (**dispatch).clone(),
        dispatch_hash: dispatch.dispatch_hash()?,
        state: PeerDispatchState::Queued,
        run_id: None,
        receipt: None,
        failure: None,
        accepted_at: first.occurred_at,
        updated_at: first.occurred_at,
        sequence: first.sequence,
        event_hash: first.event_hash.clone(),
    };
    for event in &events[1..] {
        let DispatchEventData::Transition {
            state,
            run_id,
            receipt,
            failure,
        } = &event.data
        else {
            return Err(RemoteError::CorruptSegment(
                "accepted event appears after sequence one".into(),
            ));
        };
        validate_transition(
            &record,
            *state,
            *run_id,
            receipt.as_deref(),
            failure.as_deref(),
        )?;
        record.state = *state;
        record.run_id = *run_id;
        record.receipt = receipt.as_deref().cloned();
        record.failure = failure.clone();
        record.updated_at = event.occurred_at;
        record.sequence = event.sequence;
        record.event_hash = event.event_hash.clone();
    }
    Ok(record)
}

fn validate_transition(
    current: &DispatchRecord,
    next: PeerDispatchState,
    run_id: Option<Uuid>,
    receipt: Option<&ExecutionReceipt>,
    failure: Option<&str>,
) -> Result<()> {
    if current.state.is_terminal() {
        if current.state == next
            && current.run_id == run_id
            && current.receipt.as_ref() == receipt
            && current.failure.as_deref() == failure
        {
            return Ok(());
        }
        return Err(RemoteError::InvalidTransition(format!(
            "terminal state {:?} cannot transition to {:?}",
            current.state, next
        )));
    }
    let allowed = matches!(
        (current.state, next),
        (PeerDispatchState::Queued, PeerDispatchState::Received)
            | (PeerDispatchState::Queued, PeerDispatchState::Unavailable)
            | (PeerDispatchState::Queued, PeerDispatchState::Expired)
            | (
                PeerDispatchState::Queued,
                PeerDispatchState::PendingEvidence
            )
            | (PeerDispatchState::Unavailable, PeerDispatchState::Queued)
            | (PeerDispatchState::Unavailable, PeerDispatchState::Received)
            | (PeerDispatchState::Unavailable, PeerDispatchState::Expired)
            | (PeerDispatchState::Received, PeerDispatchState::Running)
            | (PeerDispatchState::Received, PeerDispatchState::Rejected)
            | (PeerDispatchState::Received, PeerDispatchState::Expired)
            | (PeerDispatchState::Running, PeerDispatchState::Applied)
            | (PeerDispatchState::Running, PeerDispatchState::Rejected)
            | (
                PeerDispatchState::Running,
                PeerDispatchState::PendingEvidence
            )
    );
    if !allowed {
        return Err(RemoteError::InvalidTransition(format!(
            "{:?} cannot transition to {:?}",
            current.state, next
        )));
    }
    match next {
        PeerDispatchState::Applied => {
            let receipt = receipt.ok_or_else(|| {
                RemoteError::InvalidTransition("applied state requires a receipt".into())
            })?;
            if run_id != Some(receipt.run_id)
                || receipt.request_hash != current.dispatch.request_hash
            {
                return Err(RemoteError::InvalidTransition(
                    "applied receipt does not match run or request".into(),
                ));
            }
            if failure.is_some() {
                return Err(RemoteError::InvalidTransition(
                    "applied state cannot carry a failure".into(),
                ));
            }
        }
        PeerDispatchState::Rejected
        | PeerDispatchState::Expired
        | PeerDispatchState::Unavailable
        | PeerDispatchState::PendingEvidence => {
            if failure.is_none_or(str::is_empty) {
                return Err(RemoteError::InvalidTransition(format!(
                    "{next:?} state requires a failure or disposition"
                )));
            }
            if receipt.is_some() {
                return Err(RemoteError::InvalidTransition(format!(
                    "{next:?} state cannot carry an applied receipt"
                )));
            }
        }
        _ => {
            if receipt.is_some() || failure.is_some() {
                return Err(RemoteError::InvalidTransition(format!(
                    "{next:?} state cannot carry terminal evidence"
                )));
            }
        }
    }
    Ok(())
}

fn compute_event_hash(
    dispatch_id: Uuid,
    sequence: u64,
    previous_event_hash: Option<&Digest>,
    occurred_at: DateTime<Utc>,
    data: &DispatchEventData,
) -> Result<Digest> {
    Ok(hash_serializable(&(
        REMOTE_SCHEMA_VERSION,
        dispatch_id,
        sequence,
        previous_event_hash,
        occurred_at,
        data,
    ))?)
}

fn event_filename(sequence: u64, hash: &Digest) -> String {
    format!(
        "{sequence:020}-{}.json",
        hash.as_str().trim_start_matches("sha256:")
    )
}

fn open_lock(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        create_private_dir(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    set_private_file(&file)?;
    Ok(file)
}

fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_private_file(file: &File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|source| io_error("<file>", source))
}

#[cfg(not(unix))]
fn set_private_file(_file: &File) -> Result<()> {
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(path, source))
}

fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> RemoteError {
    RemoteError::Io {
        path: path.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use tempfile::tempdir;

    use super::DispatchQueue;
    use crate::{PeerDispatchState, RemoteError};

    #[test]
    fn accept_replay_conflict_restart_and_expiry_are_durable() {
        let directory = tempdir().expect("temporary queue");
        let (dispatch, enrollment, signing_key) = crate::tests::fixture();
        let now = dispatch.issued_at;
        let queue = DispatchQueue::open(directory.path()).expect("queue opens");
        let accepted = queue
            .accept(dispatch.clone(), &enrollment, now)
            .expect("dispatch accepted");
        assert!(!accepted.replayed);
        assert!(
            queue
                .accept(dispatch.clone(), &enrollment, now)
                .expect("same dispatch replayed")
                .replayed
        );

        let mut conflict = dispatch.clone();
        conflict.validity_window_secs += 1;
        crate::sign_dispatch_ed25519(&mut conflict, &signing_key).expect("conflict signed");
        assert!(matches!(
            queue.accept(conflict, &enrollment, now),
            Err(RemoteError::DispatchHashConflict { .. })
        ));

        drop(queue);
        let reopened = DispatchQueue::open(directory.path()).expect("queue reopens");
        assert_eq!(
            reopened
                .get(dispatch.dispatch_id)
                .expect("record loads")
                .expect("record exists")
                .dispatch_hash,
            accepted.record.dispatch_hash
        );
        let expired = reopened
            .reconcile_expired(now + Duration::seconds(61))
            .expect("expiry reconciles");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].state, PeerDispatchState::Expired);
    }

    #[test]
    fn tampered_segment_prevents_restart_from_false_green() {
        let directory = tempdir().expect("temporary queue");
        let (dispatch, enrollment, _) = crate::tests::fixture();
        let queue = DispatchQueue::open(directory.path()).expect("queue opens");
        queue
            .accept(dispatch, &enrollment, chrono::Utc::now())
            .expect("dispatch accepted");
        drop(queue);
        let segment = walk_first_file(&directory.path().join("segments"));
        std::fs::write(&segment, b"{}\n").expect("segment tampered");
        assert!(DispatchQueue::open(directory.path()).is_err());
    }

    #[test]
    fn read_only_inspection_never_constructs_missing_queue_state() {
        let directory = tempdir().expect("temporary root");
        let missing = directory.path().join("missing");
        assert!(DispatchQueue::inspect_read_only(&missing).is_err());
        assert!(!missing.exists());

        let queue_root = directory.path().join("queue");
        let (dispatch, enrollment, _) = crate::tests::fixture();
        let queue = DispatchQueue::open(&queue_root).expect("queue opens");
        queue
            .accept(dispatch, &enrollment, chrono::Utc::now())
            .expect("dispatch accepted");
        drop(queue);
        let before = walk_files(&queue_root);
        assert_eq!(
            DispatchQueue::inspect_read_only(&queue_root).unwrap().len(),
            1
        );
        assert_eq!(walk_files(&queue_root), before);
    }

    fn walk_first_file(root: &std::path::Path) -> std::path::PathBuf {
        let dispatch = std::fs::read_dir(root)
            .expect("dispatch directory")
            .next()
            .expect("dispatch entry")
            .expect("dispatch entry valid")
            .path();
        std::fs::read_dir(dispatch)
            .expect("segment directory")
            .next()
            .expect("segment entry")
            .expect("segment entry valid")
            .path()
    }

    fn walk_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        fn collect(
            root: &std::path::Path,
            current: &std::path::Path,
            paths: &mut Vec<std::path::PathBuf>,
        ) {
            for entry in std::fs::read_dir(current).expect("directory reads") {
                let path = entry.expect("entry reads").path();
                if path.is_dir() {
                    collect(root, &path, paths);
                } else {
                    paths.push(path.strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
        let mut paths = Vec::new();
        collect(root, root, &mut paths);
        paths.sort();
        paths
    }
}
