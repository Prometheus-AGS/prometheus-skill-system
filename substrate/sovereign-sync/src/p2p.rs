use anyhow::Result;
use bytes::Bytes;
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId};
use iroh_gossip::{
    api::{Event, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::StreamExt;
use tracing::{debug, info, warn};

#[cfg(test)]
use iroh::address_lookup::MemoryLookup;

use crate::config::PeersConfig;
use crate::error::SyncError;

/// Loro authority update bundles can exceed iroh-gossip's 4 KiB default even
/// for a small signed project. Keep a hard upper bound so malformed local
/// callers cannot enqueue unbounded frames; larger histories must use a
/// future incremental frontier exchange rather than silently truncating.
const MAX_GOSSIP_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// FSM states for the P2P node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeState {
    Disconnected,
    Bootstrapping,
    Connected,
    Syncing,
    Idle,
}

/// A message received from a peer over gossip.
#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub from: EndpointId,
    pub payload: Bytes,
}

pub struct P2PNode {
    state: Arc<RwLock<NodeState>>,
    endpoint: Endpoint,
    router: Router,
    gossip: Gossip,
    topic: TopicId,
    message_tx: mpsc::Sender<PeerMessage>,
    /// Sender half of the gossip topic — held for lifetime of node.
    sender: Arc<tokio::sync::Mutex<Option<GossipSender>>>,
    /// Current gossip neighbors, tracked from `Event::NeighborUp`/`Down`.
    /// iroh-gossip has no "list current neighbors" getter, so this is the
    /// only way to answer `GET /api/v1/sync/peers` with real data.
    neighbors: Arc<RwLock<HashSet<EndpointId>>>,
}

impl P2PNode {
    /// Create a new P2P node.
    ///
    /// `operator_id` derives the gossip topic: all nodes sharing the same
    /// operator_id subscribe to the same gossip channel and form a sync group.
    pub async fn new(
        operator_id: &[u8; 32],
        _peers_config: &PeersConfig,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let topic = Self::derive_topic(operator_id);

        // N0 preset: pkarr DNS discovery + relay mode, with bundled crypto provider.
        let endpoint = Endpoint::builder(presets::N0).bind().await?;

        Ok(Self::from_endpoint(topic, endpoint))
    }

