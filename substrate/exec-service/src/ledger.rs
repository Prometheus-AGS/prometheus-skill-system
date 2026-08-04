use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{
    verify_receipt, Digest, ExecutionReceipt, RunState, SignedExecRequest, VerificationKey,
};
use prometheus_exec_core::{AppendReceiptResult, ReceiptLog, ReceiptLogError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const LEDGER_SCHEMA_VERSION: &str = "1";
const MAX_RECORD_BYTES: u64 = 8 * 1024 * 1024;

/// Durable evidence that the executor has, or has not, crossed the spawn boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum SpawnStatus {
    NotSpawned,
    Spawned { spawned_at: DateTime<Utc> },
}

/// Receipt-log coordinates committed before the terminal ledger state is visible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalReceiptRecord {
    pub receipt: ExecutionReceipt,
    pub receipt_hash: Digest,
    pub log_sequence: u64,
    pub log_segment_hash: Digest,
}

/// The persisted identity and lifecycle state for one idempotent request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub schema_version: String,
    pub request_id: Uuid,
    pub request_hash: Digest,
    pub run_id: Uuid,
    pub state: RunState,
    pub spawn: SpawnStatus,
    pub request: SignedExecRequest,
    pub accepted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<TerminalReceiptRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcceptRunResult {
    Accepted(RunRecord),
    Replay(RunRecord),
}

impl AcceptRunResult {
    pub fn record(&self) -> &RunRecord {
        match self {
            Self::Accepted(record) | Self::Replay(record) => record,
        }
    }

