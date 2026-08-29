//! Authoritative command facade shared by REST, MCP, and the daemon.
//!
//! Callers submit the same versioned `CommandEnvelope`. During stabilization,
//! commands are committed by one journal writer protected by an exclusive
//! lock. The lock is the sole write-concurrency boundary.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Instant,
};

use kbd_runtime::{
    registry::{ProjectRegistry, RegistrationOutcome, RegistryDocument, ReplicaRegistration},
    CommandEnvelope, CommandResult, DeviceStatus, Event, KbdStateV2, Runtime,
    SignedCommandEnvelope,
};
use serde::Serialize;

#[cfg(test)]
static AUTHORITY_OPEN_COUNTS: std::sync::LazyLock<std::sync::Mutex<BTreeMap<String, usize>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommittedCommand {
    #[serde(flatten)]
    pub result: CommandResult,
    pub projection_error: Option<String>,
}

#[derive(Clone)]
/// The KBD control plane.
///
/// `Runtime` provides an append-only `events.jsonl` guarded by an exclusive
/// flock and fsynced on append before import into the authoritative project
/// Loro document. The journal lock permits exactly one local writer.
pub struct KbdControlPlane {
    runtime: Arc<Runtime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredProjectRoute {
    pub project_id: String,
    pub path: PathBuf,
    pub replica: ReplicaRegistration,
    pub ready: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct KbdProjectRouter {
    registry: ProjectRegistry,
    data_root: Option<PathBuf>,
    controls: Arc<RwLock<BTreeMap<String, Arc<KbdControlPlane>>>>,
    errors: Arc<RwLock<BTreeMap<String, String>>>,
}

impl KbdProjectRouter {
    pub async fn open_registered() -> io::Result<Self> {
        let router = Self {
            registry: ProjectRegistry::open(),
            data_root: None,
            controls: Arc::new(RwLock::new(BTreeMap::new())),
            errors: Arc::new(RwLock::new(BTreeMap::new())),
        };
        router.reload().await?;
        Ok(router)
    }

    pub async fn open_registered_at(data_root: &Path) -> io::Result<Self> {
        let router = Self {
            registry: ProjectRegistry::open_at(data_root),
            data_root: Some(data_root.to_path_buf()),
            controls: Arc::new(RwLock::new(BTreeMap::new())),
            errors: Arc::new(RwLock::new(BTreeMap::new())),
        };
        router.reload().await?;
        Ok(router)
    }

    pub async fn open_with_project(project_root: &Path) -> io::Result<Self> {
        let registry = ProjectRegistry::open();
        registry
            .register_existing(project_root)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::open_registered().await
    }

    pub async fn open_with_project_at(project_root: &Path, data_root: &Path) -> io::Result<Self> {
        // Establishing a canonical runtime is the explicit project-initialization
        // path. Registry registration itself still consumes only the manifest.
        Runtime::open_canonical_at(project_root, data_root)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::open_registered_at(data_root).await
    }

    pub fn registry(&self) -> &ProjectRegistry {
        &self.registry
    }

    pub fn registry_document(&self) -> io::Result<RegistryDocument> {
        self.registry
            .load()
            .map_err(|error| io::Error::other(error.to_string()))
    }

    pub async fn registry_document_async(&self) -> io::Result<RegistryDocument> {
        let registry = self.registry.clone();
        tokio::task::spawn_blocking(move || {
            registry
                .load()
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|error| io::Error::other(format!("registry load task failed: {error}")))?
    }

    /// Ensure a manifest-backed path is present without rewriting or
    /// reclassifying an existing replica. Daemon discovery uses this path;
    /// explicit operator registration continues through `register_path`.
    pub async fn ensure_registered_path(&self, path: &Path) -> io::Result<bool> {
        let registry = self.registry.clone();
        let lookup_path = path.to_path_buf();
        let existing = tokio::task::spawn_blocking(move || {
            registry
                .lookup_path(&lookup_path)
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|error| io::Error::other(format!("registry lookup task failed: {error}")))??;
        if existing.is_some() {
            return Ok(false);
        }
        self.register_path(path).await?;
        Ok(true)
    }

    pub async fn register_path(&self, path: &Path) -> io::Result<RegistrationOutcome> {
        let registry = self.registry.clone();
        let path = path.to_path_buf();
        let outcome = tokio::task::spawn_blocking(move || {
            registry
                .register_existing(path)
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|error| io::Error::other(format!("registry update task failed: {error}")))??;
        self.reload_project(&outcome.registration.project_id)
            .await?;
        Ok(outcome)
    }

    pub async fn reload(&self) -> io::Result<()> {
        const OPEN_CONCURRENCY: usize = 4;
        let started = Instant::now();
        let document = self.registry_document_async().await?;
        let mut next_controls = BTreeMap::new();
        let mut next_errors = BTreeMap::new();
        let projects = document
            .replicas
            .values()
            .map(|replica| replica.project_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|project_id| {
                document
                    .authoritative_replica(&project_id)
                    .map(|(path, _)| (project_id, PathBuf::from(path)))
            })
            .collect::<Vec<_>>();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(OPEN_CONCURRENCY));
        let mut opens = tokio::task::JoinSet::new();
        for (project_id, path) in projects {
            let permit = Arc::clone(&semaphore);
            let data_root = self.data_root.clone();
            opens.spawn(async move {
                let _permit = permit
                    .acquire_owned()
                    .await
                    .map_err(|error| io::Error::other(error.to_string()))?;
                let project_started = Instant::now();
                let opened = match data_root {
                    Some(data_root) => {
                        KbdControlPlane::open_registered_at(&path, &data_root, &project_id).await
                    }
                    None => KbdControlPlane::open_registered(&path, &project_id).await,
                };
                tracing::info!(
                    startup_phase = "project_open",
                    project_id,
                    elapsed_ms = project_started.elapsed().as_millis(),
                    success = opened.is_ok(),
                    "KBD authority startup project open completed"
                );
                if let Err(error) = &opened {
                    tracing::warn!(
                        startup_phase = "project_open",
                        project_id,
                        %error,
                        "KBD authority startup project is unavailable"
                    );
                }
                Ok::<_, io::Error>((project_id, opened))
            });
        }
        while let Some(joined) = opens.join_next().await {
            match joined {
                Ok(Ok((project_id, Ok(control)))) => {
                    next_controls.insert(project_id, Arc::new(control));
                }
                Ok(Ok((project_id, Err(error)))) => {
                    next_errors.insert(project_id, error.to_string());
                }
                Ok(Err(error)) => {
                    next_errors.insert("<startup-task>".into(), error.to_string());
                }
                Err(error) => {
                    next_errors.insert("<startup-task>".into(), error.to_string());
                }
            }
        }
        *self
            .controls
            .write()
            .expect("project control map lock poisoned") = next_controls;
        *self
            .errors
            .write()
            .expect("project error map lock poisoned") = next_errors;
        tracing::info!(
            startup_phase = "registry_reload",
            elapsed_ms = started.elapsed().as_millis(),
            project_count = self.project_ids().len(),
            "KBD registered authorities loaded"
        );
        Ok(())
    }

    async fn reload_project(&self, project_id: &str) -> io::Result<()> {
        let result = async {
            let document = self.registry_document_async().await?;
            let (path, _) = document.authoritative_replica(project_id).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("registered project {project_id} has no authoritative replica"),
                )
            })?;
            let path = PathBuf::from(path);
            match &self.data_root {
                Some(data_root) => {
                    KbdControlPlane::open_registered_at(&path, data_root, project_id).await
                }
                None => KbdControlPlane::open_registered(&path, project_id).await,
            }
        }
        .await;
        match result {
            Ok(control) => {
                self.errors
                    .write()
                    .expect("project error map lock poisoned")
                    .remove(project_id);
                self.controls
                    .write()
                    .expect("project control map lock poisoned")
                    .insert(project_id.to_owned(), Arc::new(control));
                Ok(())
            }
            Err(error) => {
                self.controls
                    .write()
                    .expect("project control map lock poisoned")
                    .remove(project_id);
                self.errors
                    .write()
                    .expect("project error map lock poisoned")
                    .insert(project_id.to_owned(), error.to_string());
                Err(error)
            }
        }
    }

    pub fn control(&self, project_id: &str) -> io::Result<Arc<KbdControlPlane>> {
        if let Some(control) = self
            .controls
            .read()
            .expect("project control map lock poisoned")
            .get(project_id)
            .cloned()
        {
            return Ok(control);
        }
        if let Some(error) = self
            .errors
            .read()
            .expect("project error map lock poisoned")
            .get(project_id)
            .cloned()
        {
            return Err(io::Error::other(error));
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("unknown KBD project {project_id}"),
        ))
    }

