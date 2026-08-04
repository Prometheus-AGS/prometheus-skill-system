use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{hash_serializable, Digest, RunState};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const EVENT_SCHEMA_VERSION: &str = "1";
const MAX_EVENT_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_EVENT_ID_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum RunEventData {
    Accepted {
        request_id: Uuid,
        request_hash: Digest,
    },
    GrantPending {
        reason: String,
    },
    Started,
    Stdout {
        chunk: String,
    },
    Stderr {
        chunk: String,
    },
    Progress {
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        completed: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        total: Option<u64>,
    },
    Completed {
        state: RunState,
        receipt_hash: Digest,
    },
}

impl RunEventData {
    pub fn is_lifecycle(&self) -> bool {
        matches!(
            self,
            Self::Accepted { .. } | Self::Started | Self::Completed { .. }
        )
    }

    fn validate(&self) -> Result<(), RunEventLogError> {
        match self {
            Self::GrantPending { reason } if reason.trim().is_empty() => Err(
                RunEventLogError::InvalidEvent("grant-pending reason cannot be empty".into()),
            ),
            Self::Progress {
                completed: Some(completed),
                total: Some(total),
                ..
            } if completed > total => Err(RunEventLogError::InvalidEvent(
                "progress completed count exceeds total".into(),
            )),
            Self::Completed { state, .. } if !state.is_terminal() => Err(
                RunEventLogError::InvalidEvent("completed event state must be terminal".into()),
            ),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub schema_version: String,
    pub run_id: Uuid,
    pub sequence: u64,
    pub event_id: String,
    pub occurred_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_event_hash: Option<Digest>,
    #[serde(flatten)]
    pub data: RunEventData,
    pub event_hash: Digest,
}

impl RunEvent {
    fn compute_hash(&self) -> Result<Digest, RunEventLogError> {
        let mut value = serde_json::to_value(self).map_err(|source| RunEventLogError::Json {
            path: PathBuf::from("<event>"),
            source,
        })?;
        let object = value.as_object_mut().ok_or_else(|| {
            RunEventLogError::InvalidEvent("serialized event is not an object".into())
        })?;
        object.remove("eventHash");
        hash_serializable(&value).map_err(RunEventLogError::Contract)
    }

    fn validate(&self) -> Result<(), RunEventLogError> {
        if self.schema_version != EVENT_SCHEMA_VERSION {
            return Err(RunEventLogError::InvalidEvent(format!(
                "unsupported event schema version {}",
                self.schema_version
            )));
        }
        validate_event_id(&self.event_id)?;
        self.data.validate()?;
        if self.compute_hash()? != self.event_hash {
            return Err(RunEventLogError::InvalidEvent(format!(
                "event hash mismatch at sequence {}",
                self.sequence
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppendEventResult {
    pub event: RunEvent,
    pub created: bool,
}

#[derive(Clone, Debug)]
pub struct RunEventLog {
    root: PathBuf,
}

#[derive(Debug, Error)]
pub enum RunEventLogError {
    #[error("event log I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("event log JSON failed at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("event contract failed: {0}")]
    Contract(#[source] prometheus_exec_contracts::ContractError),
    #[error("event log is invalid: {0}")]
    InvalidEvent(String),
    #[error("event id {event_id} already exists with different content for run {run_id}")]
    EventIdConflict { run_id: Uuid, event_id: String },
    #[error("event file exceeds {MAX_EVENT_FILE_BYTES} bytes: {0}")]
    EventTooLarge(PathBuf),
}

impl RunEventLog {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RunEventLogError> {
        let log = Self { root: root.into() };
        create_private_dir(&log.root)?;
        for entry in fs::read_dir(&log.root).map_err(|source| io_error(&log.root, source))? {
            let entry = entry.map_err(|source| io_error(&log.root, source))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(RunEventLogError::InvalidEvent(format!(
                    "unsafe run event directory: {}",
                    path.display()
                )));
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    RunEventLogError::InvalidEvent(format!(
                        "run event directory is not UTF-8: {}",
                        path.display()
                    ))
                })?;
            let run_id = Uuid::parse_str(name).map_err(|_| {
                RunEventLogError::InvalidEvent(format!(
                    "run event directory is not a UUID: {}",
                    path.display()
                ))
            })?;
            log.events(run_id)?;
        }
        Ok(log)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn append(
        &self,
        run_id: Uuid,
        event_id: impl Into<String>,
        occurred_at: DateTime<Utc>,
        data: RunEventData,
    ) -> Result<AppendEventResult, RunEventLogError> {
        let event_id = event_id.into();
        validate_event_id(&event_id)?;
        data.validate()?;
        self.prepare_run(run_id)?;
        let lock = self.open_lock(run_id)?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| io_error(self.lock_path(run_id), source))?;
        let existing = self.load_events(run_id)?;
        if let Some(event) = existing.iter().find(|event| event.event_id == event_id) {
            if event.occurred_at == occurred_at && event.data == data {
                return Ok(AppendEventResult {
                    event: event.clone(),
                    created: false,
                });
            }
            return Err(RunEventLogError::EventIdConflict { run_id, event_id });
        }

        let sequence = existing.last().map(|event| event.sequence + 1).unwrap_or(1);
        let previous_event_hash = existing.last().map(|event| event.event_hash.clone());
        let mut event = RunEvent {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            run_id,
            sequence,
            event_id,
            occurred_at,
            previous_event_hash,
            data,
            event_hash: Digest::from_bytes(b"pending"),
        };
        event.event_hash = event.compute_hash()?;
        let path = self.event_path(run_id, &event);
        let mut bytes =
            serde_json::to_vec_pretty(&event).map_err(|source| RunEventLogError::Json {
                path: path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        atomic_create(&path, &bytes)?;
        sync_directory(&self.segment_root(run_id))?;
        Ok(AppendEventResult {
            event,
            created: true,
        })
    }

    pub fn events(&self, run_id: Uuid) -> Result<Vec<RunEvent>, RunEventLogError> {
        let run_root = self.run_root(run_id);
        if !run_root.exists() {
            return Ok(vec![]);
        }
        let metadata =
            fs::symlink_metadata(&run_root).map_err(|source| io_error(&run_root, source))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(RunEventLogError::InvalidEvent(format!(
                "unsafe run event directory: {}",
                run_root.display()
            )));
        }
        let lock = self.open_lock(run_id)?;
        fs2::FileExt::lock_shared(&lock)
            .map_err(|source| io_error(self.lock_path(run_id), source))?;
        self.load_events(run_id)
    }

    pub fn events_after(
        &self,
        run_id: Uuid,
        after: u64,
    ) -> Result<Vec<RunEvent>, RunEventLogError> {
        Ok(self
            .events(run_id)?
            .into_iter()
            .filter(|event| event.sequence > after)
            .collect())
    }

    fn prepare_run(&self, run_id: Uuid) -> Result<(), RunEventLogError> {
        create_private_dir(&self.run_root(run_id))?;
        create_private_dir(&self.segment_root(run_id))
    }

    fn load_events(&self, run_id: Uuid) -> Result<Vec<RunEvent>, RunEventLogError> {
        let segment_root = self.segment_root(run_id);
        if !segment_root.exists() {
            return Ok(vec![]);
        }
        let metadata = fs::symlink_metadata(&segment_root)
            .map_err(|source| io_error(&segment_root, source))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(RunEventLogError::InvalidEvent(format!(
                "unsafe event segment directory: {}",
                segment_root.display()
            )));
        }
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(&segment_root).map_err(|source| io_error(&segment_root, source))?
        {
            let entry = entry.map_err(|source| io_error(&segment_root, source))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(RunEventLogError::InvalidEvent(format!(
                    "unsafe event entry: {}",
                    path.display()
                )));
            }
            if metadata.len() > MAX_EVENT_FILE_BYTES {
                return Err(RunEventLogError::EventTooLarge(path));
            }
            paths.push(path);
        }
        paths.sort();

        let mut events = Vec::with_capacity(paths.len());
        let mut previous = None;
        for (index, path) in paths.into_iter().enumerate() {
            let event = read_event(&path)?;
            let expected_sequence = index as u64 + 1;
            if event.run_id != run_id
                || event.sequence != expected_sequence
                || event.previous_event_hash != previous
            {
                return Err(RunEventLogError::InvalidEvent(format!(
                    "event chain mismatch at {}",
                    path.display()
                )));
            }
            event.validate()?;
            validate_filename(&path, &event)?;
            previous = Some(event.event_hash.clone());
            events.push(event);
        }
        Ok(events)
    }

    fn open_lock(&self, run_id: Uuid) -> Result<File, RunEventLogError> {
        self.prepare_run(run_id)?;
        let path = self.lock_path(run_id);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| io_error(&path, source))?;
        set_private_file(&file, &path)?;
        Ok(file)
    }

    fn run_root(&self, run_id: Uuid) -> PathBuf {
        self.root.join(run_id.to_string())
    }

    fn segment_root(&self, run_id: Uuid) -> PathBuf {
        self.run_root(run_id).join("segments")
    }

    fn lock_path(&self, run_id: Uuid) -> PathBuf {
        self.run_root(run_id).join("writer.lock")
    }

    fn event_path(&self, run_id: Uuid, event: &RunEvent) -> PathBuf {
        self.segment_root(run_id).join(format!(
            "{:020}-{}.json",
            event.sequence,
            event.event_hash.as_str().trim_start_matches("sha256:")
        ))
    }
}

fn validate_event_id(event_id: &str) -> Result<(), RunEventLogError> {
    if event_id.is_empty()
        || event_id.len() > MAX_EVENT_ID_BYTES
        || !event_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
    {
        return Err(RunEventLogError::InvalidEvent(format!(
            "unsafe event id: {event_id:?}"
        )));
    }
    Ok(())
}

fn validate_filename(path: &Path, event: &RunEvent) -> Result<(), RunEventLogError> {
    let expected = format!(
        "{:020}-{}.json",
        event.sequence,
        event.event_hash.as_str().trim_start_matches("sha256:")
    );
    if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
        return Err(RunEventLogError::InvalidEvent(format!(
            "event filename does not match content: {}",
            path.display()
        )));
    }
    Ok(())
}

fn read_event(path: &Path) -> Result<RunEvent, RunEventLogError> {
    let file = File::open(path).map_err(|source| io_error(path, source))?;
    let mut bytes = Vec::new();
    file.take(MAX_EVENT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    if bytes.len() as u64 > MAX_EVENT_FILE_BYTES {
        return Err(RunEventLogError::EventTooLarge(path.to_path_buf()));
    }
    serde_json::from_slice(&bytes).map_err(|source| RunEventLogError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn atomic_create(path: &Path, bytes: &[u8]) -> Result<(), RunEventLogError> {
    let parent = path
        .parent()
        .ok_or_else(|| RunEventLogError::InvalidEvent("event has no parent".into()))?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|source| io_error(parent, source))?;
    set_private_file(temporary.as_file(), temporary.path())?;
    temporary
        .write_all(bytes)
        .map_err(|source| io_error(temporary.path(), source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| io_error(temporary.path(), source))?;
    temporary
        .persist_noclobber(path)
        .map_err(|error| io_error(path, error.error))?;
    Ok(())
}

fn create_private_dir(path: &Path) -> Result<(), RunEventLogError> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn set_private_file(file: &File, path: &Path) -> Result<(), RunEventLogError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), RunEventLogError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(path, source))
}

fn io_error(path: impl AsRef<Path>, source: std::io::Error) -> RunEventLogError {
    RunEventLogError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    }
}