    pub fn created(&self) -> bool {
        matches!(self, Self::Accepted(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalCommitResult {
    pub record: RunRecord,
    pub created: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReconciliationReport {
    pub recovered_from_log: Vec<Uuid>,
    pub requeued: Vec<Uuid>,
    pub interrupted: Vec<Uuid>,
    pub orphan_receipts: Vec<Uuid>,
}

#[derive(Clone, Debug)]
pub struct RunLedger {
    root: PathBuf,
    receipt_log: ReceiptLog,
}

#[derive(Debug, Error)]
pub enum RunLedgerError {
    #[error("run ledger I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("run ledger JSON failed at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("request contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("receipt log failed: {0}")]
    ReceiptLog(#[from] ReceiptLogError),
    #[error(
        "request id {request_id} already exists with hash {existing}; candidate is {candidate}"
    )]
    RequestHashConflict {
        request_id: Uuid,
        existing: Digest,
        candidate: Digest,
    },
    #[error("run record was not found for request id {0}")]
    NotFound(Uuid),
    #[error("run record is invalid: {0}")]
    InvalidRecord(String),
    #[error("terminal receipt is invalid: {0}")]
    InvalidReceipt(String),
    #[error("restart reconciliation could not create an interrupted receipt: {0}")]
    Reconciliation(String),
    #[error("run record exceeds {MAX_RECORD_BYTES} bytes: {0}")]
    RecordTooLarge(PathBuf),
}

impl RunLedger {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RunLedgerError> {
        let root = root.into();
        create_private_dir(&root)?;
        create_private_dir(&root.join("runs"))?;
        let ledger = Self {
            receipt_log: ReceiptLog::open(root.join("receipts"))?,
            root,
        };
        let lock = ledger.open_lock()?;
        fs2::FileExt::lock_shared(&lock).map_err(|source| io_error(ledger.lock_path(), source))?;
        let records = ledger.load_records()?;
        let logged = ledger.logged_receipts()?;
        for record in records {
            if let Some(terminal) = &record.terminal {
                let Some((receipt, location)) = logged.get(&record.run_id) else {
                    return Err(RunLedgerError::InvalidRecord(format!(
                        "terminal run {} is absent from the receipt log",
                        record.run_id
                    )));
                };
                if receipt != &terminal.receipt
                    || location.receipt_hash != terminal.receipt_hash
                    || location.sequence != terminal.log_sequence
                    || location.segment_hash != terminal.log_segment_hash
                {
                    return Err(RunLedgerError::InvalidRecord(format!(
                        "terminal run {} disagrees with the receipt log",
                        record.run_id
                    )));
                }
            }
        }
        Ok(ledger)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn receipt_log(&self) -> &ReceiptLog {
        &self.receipt_log
    }

    /// Persist request identity before an executor is allowed to spawn.
    pub fn accept(&self, request: SignedExecRequest) -> Result<AcceptRunResult, RunLedgerError> {
        request.validate()?;
        let request_hash = request.request_hash()?;
        let lock = self.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(self.lock_path(), source))?;

        if let Some(existing) = self.load_record_if_present(request.request_id)? {
            if existing.request_hash == request_hash {
                return Ok(AcceptRunResult::Replay(existing));
            }
            return Err(RunLedgerError::RequestHashConflict {
                request_id: request.request_id,
                existing: existing.request_hash,
                candidate: request_hash,
            });
        }

        let now = Utc::now();
        let record = RunRecord {
            schema_version: LEDGER_SCHEMA_VERSION.into(),
            request_id: request.request_id,
            request_hash,
            run_id: Uuid::new_v4(),
            state: RunState::Queued,
            spawn: SpawnStatus::NotSpawned,
            request,
            accepted_at: now,
            updated_at: now,
            revision: 0,
            terminal: None,
        };
        validate_record(&record)?;
        self.write_record(&record, false)?;
        Ok(AcceptRunResult::Accepted(record))
    }

    pub fn get(&self, request_id: Uuid) -> Result<Option<RunRecord>, RunLedgerError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_shared(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        self.load_record_if_present(request_id)
    }

    pub fn get_by_run_id(&self, run_id: Uuid) -> Result<Option<RunRecord>, RunLedgerError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_shared(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        Ok(self
            .load_records()?
            .into_iter()
            .find(|record| record.run_id == run_id))
    }

    pub fn records(&self) -> Result<Vec<RunRecord>, RunLedgerError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_shared(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        self.load_records()
    }

    /// Durably records the spawn boundary before the backend process is started.
    pub fn mark_spawned(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
    ) -> Result<RunRecord, RunLedgerError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        let mut record = self
            .load_record_if_present(request_id)?
            .ok_or(RunLedgerError::NotFound(request_id))?;
        require_hash(&record, request_hash)?;
        if record.state.is_terminal() {
            return Err(RunLedgerError::InvalidRecord(format!(
                "run {} is already terminal",
                record.run_id
            )));
        }
        if matches!(record.spawn, SpawnStatus::Spawned { .. }) {
            return Ok(record);
        }
        let now = Utc::now();
        record.state = RunState::Running;
        record.spawn = SpawnStatus::Spawned { spawned_at: now };
        record.updated_at = now;
        record.revision = record.revision.saturating_add(1);
        self.write_record(&record, true)?;
        Ok(record)
    }

    /// Appends the signed receipt before publishing terminal state in the ledger.
    pub fn commit_terminal(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
        receipt: ExecutionReceipt,
        verification_key: &VerificationKey,
    ) -> Result<TerminalCommitResult, RunLedgerError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        let mut record = self
            .load_record_if_present(request_id)?
            .ok_or(RunLedgerError::NotFound(request_id))?;
        require_hash(&record, request_hash)?;
        validate_receipt_for_record(&receipt, &record, verification_key)?;

        if let Some(terminal) = &record.terminal {
            let candidate_hash = receipt.receipt_hash()?;
            if terminal.receipt_hash == candidate_hash && terminal.receipt == receipt {
                return Ok(TerminalCommitResult {
                    record,
                    created: false,
                });
            }
            return Err(RunLedgerError::InvalidReceipt(format!(
                "run {} already has a different terminal receipt",
                record.run_id
            )));
        }

        let appended = self.receipt_log.append(receipt.clone(), verification_key)?;
        apply_terminal(&mut record, receipt, appended);
        self.write_record(&record, true)?;
        Ok(TerminalCommitResult {
            record,
            created: true,
        })
    }

    /// Reconciles crash windows without ever executing a request a second time.
    pub fn reconcile<F>(
        &self,
        verification_key: &VerificationKey,
        mut interrupted_receipt: F,
    ) -> Result<ReconciliationReport, RunLedgerError>
    where
        F: FnMut(&RunRecord) -> Result<ExecutionReceipt, String>,
    {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        let mut records = self.load_records()?;
        let logged = self.logged_receipts()?;
        let known_runs: HashSet<_> = records.iter().map(|record| record.run_id).collect();
        let mut report = ReconciliationReport {
            orphan_receipts: logged
                .keys()
                .filter(|run_id| !known_runs.contains(run_id))
                .copied()
                .collect(),
            ..ReconciliationReport::default()
        };

        for record in &mut records {
            if record.state.is_terminal() {
                continue;
            }
            if let Some((receipt, appended)) = logged.get(&record.run_id) {
                validate_receipt_for_record(receipt, record, verification_key)?;
                apply_terminal(record, receipt.clone(), appended.clone());
                self.write_record(record, true)?;
                report.recovered_from_log.push(record.request_id);
                continue;
            }
            match record.spawn {
                SpawnStatus::NotSpawned => {
                    if record.state != RunState::Queued {
                        record.state = RunState::Queued;
                        record.updated_at = Utc::now();
                        record.revision = record.revision.saturating_add(1);
                        self.write_record(record, true)?;
                    }
                    report.requeued.push(record.request_id);
                }
                SpawnStatus::Spawned { .. } => {
                    let receipt =
                        interrupted_receipt(record).map_err(RunLedgerError::Reconciliation)?;
                    validate_receipt_for_record(&receipt, record, verification_key)?;
                    if receipt.state != RunState::Interrupted {
                        return Err(RunLedgerError::InvalidReceipt(
                            "restart reconciliation requires an interrupted receipt".into(),
                        ));
                    }
                    let appended = self.receipt_log.append(receipt.clone(), verification_key)?;
                    apply_terminal(record, receipt, appended);
                    self.write_record(record, true)?;
                    report.interrupted.push(record.request_id);
                }
            }
        }
        report.orphan_receipts.sort();
        Ok(report)
    }

    fn logged_receipts(
        &self,
    ) -> Result<BTreeMap<Uuid, (ExecutionReceipt, AppendReceiptResult)>, RunLedgerError> {
        let mut receipts = BTreeMap::new();
        for segment in self.receipt_log.segments()? {
            let segment_hash = segment.segment_hash.clone().ok_or_else(|| {
                RunLedgerError::InvalidRecord("receipt log contains an unsealed segment".into())
            })?;
            for entry in segment.entries {
                receipts.insert(
                    entry.receipt.run_id,
                    (
                        entry.receipt,
                        AppendReceiptResult {
                            sequence: segment.header.sequence,
                            segment_hash: segment_hash.clone(),
                            receipt_hash: entry.receipt_hash,
                            created: false,
                        },
                    ),
                );
            }
        }
        Ok(receipts)
    }

    fn load_records(&self) -> Result<Vec<RunRecord>, RunLedgerError> {
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(self.runs_root()).map_err(|source| io_error(self.runs_root(), source))?
        {
            let entry = entry.map_err(|source| io_error(self.runs_root(), source))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(RunLedgerError::InvalidRecord(format!(
                    "unsafe run entry: {}",
                    path.display()
                )));
            }
            if metadata.len() > MAX_RECORD_BYTES {
                return Err(RunLedgerError::RecordTooLarge(path));
            }
            paths.push(path);
        }
        paths.sort();
        let mut records = Vec::with_capacity(paths.len());
        let mut run_ids = HashSet::new();
        for path in paths {
            let record = read_record(&path)?;
            let expected = format!("{}.json", record.request_id);
            if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
                return Err(RunLedgerError::InvalidRecord(format!(
                    "record filename does not match request id: {}",
                    path.display()
                )));
            }
            validate_record(&record)?;
            if !run_ids.insert(record.run_id) {
                return Err(RunLedgerError::InvalidRecord(format!(
                    "duplicate run id {}",
                    record.run_id
                )));
            }
            records.push(record);
        }
        Ok(records)
    }