    #[cfg(test)]
    pub(crate) async fn new_with_memory_lookup(
        operator_id: &[u8; 32],
        address_lookup: MemoryLookup,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let topic = Self::derive_topic(operator_id);
        let endpoint = Endpoint::builder(presets::Minimal)
            .bind_addr(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))?
            .address_lookup(address_lookup.clone())
            .bind()
            .await?;
        address_lookup.add_endpoint_info(endpoint.addr());
        Ok(Self::from_endpoint(topic, endpoint))
    }

    fn from_endpoint(topic: TopicId, endpoint: Endpoint) -> (Self, mpsc::Receiver<PeerMessage>) {
        info!("P2P endpoint started — node_id={}", endpoint.id());

        // spawn() is synchronous — no .await needed.
        let gossip = Gossip::builder()
            .max_message_size(MAX_GOSSIP_MESSAGE_SIZE)
            .spawn(endpoint.clone());
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let (message_tx, message_rx) = mpsc::channel(256);

        (
            Self {
                state: Arc::new(RwLock::new(NodeState::Disconnected)),
                endpoint,
                router,
                gossip,
                topic,
                message_tx,
                sender: Arc::new(tokio::sync::Mutex::new(None)),
                neighbors: Arc::new(RwLock::new(HashSet::new())),
            },
            message_rx,
        )
    }

    /// Derive a deterministic TopicId from the operator_id.
    /// All nodes with the same operator_id join the same gossip topic.
    pub fn derive_topic(operator_id: &[u8; 32]) -> TopicId {
        let mut input = operator_id.to_vec();
        input.extend_from_slice(b"sovereign-sync-v1");
        let hash = *blake3::hash(&input).as_bytes();
        TopicId::from_bytes(hash)
    }

    /// Bootstrap, join the gossip topic, and enter Idle state.
    /// After this returns, `broadcast()` and `add_peers()` are available.
    pub async fn start(&self, bootstrap_peers: Vec<EndpointId>) -> Result<()> {
        {
            let mut state = self.state.write().await;
            *state = NodeState::Bootstrapping;
        }
        info!(
            "P2P bootstrapping with {} known peers",
            bootstrap_peers.len()
        );

        let topic = self
            .gossip
            .subscribe(self.topic, bootstrap_peers)
            .await
            .map_err(|e| anyhow::anyhow!("gossip subscription failed: {e}"))?;

        let (sender, mut receiver) = topic.split();

        // Store sender so broadcast() can use it without re-subscribing.
        {
            let mut guard = self.sender.lock().await;
            *guard = Some(sender);
        }

        {
            let mut state = self.state.write().await;
            *state = NodeState::Connected;
        }
        info!(
            "P2P connected — topic={}",
            blake3::Hash::from_bytes(*self.topic.as_bytes())
        );

        // Spawn receiver loop.
        let state_clone = self.state.clone();
        let tx = self.message_tx.clone();
        let neighbors_clone = self.neighbors.clone();

        tokio::spawn(async move {
            while let Some(event_result) = receiver.next().await {
                match event_result {
                    Ok(event) => match event {
                        Event::Received(msg) => {
                            let peer_msg = PeerMessage {
                                from: msg.delivered_from,
                                payload: msg.content,
                            };
                            if tx.send(peer_msg).await.is_err() {
                                break;
                            }
                        }
                        Event::NeighborUp(peer) => {
                            debug!("Neighbor up: {}", peer);
                            neighbors_clone.write().await.insert(peer);
                            let mut s = state_clone.write().await;
                            if *s == NodeState::Connected {
                                *s = NodeState::Idle;
                            }
                        }
                        Event::NeighborDown(peer) => {
                            debug!("Neighbor down: {}", peer);
                            neighbors_clone.write().await.remove(&peer);
                        }
                        Event::Lagged => {
                            warn!("Gossip receiver lagged — some messages dropped");
                        }
                    },
                    Err(e) => {
                        warn!("Gossip receiver error: {e}");
                        break;
                    }
                }
            }
        });

        // If no neighbors have connected yet, transition directly to Idle.
        {
            let mut state = self.state.write().await;
            if *state == NodeState::Connected {
                *state = NodeState::Idle;
            }
        }

        Ok(())
    }

    /// Broadcast a delta to all subscribed peers.
    ///
    /// Privacy gate is enforced upstream in `crdt.rs` — bytes reaching here
    /// have already passed the `PrivacyClass::LocalOnly` check.
    pub async fn broadcast(&self, payload: Bytes) -> Result<(), SyncError> {
        if payload.len() > MAX_GOSSIP_MESSAGE_SIZE {
            return Err(SyncError::Network(format!(
                "gossip payload is {} bytes; maximum is {MAX_GOSSIP_MESSAGE_SIZE}",
                payload.len()
            )));
        }
        {
            let mut state = self.state.write().await;
            *state = NodeState::Syncing;
        }

        let guard = self.sender.lock().await;
        match guard.as_ref() {
            Some(sender) => {
                if let Err(e) = sender.broadcast(payload).await {
                    // Reset state before returning — otherwise a failed
                    // broadcast leaves the node stuck reporting "Syncing"
                    // forever, since the only other reset is a few lines
                    // below this branch.
                    let mut state = self.state.write().await;
                    *state = NodeState::Idle;
                    return Err(SyncError::Network(e.to_string()));
                }
            }
            None => {
                // Same reset requirement: this is the exact race a fresh
                // daemon hits if a push arrives before the background
                // `start()` task has finished subscribing — the node was
                // never `Syncing` to begin with, so leaving it there would
                // be actively misleading to `GET /api/v1/sync/status`.
                let mut state = self.state.write().await;
                *state = NodeState::Disconnected;
                return Err(SyncError::Network(
                    "P2P node not started — call start() first".into(),
                ));
            }
        }

        {
            let mut state = self.state.write().await;
            *state = NodeState::Idle;
        }

        Ok(())
    }

    /// Add new peers discovered out-of-band (e.g., from config or mDNS).
    pub async fn add_peers(&self, peers: Vec<EndpointId>) -> Result<(), SyncError> {
        let guard = self.sender.lock().await;
        match guard.as_ref() {
            Some(sender) => sender
                .join_peers(peers)
                .await
                .map_err(|e| SyncError::Network(e.to_string())),
            None => Err(SyncError::Network(
                "P2P node not started — call start() first".into(),
            )),
        }
    }

    /// Return the stable NodeId (Ed25519 public key) for this node.
    pub fn node_id(&self) -> EndpointId {
        self.endpoint.id()
    }

    /// Return the current FSM state.
    pub async fn state(&self) -> NodeState {
        self.state.read().await.clone()
    }

    /// Return the current set of gossip neighbors (peers with an active
    /// `NeighborUp` that haven't since gone `NeighborDown`).
    pub async fn neighbors(&self) -> Vec<EndpointId> {
        self.neighbors.read().await.iter().copied().collect()
    }

    /// Gracefully shut down the endpoint.
    pub async fn shutdown(self) -> Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_is_deterministic() {
        let op_id = [1u8; 32];
        let t1 = P2PNode::derive_topic(&op_id);
        let t2 = P2PNode::derive_topic(&op_id);
        assert_eq!(t1.as_bytes(), t2.as_bytes());
    }

    #[test]
    fn different_operators_get_different_topics() {
        let t1 = P2PNode::derive_topic(&[1u8; 32]);
        let t2 = P2PNode::derive_topic(&[2u8; 32]);
        assert_ne!(t1.as_bytes(), t2.as_bytes());
    }
}