    pub fn project_ids(&self) -> Vec<String> {
        let mut project_ids = self
            .controls
            .read()
            .expect("project control map lock poisoned")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        project_ids.extend(
            self.errors
                .read()
                .expect("project error map lock poisoned")
                .keys()
                .cloned(),
        );
        project_ids.into_iter().collect()
    }

    pub fn startup_counts(&self) -> (usize, usize, usize) {
        let opened = self
            .controls
            .read()
            .expect("project control map lock poisoned")
            .len();
        let failed = self
            .errors
            .read()
            .expect("project error map lock poisoned")
            .len();
        (opened + failed, opened, failed)
    }

    pub fn routes(&self) -> io::Result<Vec<RegisteredProjectRoute>> {
        let document = self.registry_document()?;
        let controls = self
            .controls
            .read()
            .expect("project control map lock poisoned");
        let errors = self.errors.read().expect("project error map lock poisoned");
        Ok(document
            .replicas
            .into_iter()
            .map(|(path, replica)| RegisteredProjectRoute {
                project_id: replica.project_id.clone(),
                ready: controls.contains_key(&replica.project_id),
                error: errors.get(&replica.project_id).cloned(),
                path: PathBuf::from(path),
                replica,
            })
            .collect())
    }
}

impl KbdControlPlane {
    pub async fn open(project_root: &Path) -> io::Result<Self> {
        let project_root = project_root.to_path_buf();
        let runtime = Arc::new(
            tokio::task::spawn_blocking(move || {
                Runtime::open_canonical(project_root)
                    .map_err(|error| io::Error::other(error.to_string()))
            })
            .await
            .map_err(|error| io::Error::other(format!("runtime open task failed: {error}")))??,
        );
        Self::from_runtime(runtime).await
    }