    fn load_record_if_present(
        &self,
        request_id: Uuid,
    ) -> Result<Option<RunRecord>, RunLedgerError> {
        let path = self.record_path(request_id);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(RunLedgerError::InvalidRecord(format!(
                        "unsafe run record: {}",
                        path.display()
                    )));
                }
                let record = read_record(&path)?;
                validate_record(&record)?;
                Ok(Some(record))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(io_error(path, source)),
        }
    }

    fn write_record(&self, record: &RunRecord, replace: bool) -> Result<(), RunLedgerError> {
        validate_record(record)?;
        let path = self.record_path(record.request_id);
        let mut bytes =
            serde_json::to_vec_pretty(record).map_err(|source| RunLedgerError::Json {
                path: path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        let parent = self.runs_root();
        let mut temporary =
            tempfile::NamedTempFile::new_in(&parent).map_err(|source| io_error(&parent, source))?;
        set_private_file(temporary.as_file(), temporary.path())?;
        temporary
            .write_all(&bytes)
            .map_err(|source| io_error(temporary.path(), source))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| io_error(temporary.path(), source))?;
        if replace {
            temporary
                .persist(&path)
                .map_err(|error| io_error(&path, error.error))?;
        } else {
            temporary
                .persist_noclobber(&path)
                .map_err(|error| io_error(&path, error.error))?;
        }
        sync_directory(&parent)
    }

    fn open_lock(&self) -> Result<File, RunLedgerError> {
        let path = self.lock_path();
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

    fn runs_root(&self) -> PathBuf {
        self.root.join("runs")
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("writer.lock")
    }

    fn record_path(&self, request_id: Uuid) -> PathBuf {
        self.runs_root().join(format!("{request_id}.json"))
    }
}

fn validate_record(record: &RunRecord) -> Result<(), RunLedgerError> {
    if record.schema_version != LEDGER_SCHEMA_VERSION {
        return Err(RunLedgerError::InvalidRecord(format!(
            "unsupported ledger schema version {}",
            record.schema_version
        )));
    }
    record.request.validate()?;
    if record.request_id != record.request.request_id {
        return Err(RunLedgerError::InvalidRecord(
            "request id does not match persisted request".into(),
        ));
    }
    if record.request_hash != record.request.request_hash()? {
        return Err(RunLedgerError::InvalidRecord(
            "request hash does not match persisted request".into(),
        ));
    }
    if record.updated_at < record.accepted_at {
        return Err(RunLedgerError::InvalidRecord(
            "updated timestamp precedes accepted timestamp".into(),
        ));
    }
    match (&record.terminal, record.state.is_terminal()) {
        (Some(terminal), true) => {
            if terminal.receipt.run_id != record.run_id
                || terminal.receipt.request_hash != record.request_hash
                || terminal.receipt.state != record.state
                || terminal.receipt.receipt_hash()? != terminal.receipt_hash
            {
                return Err(RunLedgerError::InvalidRecord(
                    "terminal receipt does not match run record".into(),
                ));
            }
        }
        (None, false) => {}
        _ => {
            return Err(RunLedgerError::InvalidRecord(
                "terminal state and receipt presence disagree".into(),
            ))
        }
    }
    Ok(())
}

fn validate_receipt_for_record(
    receipt: &ExecutionReceipt,
    record: &RunRecord,
    verification_key: &VerificationKey,
) -> Result<(), RunLedgerError> {
    if receipt.run_id != record.run_id {
        return Err(RunLedgerError::InvalidReceipt(format!(
            "receipt run id {} does not match {}",
            receipt.run_id, record.run_id
        )));
    }
    if receipt.request_hash != record.request_hash {
        return Err(RunLedgerError::InvalidReceipt(
            "receipt request hash does not match ledger".into(),
        ));
    }
    let verification = verify_receipt(receipt, verification_key, Some(&record.request), None);
    if !verification.valid {
        return Err(RunLedgerError::InvalidReceipt(
            verification
                .failures
                .into_iter()
                .map(|failure| failure.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    Ok(())
}

fn apply_terminal(
    record: &mut RunRecord,
    receipt: ExecutionReceipt,
    appended: AppendReceiptResult,
) {
    record.state = receipt.state;
    record.updated_at = Utc::now();
    record.revision = record.revision.saturating_add(1);
    record.terminal = Some(TerminalReceiptRecord {
        receipt,
        receipt_hash: appended.receipt_hash,
        log_sequence: appended.sequence,
        log_segment_hash: appended.segment_hash,
    });
}

fn require_hash(record: &RunRecord, candidate: &Digest) -> Result<(), RunLedgerError> {
    if &record.request_hash == candidate {
        return Ok(());
    }
    Err(RunLedgerError::RequestHashConflict {
        request_id: record.request_id,
        existing: record.request_hash.clone(),
        candidate: candidate.clone(),
    })
}

fn read_record(path: &Path) -> Result<RunRecord, RunLedgerError> {
    let file = File::open(path).map_err(|source| io_error(path, source))?;
    let mut bytes = Vec::new();
    file.take(MAX_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(RunLedgerError::RecordTooLarge(path.to_path_buf()));
    }
    serde_json::from_slice(&bytes).map_err(|source| RunLedgerError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn create_private_dir(path: &Path) -> Result<(), RunLedgerError> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn set_private_file(file: &File, path: &Path) -> Result<(), RunLedgerError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), RunLedgerError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(path, source))
}

fn io_error(path: impl AsRef<Path>, source: std::io::Error) -> RunLedgerError {
    RunLedgerError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    }
}
