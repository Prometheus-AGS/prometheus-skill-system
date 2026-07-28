//! Authenticated embedded transport for OpenRaft.
//!
//! This transport is used for voters embedded in the same Sovereign Sync
//! process (including a local witness). Cross-process transports implement the
//! same OpenRaft traits over authenticated iroh streams; authoritative KBD
//! events are never sent through Loro or gossip.

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

use openraft::{
    error::{InstallSnapshotError, RPCError, RaftError, RemoteError, Unreachable},
    network::{RPCOption, RaftNetwork, RaftNetworkFactory},
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    Raft,
};
use tokio::sync::RwLock;

use crate::kbd_raft::{KbdNodeId, KbdRaftConfig, KbdRaftNode};

#[derive(Clone)]
struct RegisteredVoter {
    node: KbdRaftNode,
    raft: Raft<KbdRaftConfig>,
    allowed_signer_keys: BTreeSet<String>,
}

type Registry = Arc<RwLock<BTreeMap<KbdNodeId, RegisteredVoter>>>;

/// Explicitly authenticated in-process Raft network. Both endpoint and signer
/// key must match the committed/configured membership record.
#[derive(Clone)]
pub struct EmbeddedRaftNetworkFactory {
    source_node: KbdRaftNode,
    registry: Registry,
}

impl EmbeddedRaftNetworkFactory {
    pub fn new(source_node: KbdRaftNode) -> Self {
        Self {
            source_node,
            registry: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn shared_for(&self, source_node: KbdRaftNode) -> Self {
        Self {
            source_node,
            registry: self.registry.clone(),
        }
    }

    pub async fn register(
        &self,
        node_id: KbdNodeId,
        node: KbdRaftNode,
        raft: Raft<KbdRaftConfig>,
        allowed_signer_keys: impl IntoIterator<Item = String>,
    ) {
        self.registry.write().await.insert(
            node_id,
            RegisteredVoter {
                node,
                raft,
                allowed_signer_keys: allowed_signer_keys.into_iter().collect(),
            },
        );
    }
}

pub struct EmbeddedRaftNetwork {
    source_node: KbdRaftNode,
    target: KbdNodeId,
    expected_target: KbdRaftNode,
    registry: Registry,
}

impl EmbeddedRaftNetwork {
    async fn target(&self) -> io::Result<Raft<KbdRaftConfig>> {
        let registry = self.registry.read().await;
        let target = registry.get(&self.target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                format!("Raft voter {} is not connected", self.target),
            )
        })?;
        if target.node != self.expected_target {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "target endpoint or signer key does not match membership",
            ));
        }
        if !target
            .allowed_signer_keys
            .contains(&self.source_node.signer_key_id)
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "source signer key is not in the target voter allowlist",
            ));
        }
        Ok(target.raft.clone())
    }

    fn unreachable<E>(&self, error: io::Error) -> RPCError<KbdNodeId, KbdRaftNode, E>
    where
        E: std::error::Error,
    {
        RPCError::Unreachable(Unreachable::new(&error))
    }
}

impl RaftNetworkFactory<KbdRaftConfig> for EmbeddedRaftNetworkFactory {
    type Network = EmbeddedRaftNetwork;

    async fn new_client(&mut self, target: KbdNodeId, node: &KbdRaftNode) -> Self::Network {
        EmbeddedRaftNetwork {
            source_node: self.source_node.clone(),
            target,
            expected_target: node.clone(),
            registry: self.registry.clone(),
        }
    }
}

impl RaftNetwork<KbdRaftConfig> for EmbeddedRaftNetwork {
    async fn append_entries(
        &mut self,
        rpc: AppendEntriesRequest<KbdRaftConfig>,
        _option: RPCOption,
    ) -> Result<
        AppendEntriesResponse<KbdNodeId>,
        RPCError<KbdNodeId, KbdRaftNode, RaftError<KbdNodeId>>,
    > {
        let target = self
            .target()
            .await
            .map_err(|error| self.unreachable(error))?;
        target
            .append_entries(rpc)
            .await
            .map_err(|error| RPCError::RemoteError(RemoteError::new(self.target, error)))
    }

