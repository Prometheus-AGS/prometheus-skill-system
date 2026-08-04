use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{Digest, ExecutionReceipt, SignedExecRequest, VerificationKey};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    AcceptRunResult, AppendEventResult, GrantPendingRecord, ReconciliationReport, RunEvent,
    RunEventData, RunEventLog, RunEventLogError, RunLedger, RunLedgerError, RunRecord,
    SpawnRunResult, SpawnStatus, TerminalCommitResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmitRunResult {
    pub record: RunRecord,
    pub replayed: bool,
}

#[derive(Clone, Debug)]
pub struct ExecutionService {
    root: PathBuf,
    ledger: RunLedger,
    event_log: RunEventLog,
}

#[derive(Debug, Error)]
pub enum ExecutionServiceError {
    #[error("run ledger failed: {0}")]
    Ledger(#[from] RunLedgerError),
    #[error("run event log failed: {0}")]
    EventLog(#[from] RunEventLogError),
    #[error("run id was not found: {0}")]
    RunNotFound(Uuid),
    #[error("run {0} is already terminal")]
    RunTerminal(Uuid),
    #[error("run {0} has not crossed the durable spawn boundary")]
    RunNotSpawned(Uuid),
    #[error("lifecycle events can only be emitted by the service")]
    ReservedLifecycleEvent,
    #[error("execution service I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl ExecutionService {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ExecutionServiceError> {
        let root = root.into();
        create_private_dir(&root)?;
        let service = Self {
            ledger: RunLedger::open(root.join("ledger"))?,
            event_log: RunEventLog::open(root.join("events"))?,
            root,
        };
        let lock = service.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| service.io_error(service.lock_path(), source))?;
        service.synchronize_all_events()?;
        Ok(service)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ledger(&self) -> &RunLedger {
        &self.ledger
    }

    pub fn event_log(&self) -> &RunEventLog {
        &self.event_log
    }

    pub fn submit(
        &self,
        request: SignedExecRequest,
    ) -> Result<SubmitRunResult, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let accepted = self.ledger.accept(request)?;
        let replayed = !accepted.created();
        let record = accepted.record().clone();
        self.synchronize_events(&record)?;
        Ok(SubmitRunResult { record, replayed })
    }

    pub fn run(&self, run_id: Uuid) -> Result<Option<RunRecord>, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_shared(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        Ok(self.ledger.get_by_run_id(run_id)?)
    }

    pub fn receipt(&self, run_id: Uuid) -> Result<Option<ExecutionReceipt>, ExecutionServiceError> {
        Ok(self
            .run(run_id)?
            .and_then(|record| record.terminal.map(|terminal| terminal.receipt)))
    }

