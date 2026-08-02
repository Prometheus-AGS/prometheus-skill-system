//! Authoritative command facade shared by REST, MCP, and the daemon.
//!
//! Callers submit the same versioned `CommandEnvelope`. During stabilization,
//! commands are committed by one journal writer protected by an exclusive
//! flock. Multi-voter configurations are rejected explicitly.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use kbd_runtime::{
    registry::{ProjectRegistry, RegistrationOutcome, RegistryDocument, ReplicaRegistration},
    CommandEnvelope, CommandResult, DeviceStatus, Event, KbdStateV2, Runtime,
    SignedCommandEnvelope,
};
use serde::Serialize;

use crate::kbd_single_writer::{QuorumPolicy, QuorumStatus};

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
/// Loro document. The compatibility policy permits exactly one local writer.
pub struct KbdControlPlane {
    runtime: Arc<Runtime>,
    quorum: Arc<QuorumPolicy>,
    available_voters: Arc<RwLock<BTreeSet<u64>>>,
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
    policy: QuorumPolicy,
    controls: Arc<RwLock<BTreeMap<String, Arc<KbdControlPlane>>>>,
    errors: Arc<RwLock<BTreeMap<String, String>>>,
}

impl KbdProjectRouter {
    pub async fn open_registered(policy: QuorumPolicy) -> io::Result<Self> {
        let router = Self {
            registry: ProjectRegistry::open(),
            data_root: None,
            policy,
            controls: Arc::new(RwLock::new(BTreeMap::new())),
            errors: Arc::new(RwLock::new(BTreeMap::new())),
        };
        router.reload().await?;
        Ok(router)
    }

    pub async fn open_registered_at(data_root: &Path, policy: QuorumPolicy) -> io::Result<Self> {
        let router = Self {
            registry: ProjectRegistry::open_at(data_root),
            data_root: Some(data_root.to_path_buf()),
            policy,
            controls: Arc::new(RwLock::new(BTreeMap::new())),
            errors: Arc::new(RwLock::new(BTreeMap::new())),
        };
        router.reload().await?;
        Ok(router)
    }

    pub async fn open_with_project(project_root: &Path, policy: QuorumPolicy) -> io::Result<Self> {
        let registry = ProjectRegistry::open();
        registry
            .register_existing(project_root)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::open_registered(policy).await
    }

    pub async fn open_with_project_at(
        project_root: &Path,
        data_root: &Path,
        policy: QuorumPolicy,
    ) -> io::Result<Self> {
        // Establishing a canonical runtime is the explicit project-initialization
        // path. Registry registration itself still consumes only the manifest.
        Runtime::open_canonical_at(project_root, data_root)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Self::open_registered_at(data_root, policy).await
    }

    pub fn registry(&self) -> &ProjectRegistry {
        &self.registry
    }

    pub fn registry_document(&self) -> io::Result<RegistryDocument> {
        self.registry
            .load()
            .map_err(|error| io::Error::other(error.to_string()))
    }

    pub async fn register_path(&self, path: &Path) -> io::Result<RegistrationOutcome> {
        let outcome = self
            .registry
            .register_existing(path)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.reload().await?;
        Ok(outcome)
    }

    pub async fn reload(&self) -> io::Result<()> {
        let document = self.registry_document()?;
        let mut next_controls = BTreeMap::new();
        let mut next_errors = BTreeMap::new();
        let project_ids = document
            .replicas
            .values()
            .map(|replica| replica.project_id.clone())
            .collect::<BTreeSet<_>>();
        for project_id in project_ids {
            let Some((path, _)) = document.authoritative_replica(&project_id) else {
                continue;
            };
            let path = PathBuf::from(path);
            let opened = match &self.data_root {
                Some(data_root) => {
                    KbdControlPlane::open_at(&path, data_root, self.policy.clone()).await
                }
                None => KbdControlPlane::open(&path, self.policy.clone()).await,
            };
            match opened {
                Ok(control) => {
                    next_controls.insert(project_id, Arc::new(control));
                }
                Err(error) => {
                    next_errors.insert(project_id, error.to_string());
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
        Ok(())
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
    pub async fn open(project_root: &Path, quorum: QuorumPolicy) -> io::Result<Self> {
        let project_root = project_root.to_path_buf();
        let runtime = Arc::new(
            tokio::task::spawn_blocking(move || {
                Runtime::open_canonical(project_root)
                    .map_err(|error| io::Error::other(error.to_string()))
            })
            .await
            .map_err(|error| io::Error::other(format!("runtime open task failed: {error}")))??,
        );
        Self::from_runtime(runtime, quorum).await
    }

    pub async fn open_at(
        project_root: &Path,
        data_root: &Path,
        quorum: QuorumPolicy,
    ) -> io::Result<Self> {
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
        Self::from_runtime(runtime, quorum).await
    }

    async fn from_runtime(runtime: Arc<Runtime>, quorum: QuorumPolicy) -> io::Result<Self> {
        if quorum.voters().len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "multi-voter KBD configuration is unsupported; configure exactly one journal writer",
            ));
        }
        let recovery_runtime = Arc::clone(&runtime);
        if let Some(archive) = tokio::task::spawn_blocking(move || {
            let archive = recovery_runtime
                .recover_journal_tail()
                .map_err(|error| io::Error::other(error.to_string()))?;
            recovery_runtime
                .reconcile_project_document()
                .map_err(|error| io::Error::other(error.to_string()))?;
            Ok::<_, io::Error>(archive)
        })
        .await
        .map_err(|join_error| {
            io::Error::other(format!("journal recovery task failed: {join_error}"))
        })?? {
            tracing::warn!(
                archive = %archive.display(),
                "archived an interrupted KBD journal tail before recovery"
            );
        }
        Ok(Self {
            runtime,
            available_voters: Arc::new(RwLock::new(BTreeSet::from([quorum.node_id()]))),
            quorum: Arc::new(quorum),
        })
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

    pub fn quorum_status(&self) -> QuorumStatus {
        let available = self
            .available_voters
            .read()
            .expect("available voter lock poisoned")
            .clone();
        self.quorum.status(available)
    }

    pub fn set_voter_available(&self, node_id: u64, available: bool) {
        let mut voters = self
            .available_voters
            .write()
            .expect("available voter lock poisoned");
        if available {
            voters.insert(node_id);
        } else {
            voters.remove(&node_id);
        }
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
            "schemaVersion": "1",
            "quorum": self.quorum_status(),
            "singleWriter": {
                "nodeId": self.quorum.node_id(),
                "lockPath": self.runtime.journal_lock_path(),
                "available": self.quorum_status().writable
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
        let quorum = self.quorum_status();
        if !quorum.writable {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, quorum.reason));
        }
        let runtime = Arc::clone(&self.runtime);
        tokio::task::spawn_blocking(move || {
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
                .write_compatibility_projections_from_state(&result.state, timestamp)
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
    use kbd_runtime::{Actor, ActorKind, CommandKind};
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
        let control =
            KbdControlPlane::from_runtime(runtime.clone(), QuorumPolicy::new(1, [1]).unwrap())
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
}