    async fn install_snapshot(
        &mut self,
        rpc: InstallSnapshotRequest<KbdRaftConfig>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<KbdNodeId>,
        RPCError<KbdNodeId, KbdRaftNode, RaftError<KbdNodeId, InstallSnapshotError>>,
    > {
        let target = self
            .target()
            .await
            .map_err(|error| self.unreachable(error))?;
        target
            .install_snapshot(rpc)
            .await
            .map_err(|error| RPCError::RemoteError(RemoteError::new(self.target, error)))
    }

    async fn vote(
        &mut self,
        rpc: VoteRequest<KbdNodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<KbdNodeId>, RPCError<KbdNodeId, KbdRaftNode, RaftError<KbdNodeId>>>
    {
        let target = self
            .target()
            .await
            .map_err(|error| self.unreachable(error))?;
        target
            .vote(rpc)
            .await
            .map_err(|error| RPCError::RemoteError(RemoteError::new(self.target, error)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kbd_raft::RedbRaftStore;
    use kbd_runtime::{Actor, Runtime};
    use openraft::Config;
    use std::{path::Path, time::Duration};
    use tempfile::tempdir;

    async fn voter(
        id: u64,
        node: &KbdRaftNode,
        factory: EmbeddedRaftNetworkFactory,
        root: &Path,
    ) -> (Raft<KbdRaftConfig>, Arc<RedbRaftStore>) {
        let store = RedbRaftStore::open(&root.join(format!("node-{id}.redb"))).unwrap();
        let (log, state_machine) = store.into_openraft_stores();
        let config = Config {
            cluster_name: "kbd-embedded-test".into(),
            election_timeout_min: 150,
            election_timeout_max: 300,
            heartbeat_interval: 50,
            ..Default::default()
        }
        .validate()
        .unwrap();
        let raft = Raft::new(id, Arc::new(config), factory, log, state_machine)
            .await
            .unwrap();
        assert_eq!(node.endpoint, format!("embedded://{id}"));
        (raft, store)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn three_voters_commit_one_order_and_require_a_quorum() {
        let directory = tempdir().unwrap();
        let nodes = (1..=3)
            .map(|id| {
                (
                    id,
                    KbdRaftNode {
                        endpoint: format!("embedded://{id}"),
                        signer_key_id: format!("key-{id}"),
                        witness: id == 3,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let base = EmbeddedRaftNetworkFactory::new(nodes[&1].clone());
        let (raft1, store1) = voter(
            1,
            &nodes[&1],
            base.shared_for(nodes[&1].clone()),
            directory.path(),
        )
        .await;
        let (raft2, store2) = voter(
            2,
            &nodes[&2],
            base.shared_for(nodes[&2].clone()),
            directory.path(),
        )
        .await;
        let (raft3, store3) = voter(
            3,
            &nodes[&3],
            base.shared_for(nodes[&3].clone()),
            directory.path(),
        )
        .await;
        let allowed = nodes
            .values()
            .map(|node| node.signer_key_id.clone())
            .collect::<Vec<_>>();
        base.register(1, nodes[&1].clone(), raft1.clone(), allowed.clone())
            .await;
        base.register(2, nodes[&2].clone(), raft2.clone(), allowed.clone())
            .await;
        base.register(3, nodes[&3].clone(), raft3.clone(), allowed)
            .await;

        raft1.initialize(nodes.clone()).await.unwrap();
        raft1
            .wait(Some(Duration::from_secs(5)))
            .current_leader(1, "initial KBD leader")
            .await
            .unwrap();

        let event_source = Runtime::open(directory.path().join("events"));
        event_source
            .initialize("project-a", "run-a", Actor::operator("operator-a", "test"))
            .unwrap();
        let event = event_source.events().unwrap().remove(0);
        let response = raft1.client_write(event).await.unwrap();
        assert_eq!(response.data.committed_revision, 1);
        for (raft, store) in [(&raft1, &store1), (&raft2, &store2), (&raft3, &store3)] {
            raft.wait(Some(Duration::from_secs(5)))
                .applied_index(Some(response.log_id.index), "replicated KBD event")
                .await
                .unwrap();
            assert_eq!(store.runtime_state().unwrap().revision, 1);
        }

        // Isolate the leader from both followers. A second command must not be
        // acknowledged, even though the former leader is still running.
        base.registry.write().await.remove(&2);
        base.registry.write().await.remove(&3);
        let claimed = event_source
            .claim(
                Actor::operator("operator-a", "test"),
                1,
                "project/phase",
                false,
            )
            .unwrap();
        let second = event_source.events().unwrap().remove(1);
        assert!(
            tokio::time::timeout(Duration::from_millis(500), raft1.client_write(second))
                .await
                .is_err(),
            "a partitioned leader must not acknowledge a write without quorum"
        );
        assert_eq!(claimed.revision, 2);
        assert_eq!(store1.runtime_state().unwrap().revision, 1);

        raft1.shutdown().await.unwrap();
        raft2.shutdown().await.unwrap();
        raft3.shutdown().await.unwrap();
    }
}