    pub fn mark_spawned(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
    ) -> Result<RunRecord, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let record = self.ledger.mark_spawned(request_id, request_hash)?;
        self.synchronize_events(&record)?;
        Ok(record)
    }

    pub fn claim_for_execution(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
    ) -> Result<SpawnRunResult, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let claimed = self.ledger.claim_for_execution(request_id, request_hash)?;
        self.synchronize_events(&claimed.record)?;
        Ok(claimed)
    }

    pub fn mark_grant_pending(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
        event_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<RunRecord, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let pending = GrantPendingRecord {
            event_id: event_id.into(),
            reason: reason.into(),
            occurred_at: Utc::now(),
        };
        let record = self
            .ledger
            .mark_grant_pending(request_id, request_hash, pending)?;
        self.synchronize_events(&record)?;
        Ok(record)
    }

    pub fn append_runtime_event(
        &self,
        run_id: Uuid,
        event_id: impl Into<String>,
        occurred_at: DateTime<Utc>,
        data: RunEventData,
    ) -> Result<AppendEventResult, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        if data.is_lifecycle() {
            return Err(ExecutionServiceError::ReservedLifecycleEvent);
        }
        let record = self
            .ledger
            .get_by_run_id(run_id)?
            .ok_or(ExecutionServiceError::RunNotFound(run_id))?;
        if record.state.is_terminal() {
            return Err(ExecutionServiceError::RunTerminal(run_id));
        }
        if !matches!(record.spawn, SpawnStatus::Spawned { .. }) {
            return Err(ExecutionServiceError::RunNotSpawned(run_id));
        }
        Ok(self.event_log.append(run_id, event_id, occurred_at, data)?)
    }

    pub fn commit_terminal(
        &self,
        request_id: Uuid,
        request_hash: &Digest,
        receipt: ExecutionReceipt,
        verification_key: &VerificationKey,
    ) -> Result<TerminalCommitResult, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let committed =
            self.ledger
                .commit_terminal(request_id, request_hash, receipt, verification_key)?;
        self.synchronize_events(&committed.record)?;
        Ok(committed)
    }

    pub fn events_after(
        &self,
        run_id: Uuid,
        after: u64,
    ) -> Result<Vec<RunEvent>, ExecutionServiceError> {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_shared(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        if self.ledger.get_by_run_id(run_id)?.is_none() {
            return Err(ExecutionServiceError::RunNotFound(run_id));
        }
        Ok(self.event_log.events_after(run_id, after)?)
    }

    pub fn reconcile<F>(
        &self,
        verification_key: &VerificationKey,
        interrupted_receipt: F,
    ) -> Result<ReconciliationReport, ExecutionServiceError>
    where
        F: FnMut(&RunRecord) -> Result<ExecutionReceipt, String>,
    {
        let lock = self.open_operation_lock()?;
        fs2::FileExt::lock_exclusive(&lock)
            .map_err(|source| self.io_error(self.lock_path(), source))?;
        let report = self
            .ledger
            .reconcile(verification_key, interrupted_receipt)?;
        self.synchronize_all_events()?;
        Ok(report)
    }

    fn synchronize_all_events(&self) -> Result<(), ExecutionServiceError> {
        for record in self.ledger.records()? {
            self.synchronize_events(&record)?;
        }
        Ok(())
    }

    fn synchronize_events(&self, record: &RunRecord) -> Result<(), ExecutionServiceError> {
        self.event_log.append(
            record.run_id,
            "run.accepted",
            record.accepted_at,
            RunEventData::Accepted {
                request_id: record.request_id,
                request_hash: record.request_hash.clone(),
            },
        )?;
        if let SpawnStatus::Spawned { spawned_at } = &record.spawn {
            self.event_log.append(
                record.run_id,
                "run.started",
                *spawned_at,
                RunEventData::Started,
            )?;
        }
        if let Some(pending) = &record.grant_pending {
            self.event_log.append(
                record.run_id,
                &pending.event_id,
                pending.occurred_at,
                RunEventData::GrantPending {
                    reason: pending.reason.clone(),
                },
            )?;
        }
        if let Some(terminal) = &record.terminal {
            self.event_log.append(
                record.run_id,
                "run.completed",
                terminal.receipt.finished_at,
                RunEventData::Completed {
                    state: terminal.receipt.state,
                    receipt_hash: terminal.receipt_hash.clone(),
                },
            )?;
        }
        Ok(())
    }

    fn open_operation_lock(&self) -> Result<File, ExecutionServiceError> {
        let path = self.lock_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| self.io_error(&path, source))?;
        set_private_file(&file, &path)?;
        Ok(file)
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("service-writer.lock")
    }

    fn io_error(&self, path: impl AsRef<Path>, source: std::io::Error) -> ExecutionServiceError {
        ExecutionServiceError::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

impl From<AcceptRunResult> for SubmitRunResult {
    fn from(value: AcceptRunResult) -> Self {
        let replayed = !value.created();
        Self {
            record: value.record().clone(),
            replayed,
        }
    }
}

fn create_private_dir(path: &Path) -> Result<(), ExecutionServiceError> {
    fs::create_dir_all(path).map_err(|source| ExecutionServiceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
            ExecutionServiceError::Io {
                path: path.to_path_buf(),
                source,
            }
        })?;
    }
    Ok(())
}

fn set_private_file(file: &File, path: &Path) -> Result<(), ExecutionServiceError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| ExecutionServiceError::Io {
                path: path.to_path_buf(),
                source,
            })?;
    }
    Ok(())
}