    pub async fn open_at(project_root: &Path, data_root: &Path) -> io::Result<Self> {
        let project_root = project_root.to_path_buf();
        let data_root = data_root.to_path_buf();
        let runtime = Arc::new(
            tokio::task::spawn_blocking(move || {
                Runtime::open_canonical_at(project_root, data_root)
                    .map_err(|error| io::Error::other(error.to_string()))
            })
            .await
            .map_err(|error| io::Error::other(format!("runtime open task failed: {error}")))??,
        );
        Self::from_runtime(runtime).await
    }

    /// Open a replica that is already present in the platform registry without
    /// re-running classification. Daemon routing must use this constructor so
    /// recovered/CI/read-only replicas cannot become writable during startup.
    pub async fn open_registered(
        project_root: &Path,
        expected_project_id: &str,
    ) -> io::Result<Self> {
        let project_root = project_root.to_path_buf();
        let expected_project_id = expected_project_id.to_owned();
        let runtime = Arc::new(
            tokio::task::spawn_blocking(move || {
                Runtime::open_registered(&project_root, &expected_project_id)
                    .map_err(|error| io::Error::other(error.to_string()))
            })
            .await
            .map_err(|error| io::Error::other(format!("runtime open task failed: {error}")))??,
        );
        Self::from_runtime(runtime).await
    }

