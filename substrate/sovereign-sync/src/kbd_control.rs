//! Authoritative command facade shared by REST, MCP, and the daemon.
//!
//! Callers submit the same versioned `CommandEnvelope`. During stabilization,
//! commands are committed by one journal writer protected by an exclusive
//! flock. Multi-voter configurations are rejected explicitly.

use std::{
    collections::BTreeSet,
    fs, io,
    path::Path,
    sync::{Arc, RwLock},
};

use kbd_runtime::{
    replay_events, CommandEnvelope, CommandResult, DeviceStatus, Event, KbdStateV2, Runtime,
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
/// flock and fsynced on append. The compatibility policy permits exactly one
/// writer until the project Loro document becomes the converged authority.
pub struct KbdControlPlane {
    runtime: Arc<Runtime>,
    quorum: Arc<QuorumPolicy>,
    available_voters: Arc<RwLock<BTreeSet<u64>>>,
}

impl KbdControlPlane {
    pub async fn open(project_root: &Path, quorum: QuorumPolicy) -> io::Result<Self> {
        let runtime = Arc::new(
            Runtime::open_canonical(project_root)
                .map_err(|error| io::Error::other(error.to_string()))?,
        );
        Self::from_runtime(runtime, quorum).await
    }

    pub async fn open_at(
        project_root: &Path,
        data_root: &Path,
        quorum: QuorumPolicy,
    ) -> io::Result<Self> {
        let runtime = Arc::new(
            Runtime::open_canonical_at(project_root, data_root)
                .map_err(|error| io::Error::other(error.to_string()))?,
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
            recovery_runtime
                .recover_journal_tail()
                .map_err(|error| io::Error::other(error.to_string()))
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
            .filter(|event| event.revision > since_revision)
            .collect())
    }

    pub async fn events_async(&self, since_revision: u64) -> io::Result<Vec<Event>> {
        let control = self.clone();
        tokio::task::spawn_blocking(move || control.events(since_revision))
            .await
            .map_err(|join_error| io::Error::other(format!("events task failed: {join_error}")))?
    }

    pub fn diagnostics(&self) -> io::Result<serde_json::Value> {
        let state = self.status()?;
        let events = self.events(0)?;
        let signature_chain_valid = replay_events(&events).is_ok();
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
        Ok(serde_json::json!({
            "schemaVersion": "1",
            "quorum": self.quorum_status(),
            "singleWriter": {
                "nodeId": self.quorum.node_id(),
                "lockPath": self.runtime.runtime_root().join("runtime.lock"),
                "available": self.quorum_status().writable
            },
            "journal": {
                "path": journal_path,
                "bytes": journal_bytes,
                "eventCount": events.len(),
                "lastRevision": events.last().map(|event| event.revision).unwrap_or(0),
                "matchesRuntime": events.last().map(|event| event.revision).unwrap_or(0) == state.revision
            },
            "runtime": {
                "projectId": state.project_id,
                "revision": state.revision,
                "lifecycle": state.lifecycle,
                "planRevision": state.plan_revision,
                "lease": state.lease,
                "fencingToken": state.last_fencing_token
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
            expected_revision: initialized.revision,
            actor,
            lease_id: None,
            fencing_token: None,
            command: CommandKind::Claim {
                scope: "project/phase".into(),
                force: false,
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
