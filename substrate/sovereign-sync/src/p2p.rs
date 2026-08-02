use anyhow::Result;
use bytes::Bytes;
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId};
use iroh_gossip::{
    api::{Event, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
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
const P2P_COMMAND_CAPACITY: usize = 64;
const P2P_EVENT_CAPACITY: usize = 256;
const P2P_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum P2PTransportState {
    Disabled,
    Initializing,
    Bootstrapping,
    Ready,
    Degraded,
    Failed,
    Stopping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PStatusSnapshot {
    pub state: P2PTransportState,
    pub node_id: Option<String>,
    pub peers: Vec<String>,
    pub attempt: u32,
    pub last_error: Option<String>,
    pub next_retry_ms: Option<u64>,
}

impl P2PStatusSnapshot {
    pub fn disabled() -> Self {
        Self {
            state: P2PTransportState::Disabled,
            node_id: None,
            peers: Vec::new(),
            attempt: 0,
            last_error: None,
            next_retry_ms: None,
        }
    }

    fn initializing() -> Self {
        Self {
            state: P2PTransportState::Initializing,
            ..Self::disabled()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P2PHandleErrorKind {
    Unavailable,
    Timeout,
    Transport,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct P2PHandleError {
    pub kind: P2PHandleErrorKind,
    pub status: P2PStatusSnapshot,
    message: String,
}

impl P2PHandleError {
    pub fn unavailable(status: P2PStatusSnapshot) -> Self {
        Self {
            kind: P2PHandleErrorKind::Unavailable,
            message: format!("P2P transport is {:?}", status.state),
            status,
        }
    }

    pub fn transport(message: impl Into<String>, status: P2PStatusSnapshot) -> Self {
        Self {
            kind: P2PHandleErrorKind::Transport,
            message: message.into(),
            status,
        }
    }
}

enum P2PCommand {
    Broadcast {
        payload: Bytes,
        reply: oneshot::Sender<std::result::Result<(), String>>,
    },
    Shutdown {
        reply: oneshot::Sender<std::result::Result<(), String>>,
    },
}

#[derive(Clone)]
pub struct P2PHandle {
    status: Arc<std::sync::RwLock<P2PStatusSnapshot>>,
    commands: Arc<std::sync::RwLock<Option<mpsc::Sender<P2PCommand>>>>,
}

impl P2PHandle {
    pub fn disabled() -> Self {
        Self {
            status: Arc::new(std::sync::RwLock::new(P2PStatusSnapshot::disabled())),
            commands: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn pending() -> Self {
        Self {
            status: Arc::new(std::sync::RwLock::new(P2PStatusSnapshot::initializing())),
            commands: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn status(&self) -> P2PStatusSnapshot {
        self.status
            .read()
            .expect("P2P status lock poisoned")
            .clone()
    }

    pub fn mark_failed(&self, message: impl Into<String>) {
        let mut status = self.status();
        status.state = P2PTransportState::Failed;
        status.last_error = Some(message.into());
        status.next_retry_ms = None;
        self.replace_status(status);
    }

    fn replace_status(&self, status: P2PStatusSnapshot) {
        *self.status.write().expect("P2P status lock poisoned") = status;
    }

    fn attach(&self, commands: mpsc::Sender<P2PCommand>) {
        *self.commands.write().expect("P2P command lock poisoned") = Some(commands);
    }

    fn detach(&self) {
        *self.commands.write().expect("P2P command lock poisoned") = None;
    }

    pub async fn broadcast(&self, payload: Bytes) -> std::result::Result<(), P2PHandleError> {
        let status = self.status();
        if status.state != P2PTransportState::Ready {
            return Err(P2PHandleError::unavailable(status));
        }
        let commands = self
            .commands
            .read()
            .expect("P2P command lock poisoned")
            .clone()
            .ok_or_else(|| P2PHandleError {
                kind: P2PHandleErrorKind::Unavailable,
                status: self.status(),
                message: "P2P supervisor command channel is unavailable".into(),
            })?;
        let (reply, response) = oneshot::channel();
        let request = async {
            commands
                .send(P2PCommand::Broadcast { payload, reply })
                .await
                .map_err(|_| "P2P supervisor command channel closed".to_string())?;
            response
                .await
                .map_err(|_| "P2P supervisor dropped the broadcast reply".to_string())?
        };
        match tokio::time::timeout(P2P_REQUEST_TIMEOUT, request).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(message)) => Err(P2PHandleError {
                kind: P2PHandleErrorKind::Transport,
                status: self.status(),
                message,
            }),
            Err(_) => Err(P2PHandleError {
                kind: P2PHandleErrorKind::Timeout,
                status: self.status(),
                message: "P2P broadcast exceeded five seconds".into(),
            }),
        }
    }

    async fn request_shutdown(&self) -> std::result::Result<(), P2PHandleError> {
        let Some(commands) = self
            .commands
            .read()
            .expect("P2P command lock poisoned")
            .clone()
        else {
            return Ok(());
        };
        let (reply, response) = oneshot::channel();
        let request = async {
            commands
                .send(P2PCommand::Shutdown { reply })
                .await
                .map_err(|_| "P2P supervisor command channel closed".to_string())?;
            response
                .await
                .map_err(|_| "P2P supervisor dropped the shutdown reply".to_string())?
        };
        match tokio::time::timeout(P2P_REQUEST_TIMEOUT, request).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(message)) => Err(P2PHandleError {
                kind: P2PHandleErrorKind::Transport,
                status: self.status(),
                message,
            }),
            Err(_) => Err(P2PHandleError {
                kind: P2PHandleErrorKind::Timeout,
                status: self.status(),
                message: "P2P shutdown exceeded five seconds".into(),
            }),
        }
    }
}

pub struct P2PSupervisor {
    handle: P2PHandle,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
}

impl P2PSupervisor {
    pub fn spawn(
        operator_id: [u8; 32],
        peers_config: PeersConfig,
        handle: P2PHandle,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let (commands_tx, commands_rx) = mpsc::channel(P2P_COMMAND_CAPACITY);
        let (incoming_tx, incoming_rx) = mpsc::channel(P2P_EVENT_CAPACITY);
        handle.attach(commands_tx);
        let runtime_handle = handle.clone();
        let thread = std::thread::Builder::new()
            .name("sovereign-p2p".into())
            .spawn(move || {
                let failure_handle = runtime_handle.clone();
                let result = (|| {
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .thread_name("sovereign-p2p-worker")
                        .enable_all()
                        .build()?;
                    runtime.block_on(run_supervisor(
                        operator_id,
                        peers_config,
                        runtime_handle,
                        commands_rx,
                        incoming_tx,
                    ))
                })();
                if let Err(error) = &result {
                    failure_handle.detach();
                    failure_handle.mark_failed(error.to_string());
                }
                result
            })?;
        Ok((
            Self {
                handle,
                thread: Some(thread),
            },
            incoming_rx,
        ))
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let request_result = self.handle.request_shutdown().await;
        let thread = self
            .thread
            .take()
            .ok_or_else(|| anyhow::anyhow!("P2P supervisor thread is unavailable"))?;
        let deadline = tokio::time::Instant::now() + P2P_REQUEST_TIMEOUT;
        while !thread.is_finished() && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        if !thread.is_finished() {
            anyhow::bail!("P2P supervisor did not stop within five seconds");
        }
        let join_result = tokio::task::spawn_blocking(move || {
            thread
                .join()
                .map_err(|_| anyhow::anyhow!("P2P supervisor runtime panicked"))?
        })
        .await
        .map_err(|error| anyhow::anyhow!("P2P supervisor join task failed: {error}"))?;
        request_result.map_err(anyhow::Error::from)?;
        join_result
    }

    #[cfg(test)]
    fn spawn_fake_ready(handle: P2PHandle) -> Result<Self> {
        let (commands_tx, mut commands_rx) = mpsc::channel(P2P_COMMAND_CAPACITY);
        handle.attach(commands_tx);
        handle.replace_status(P2PStatusSnapshot {
            state: P2PTransportState::Ready,
            node_id: Some("test-node".into()),
            peers: Vec::new(),
            attempt: 1,
            last_error: None,
            next_retry_ms: None,
        });
        let runtime_handle = handle.clone();
        let thread = std::thread::Builder::new()
            .name("sovereign-p2p-test".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()?;
                runtime.block_on(async move {
                    while let Some(command) = commands_rx.recv().await {
                        match command {
                            P2PCommand::Broadcast { reply, .. } => {
                                let _ = reply.send(Ok(()));
                            }
                            P2PCommand::Shutdown { reply } => {
                                runtime_handle.replace_status(P2PStatusSnapshot {
                                    state: P2PTransportState::Stopping,
                                    ..runtime_handle.status()
                                });
                                let _ = reply.send(Ok(()));
                                break;
                            }
                        }
                    }
                    runtime_handle.detach();
                    Ok(())
                })
            })?;
        Ok(Self {
            handle,
            thread: Some(thread),
        })
    }
}

async fn run_supervisor(
    operator_id: [u8; 32],
    peers_config: PeersConfig,
    handle: P2PHandle,
    mut commands: mpsc::Receiver<P2PCommand>,
    incoming_tx: mpsc::Sender<PeerMessage>,
) -> Result<()> {
    let bootstrap_peers = peers_config
        .bootstrap
        .iter()
        .filter_map(|peer| match peer.parse() {
            Ok(peer) => Some(peer),
            Err(error) => {
                warn!(%peer, %error, "ignoring invalid P2P bootstrap peer");
                None
            }
        })
        .collect::<Vec<_>>();
    let mut attempt = 0_u32;
    let mut retry_seconds = 1_u64;

    loop {
        attempt = attempt.saturating_add(1);
        handle.replace_status(P2PStatusSnapshot {
            state: P2PTransportState::Initializing,
            node_id: None,
            peers: Vec::new(),
            attempt,
            last_error: None,
            next_retry_ms: None,
        });
        let bind_started = std::time::Instant::now();
        let initialized = P2PNode::new(&operator_id, &peers_config).await;
        tracing::info!(
            startup_phase = "p2p_bind",
            elapsed_ms = bind_started.elapsed().as_millis(),
            attempt,
            success = initialized.is_ok(),
            "production N0 P2P bind attempt completed"
        );
        let (node, mut incoming) = match initialized {
            Ok(node) => node,
            Err(error) => {
                let delay = jittered_retry(retry_seconds, attempt);
                handle.replace_status(P2PStatusSnapshot {
                    state: P2PTransportState::Failed,
                    node_id: None,
                    peers: Vec::new(),
                    attempt,
                    last_error: Some(error.to_string()),
                    next_retry_ms: Some(delay.as_millis().min(u64::MAX as u128) as u64),
                });
                if wait_for_retry_or_shutdown(&handle, &mut commands, delay).await? {
                    break;
                }
                retry_seconds = (retry_seconds * 2).min(60);
                continue;
            }
        };

        let node_id = node.node_id().to_string();
        handle.replace_status(P2PStatusSnapshot {
            state: P2PTransportState::Bootstrapping,
            node_id: Some(node_id.clone()),
            peers: Vec::new(),
            attempt,
            last_error: None,
            next_retry_ms: None,
        });
        let subscribe_started = std::time::Instant::now();
        if let Err(error) = node.start(bootstrap_peers.clone()).await {
            tracing::warn!(%error, attempt, "P2P gossip subscription failed");
            let _ = node.shutdown().await;
            let delay = jittered_retry(retry_seconds, attempt);
            handle.replace_status(P2PStatusSnapshot {
                state: P2PTransportState::Failed,
                node_id: Some(node_id),
                peers: Vec::new(),
                attempt,
                last_error: Some(error.to_string()),
                next_retry_ms: Some(delay.as_millis().min(u64::MAX as u128) as u64),
            });
            if wait_for_retry_or_shutdown(&handle, &mut commands, delay).await? {
                break;
            }
            retry_seconds = (retry_seconds * 2).min(60);
            continue;
        }
        tracing::info!(
            startup_phase = "gossip_subscription",
            elapsed_ms = subscribe_started.elapsed().as_millis(),
            attempt,
            "P2P gossip subscription completed"
        );
        handle.replace_status(P2PStatusSnapshot {
            state: P2PTransportState::Ready,
            node_id: Some(node_id),
            peers: Vec::new(),
            attempt,
            last_error: None,
            next_retry_ms: None,
        });

        let mut peer_refresh = tokio::time::interval(Duration::from_secs(1));
        let mut shutdown_reply = None;
        let mut retry_after_shutdown = false;
        loop {
            tokio::select! {
                command = commands.recv() => match command {
                    Some(P2PCommand::Broadcast { payload, reply }) => {
                        let result = node.broadcast(payload).await.map_err(|error| error.to_string());
                        if let Err(error) = &result {
                            let mut status = handle.status();
                            status.state = P2PTransportState::Degraded;
                            status.last_error = Some(error.clone());
                            handle.replace_status(status);
                        }
                        let _ = reply.send(result);
                    }
                    Some(P2PCommand::Shutdown { reply }) => {
                        shutdown_reply = Some(reply);
                        break;
                    }
                    None => break,
                },
                message = incoming.recv() => match message {
                    Some(message) => {
                        if incoming_tx.try_send(message).is_err() {
                            let mut status = handle.status();
                            status.state = P2PTransportState::Degraded;
                            status.last_error = Some("local authority consumer is lagging".into());
                            handle.replace_status(status);
                        }
                    }
                    None => {
                        let mut status = handle.status();
                        status.state = P2PTransportState::Degraded;
                        status.last_error = Some("gossip receive loop stopped".into());
                        handle.replace_status(status);
                        retry_after_shutdown = true;
                        break;
                    }
                },
                _ = peer_refresh.tick() => {
                    let peers = node.neighbors().await.into_iter().map(|peer| peer.to_string()).collect();
                    let mut status = handle.status();
                    status.peers = peers;
                    if status.state == P2PTransportState::Degraded && status.last_error.as_deref() == Some("local authority consumer is lagging") {
                        status.state = P2PTransportState::Ready;
                        status.last_error = None;
                    }
                    handle.replace_status(status);
                }
            }
        }

        handle.replace_status(P2PStatusSnapshot {
            state: P2PTransportState::Stopping,
            ..handle.status()
        });
        let shutdown_result = node.shutdown().await.map_err(|error| error.to_string());
        if let Some(reply) = shutdown_reply {
            let _ = reply.send(shutdown_result.clone().map(|_| ()));
        }
        shutdown_result.map_err(anyhow::Error::msg)?;
        if retry_after_shutdown {
            let delay = jittered_retry(retry_seconds, attempt);
            let mut status = handle.status();
            status.state = P2PTransportState::Failed;
            status.next_retry_ms = Some(delay.as_millis().min(u64::MAX as u128) as u64);
            handle.replace_status(status);
            if wait_for_retry_or_shutdown(&handle, &mut commands, delay).await? {
                break;
            }
            retry_seconds = (retry_seconds * 2).min(60);
            continue;
        }
        break;
    }
    handle.detach();
    handle.replace_status(P2PStatusSnapshot {
        state: P2PTransportState::Disabled,
        ..handle.status()
    });
    Ok(())
}

async fn wait_for_retry_or_shutdown(
    handle: &P2PHandle,
    commands: &mut mpsc::Receiver<P2PCommand>,
    delay: Duration,
) -> Result<bool> {
    tokio::select! {
        _ = tokio::time::sleep(delay) => Ok(false),
        command = commands.recv() => match command {
            Some(P2PCommand::Shutdown { reply }) => {
                handle.replace_status(P2PStatusSnapshot {
                    state: P2PTransportState::Stopping,
                    ..handle.status()
                });
                let _ = reply.send(Ok(()));
                Ok(true)
            }
            Some(P2PCommand::Broadcast { reply, .. }) => {
                let _ = reply.send(Err("P2P transport is retrying initialization".into()));
                Ok(false)
            }
            None => Ok(true),
        }
    }
}

fn jittered_retry(base_seconds: u64, attempt: u32) -> Duration {
    let seed = blake3::hash(&attempt.to_le_bytes());
    let percent = 90 + u64::from(seed.as_bytes()[0] % 21);
    Duration::from_millis(base_seconds.saturating_mul(1_000).saturating_mul(percent) / 100)
}

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

    #[tokio::test]
    async fn bounded_handle_broadcasts_and_joins_its_dedicated_runtime() {
        let handle = P2PHandle::pending();
        let supervisor = P2PSupervisor::spawn_fake_ready(handle.clone()).unwrap();

        handle
            .broadcast(Bytes::from_static(b"signed-event"))
            .await
            .unwrap();
        supervisor.shutdown().await.unwrap();

        assert!(matches!(
            handle.status().state,
            P2PTransportState::Stopping | P2PTransportState::Disabled
        ));
    }
}