    pub async fn open_registered_at(
        project_root: &Path,
        data_root: &Path,
        expected_project_id: &str,
    ) -> io::Result<Self> {
        let project_root = project_root.to_path_buf();
        let data_root = data_root.to_path_buf();
        let expected_project_id = expected_project_id.to_owned();
        let runtime = Arc::new(
            tokio::task::spawn_blocking(move || {
                Runtime::open_registered_at(&project_root, &data_root, &expected_project_id)
                    .map_err(|error| io::Error::other(error.to_string()))
            })
            .await
            .map_err(|error| io::Error::other(format!("runtime open task failed: {error}")))??,
        );
        Self::from_runtime(runtime).await
    }

    async fn from_runtime(runtime: Arc<Runtime>) -> io::Result<Self> {
        #[cfg(test)]
        {
            let key = runtime.runtime_root().display().to_string();
            *AUTHORITY_OPEN_COUNTS
                .lock()
                .expect("authority open counter lock poisoned")
                .entry(key)
                .or_default() += 1;
        }
        Ok(Self { runtime })
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn status(&self) -> io::Result<KbdStateV2> {
        self.runtime
            .replay()
            .map_err(|error| io::Error::other(error.to_string()))
    }

    /// Replay the journal without parking an async runtime worker.
    pub async fn status_async(&self) -> io::Result<KbdStateV2> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            runtime
                .replay()
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|join_error| io::Error::other(format!("status task failed: {join_error}")))?
    }

    /// Validate journal/Loro authority without invoking local git plumbing.
    /// Used by `/ready`, whose latency must not depend on checkout size.
    pub async fn authority_status_async(&self) -> io::Result<KbdStateV2> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            runtime
                .replay_authority()
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|join_error| io::Error::other(format!("readiness task failed: {join_error}")))?
    }

    pub fn events(&self, since_revision: u64) -> io::Result<Vec<Event>> {
        let events = self
            .runtime
            .events()
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(events
            .into_iter()
            .enumerate()
            .filter(|(index, _)| (*index as u64).saturating_add(1) > since_revision)
            .map(|(_, event)| event)
            .collect())
    }

    pub async fn events_async(&self, since_revision: u64) -> io::Result<Vec<Event>> {
        let control = self.clone();
        tokio::task::spawn_blocking(move || control.events(since_revision))
            .await
            .map_err(|join_error| io::Error::other(format!("events task failed: {join_error}")))?
    }

    pub async fn signed_audit_jsonl(&self) -> io::Result<Vec<u8>> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            runtime
                .signed_audit_jsonl()
                .map(|(bytes, _)| bytes)
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|join_error| io::Error::other(format!("audit export task failed: {join_error}")))?
    }

    pub fn diagnostics(&self) -> io::Result<serde_json::Value> {
        let state = self.status()?;
        let events = self.events(0)?;
        let signature_chain_valid = self.runtime.replay().is_ok();
        let replica_events = self
            .runtime
            .replica_events()
            .map_err(|error| io::Error::other(error.to_string()))?;
        let active_devices = state
            .devices
            .values()
            .filter(|device| device.status == DeviceStatus::Active)
            .count();
        let revoked_devices = state.devices.len().saturating_sub(active_devices);
        let journal_path = self.runtime.events_path();
        let journal_bytes = fs::metadata(&journal_path)
            .map(|meta| meta.len())
            .unwrap_or(0);
        let projection_path = self
            .runtime
            .project_root()
            .join(".kbd-orchestrator/current-waypoint.json");
        let projection_revision = fs::read(&projection_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .and_then(|value| {
                value
                    .get("sourceRevision")
                    .and_then(|revision| revision.as_u64())
            });
        let document = self
            .runtime
            .project_document()
            .map(|document| document.status())
            .transpose()
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(serde_json::json!({
            "schemaVersion": "2",
            "singleWriter": {
                "lockPath": self.runtime.journal_lock_path(),
                "available": true,
                "authority": "exclusive-journal-lock"
            },
            "journal": {
                "path": journal_path,
                "bytes": journal_bytes,
                "replicaId": self.runtime.replica_id(),
                "eventCount": replica_events.len(),
                "lastLamport": replica_events.last().map(|event| event.lamport).unwrap_or(0),
                "ingested": replica_events.iter().all(|event| events.iter().any(|committed| committed.event_id == event.event_id))
            },
            "document": {
                "status": document,
                "eventCount": events.len(),
                "derivedRevision": state.revision,
                "frontier": state.frontier,
                "conflictCount": state.conflicts.len()
            },
            "runtime": {
                "projectId": state.project_id,
                "revision": state.revision,
                "frontier": state.frontier,
                "lifecycle": state.lifecycle,
                "planRevision": state.plan_revision
            },
            "projection": {
                "revision": projection_revision,
                "path": projection_path,
                "matchesRuntime": projection_revision == Some(state.revision)
            },
            "integrity": {
                "signatureChainValid": signature_chain_valid,
                "eventCount": events.len()
            },
            "trust": {
                "activeDevices": active_devices,
                "revokedDevices": revoked_devices
            }
        }))
    }

    pub async fn diagnostics_async(&self) -> io::Result<serde_json::Value> {
        let control = self.clone();
        tokio::task::spawn_blocking(move || control.diagnostics())
            .await
            .map_err(|join_error| {
                io::Error::other(format!("diagnostics task failed: {join_error}"))
            })?
    }

    pub async fn export_project_updates(&self) -> io::Result<Vec<u8>> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            runtime
                .export_project_updates()
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|join_error| io::Error::other(format!("export task failed: {join_error}")))?
    }

    pub async fn import_project_updates(
        &self,
        updates: Vec<u8>,
    ) -> io::Result<(usize, KbdStateV2)> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            runtime
                .import_project_updates(&updates)
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await
        .map_err(|join_error| io::Error::other(format!("import task failed: {join_error}")))?
    }

    pub async fn submit(&self, envelope: CommandEnvelope) -> io::Result<CommittedCommand> {
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
            let projection_command = envelope.command.clone();
            let result = runtime
                .execute_command(envelope)
                .map_err(|error| io::Error::other(error.to_string()))?;
            if let Some(error) = result.apply_error.clone() {
                return Err(io::Error::other(error));
            }
            let timestamp = runtime
                .events()
                .map_err(|error| io::Error::other(error.to_string()))?
                .get(result.committed_revision.saturating_sub(1) as usize)
                .map(|event| event.timestamp)
                .ok_or_else(|| io::Error::other("committed event is missing from journal"))?;
            let projection_error = runtime
                .write_compatibility_projections_from_state_for_command(
                    &result.state,
                    timestamp,
                    &projection_command,
                )
                .err()
                .map(|error| error.to_string());
            Ok(CommittedCommand {
                result,
                projection_error,
            })
        })
        .await
        .map_err(|join_error| io::Error::other(format!("command task failed: {join_error}")))?
    }

    pub async fn submit_signed(
        &self,
        signed: SignedCommandEnvelope,
    ) -> io::Result<CommittedCommand> {
        if signed.command.schema_version != "2" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "remote commands require schemaVersion 2",
            ));
        }
        let state = self.status_async().await?;
        signed
            .verify(&state)
            .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error.to_string()))?;
        self.submit(signed.command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kbd_runtime::{registry::ReplicaKind, Actor, ActorKind, CommandKind};
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn standalone_commands_commit_to_the_fsynced_journal() {
        let project = tempdir().unwrap();
        let runtime = Arc::new(Runtime::open(project.path()));
        let project_id = runtime.project_manifest(true).unwrap().unwrap().project_id;
        let initialized = runtime
            .initialize(project_id, "run-a", Actor::operator("operator-a", "test"))
            .unwrap();
        let control = KbdControlPlane::from_runtime(runtime.clone())
            .await
            .unwrap();
        let actor = Actor {
            kind: ActorKind::Harness,
            id: "codex".into(),
            device: "device-codex".into(),
            harness: "codex".into(),
            session: "session-codex".into(),
        };
        let command_id = Uuid::new_v4().to_string();
        let envelope = CommandEnvelope {
            schema_version: "1".into(),
            project_id: initialized.project_id.clone(),
            run_id: initialized.run_id.clone(),
            command_id: command_id.clone(),
            frontier: None,
            expected_revision: initialized.revision,
            actor,
            command: CommandKind::LifecycleTransition {
                to: kbd_runtime::LifecycleState::Running,
                reason: "test journal commit".into(),
            },
        };
        let committed = control.submit(envelope.clone()).await.unwrap();
        assert_eq!(committed.result.committed_revision, 2);
        assert!(!committed.result.duplicate);
        assert_eq!(runtime.events().unwrap().len(), 2);
        assert_eq!(control.events(1).unwrap().len(), 1);

        let duplicate = control.submit(envelope).await.unwrap();
        assert!(duplicate.result.duplicate);
        assert_eq!(duplicate.result.committed_revision, 2);
        assert_eq!(control.status().unwrap().revision, 2);
        let waypoint: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(
                project
                    .path()
                    .join(".kbd-orchestrator/current-waypoint.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(waypoint["sourceRevision"], 2);
    }

    #[tokio::test]
    async fn successor_run_commit_refreshes_daemon_projections() {
        let project = tempdir().unwrap();
        let runtime = Arc::new(Runtime::open(project.path()));
        let project_id = runtime.project_manifest(true).unwrap().unwrap().project_id;
        let operator = Actor::operator("operator-a", "test");
        let initialized = runtime
            .initialize(project_id.clone(), "run-a", operator.clone())
            .unwrap();
        let cancelled = runtime
            .append(
                operator.clone(),
                initialized.revision,
                kbd_runtime::EventKind::LifecycleTransition {
                    from: kbd_runtime::LifecycleState::Ready,
                    to: kbd_runtime::LifecycleState::Cancelled,
                    reason: "old run complete".into(),
                },
            )
            .unwrap();
        let control = KbdControlPlane::from_runtime(runtime.clone())
            .await
            .unwrap();
        let committed = control
            .submit(CommandEnvelope {
                schema_version: "2".into(),
                project_id,
                run_id: "run-a".into(),
                command_id: "start-run-b".into(),
                frontier: Some(cancelled.frontier),
                expected_revision: cancelled.revision,
                actor: operator,
                command: CommandKind::RunStart {
                    run_id: "run-b".into(),
                    reason: "new work".into(),
                    exact_next_work: Some("/kbd-new-phase".into()),
                },
            })
            .await
            .unwrap();

        assert!(committed.projection_error.is_none());
        assert_eq!(committed.result.state.run_id, "run-b");
        assert!(committed.result.state.phases.is_empty());
        let waypoint: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(
                project
                    .path()
                    .join(".kbd-orchestrator/current-waypoint.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(waypoint["runId"], "run-b");
        assert_eq!(waypoint["status"], "ready");
        assert_eq!(waypoint["implementationCompleted"], 0);
        assert_eq!(waypoint["implementationTotal"], 0);
    }

    #[tokio::test]
    async fn daemon_router_preserves_recovered_read_only_classification() {
        let data_root = tempdir().unwrap();
        let project_id = Uuid::new_v4().to_string();
        std::fs::create_dir_all(
            data_root
                .path()
                .join("prometheus/kbd/projects")
                .join(&project_id),
        )
        .unwrap();
        let registry = ProjectRegistry::open_at(data_root.path());
        let recovered = registry.register_recovered(&project_id).unwrap();
        assert_eq!(recovered.registration.kind, ReplicaKind::Recovered);
        assert!(recovered.registration.read_only);
        let registry_before = std::fs::read(registry.registry_path()).unwrap();

        let router = KbdProjectRouter::open_registered_at(data_root.path())
            .await
            .unwrap();
        assert!(!router
            .ensure_registered_path(std::path::Path::new(&recovered.path))
            .await
            .unwrap());
        assert_eq!(
            std::fs::read(registry.registry_path()).unwrap(),
            registry_before,
            "daemon discovery must not rewrite an existing registration"
        );
        let after = router
            .registry()
            .lookup_path(&recovered.path)
            .unwrap()
            .unwrap();
        assert_eq!(after.kind, ReplicaKind::Recovered);
        assert!(after.read_only);
        let authority = router
            .control(&project_id)
            .unwrap()
            .authority_status_async()
            .await
            .unwrap();
        assert!(authority.replica_view.is_none());
    }

    #[tokio::test]
    async fn failed_registered_project_does_not_hide_healthy_authorities() {
        let fixture = tempdir().unwrap();
        let data_root = fixture.path().join("data");
        let healthy_path = fixture.path().join("healthy");
        let stale_path = fixture.path().join("stale");
        std::fs::create_dir_all(&healthy_path).unwrap();
        std::fs::create_dir_all(&stale_path).unwrap();
        let healthy = Runtime::open_canonical_at(&healthy_path, &data_root).unwrap();
        let healthy_project_id = healthy.project_manifest(false).unwrap().unwrap().project_id;
        let stale = Runtime::open_canonical_at(&stale_path, &data_root).unwrap();
        let stale_project_id = stale.project_manifest(false).unwrap().unwrap().project_id;
        drop(healthy);
        drop(stale);
        std::fs::remove_dir_all(&stale_path).unwrap();

        let router = KbdProjectRouter::open_registered_at(&data_root)
            .await
            .unwrap();

        assert_eq!(router.startup_counts(), (2, 1, 1));
        assert!(router.control(&healthy_project_id).is_ok());
        assert!(router.control(&stale_project_id).is_err());
        let routes = router.routes().unwrap();
        assert!(routes
            .iter()
            .any(|route| route.project_id == healthy_project_id && route.ready));
        assert!(routes.iter().any(|route| {
            route.project_id == stale_project_id && !route.ready && route.error.is_some()
        }));
    }

    #[tokio::test]
    async fn eighteen_authorities_open_once_and_existing_discovery_does_not_reload() {
        let fixture = tempdir().unwrap();
        let data_root = fixture.path().join("data");
        let mut paths = Vec::new();
        for index in 0..18 {
            let path = fixture.path().join(format!("project-{index:02}"));
            std::fs::create_dir_all(&path).unwrap();
            Runtime::open_canonical_at(&path, &data_root).unwrap();
            paths.push(path);
        }
        AUTHORITY_OPEN_COUNTS
            .lock()
            .expect("authority open counter lock poisoned")
            .clear();

        let router = KbdProjectRouter::open_registered_at(&data_root)
            .await
            .unwrap();

        assert_eq!(router.project_ids().len(), 18);
        let data_prefix = data_root.display().to_string();
        let counts_before = AUTHORITY_OPEN_COUNTS
            .lock()
            .expect("authority open counter lock poisoned")
            .iter()
            .filter(|(path, _)| path.starts_with(&data_prefix))
            .map(|(path, count)| (path.clone(), *count))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(counts_before.len(), 18);
        assert!(counts_before.values().all(|count| *count == 1));
        let registry_before = std::fs::read(router.registry().registry_path()).unwrap();

        assert!(!router.ensure_registered_path(&paths[0]).await.unwrap());

        let counts_after = AUTHORITY_OPEN_COUNTS
            .lock()
            .expect("authority open counter lock poisoned")
            .iter()
            .filter(|(path, _)| path.starts_with(&data_prefix))
            .map(|(path, count)| (path.clone(), *count))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            counts_after, counts_before,
            "an existing manifest-backed path must not trigger a second authority reload"
        );
        assert_eq!(
            std::fs::read(router.registry().registry_path()).unwrap(),
            registry_before,
            "existing daemon discovery must remain byte-stable"
        );
    }
}
