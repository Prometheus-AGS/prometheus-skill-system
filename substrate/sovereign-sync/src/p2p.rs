use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bytes::Bytes;
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId, SecretKey, Signature};
use iroh_gossip::{
    api::{Event, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
const PEER_FRAME_MAX_AGE: Duration = Duration::from_secs(300);
const PEER_REPLAY_WINDOW: usize = 4_096;

#[derive(Clone)]
pub struct P2PIdentity {
    secret_key: SecretKey,
    group_secret: [u8; 32],
    allow_list: BTreeMap<String, String>,
    path: PathBuf,
}

impl std::fmt::Debug for P2PIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("P2PIdentity")
            .field("endpoint_id", &self.endpoint_id())
            .field("allow_list_entries", &self.allow_list.len())
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StoredP2PIdentity {
    schema_version: String,
    secret_key: String,
    group_secret: String,
    #[serde(default)]
    allow_list: BTreeMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PairingTicket {
    pub protocol_version: String,
    pub group_secret: String,
    pub endpoint_id: String,
    pub signing_key_fingerprint: String,
}

impl std::fmt::Debug for PairingTicket {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PairingTicket")
            .field("protocol_version", &self.protocol_version)
            .field("endpoint_id", &self.endpoint_id)
            .field("signing_key_fingerprint", &self.signing_key_fingerprint)
            .field("group_secret", &"[REDACTED]")
            .finish()
    }
}

impl P2PIdentity {
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            return Self::load(path);
        }
        let mut group_secret = [0_u8; 32];
        OsRng.fill_bytes(&mut group_secret);
        let identity = Self {
            secret_key: SecretKey::generate(),
            group_secret,
            allow_list: BTreeMap::new(),
            path,
        };
        identity.persist()?;
        Ok(identity)
    }

    fn load(path: PathBuf) -> Result<Self> {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            anyhow::bail!("{} must be a regular P2P identity file", path.display());
        }
        #[cfg(unix)]
        if metadata.permissions().mode() & 0o077 != 0 {
            anyhow::bail!("{} must have mode 0600", path.display());
        }
        let stored: StoredP2PIdentity = serde_json::from_reader(File::open(&path)?)?;
        if stored.schema_version != "1" {
            anyhow::bail!("unsupported P2P identity schema {}", stored.schema_version);
        }
        let secret: [u8; 32] = URL_SAFE_NO_PAD
            .decode(stored.secret_key)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("P2P secret key must contain 32 bytes"))?;
        let group_secret: [u8; 32] = URL_SAFE_NO_PAD
            .decode(stored.group_secret)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("P2P group secret must contain 32 bytes"))?;
        Ok(Self {
            secret_key: SecretKey::from_bytes(&secret),
            group_secret,
            allow_list: stored.allow_list,
            path,
        })
    }

    fn persist(&self) -> Result<()> {
        let stored = StoredP2PIdentity {
            schema_version: "1".into(),
            secret_key: URL_SAFE_NO_PAD.encode(self.secret_key.to_bytes()),
            group_secret: URL_SAFE_NO_PAD.encode(self.group_secret),
            allow_list: self.allow_list.clone(),
        };
        write_private_json(&self.path, &stored)
    }

    pub fn endpoint_id(&self) -> EndpointId {
        self.secret_key.public()
    }

    pub fn signing_key_fingerprint(&self) -> String {
        endpoint_signing_fingerprint(self.endpoint_id())
    }

    pub fn export_ticket(&self) -> Result<String> {
        let ticket = PairingTicket {
            protocol_version: "1".into(),
            group_secret: URL_SAFE_NO_PAD.encode(self.group_secret),
            endpoint_id: self.endpoint_id().to_string(),
            signing_key_fingerprint: self.signing_key_fingerprint(),
        };
        Ok(URL_SAFE_NO_PAD.encode(serde_jcs::to_vec(&ticket)?))
    }

    pub fn import_ticket(&mut self, encoded: &str) -> Result<PairingTicket> {
        let ticket: PairingTicket = serde_json::from_slice(
            &URL_SAFE_NO_PAD
                .decode(encoded.trim())
                .context("invalid pairing ticket encoding")?,
        )?;
        if ticket.protocol_version != "1" {
            anyhow::bail!("unsupported pairing protocol {}", ticket.protocol_version);
        }
        let group_secret: [u8; 32] = URL_SAFE_NO_PAD
            .decode(&ticket.group_secret)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("pairing group secret must contain 32 bytes"))?;
        let endpoint_id = ticket.endpoint_id.parse::<EndpointId>()?;
        if ticket.signing_key_fingerprint.trim().is_empty() {
            anyhow::bail!("pairing ticket has no signing-key fingerprint");
        }
        if ticket.signing_key_fingerprint != endpoint_signing_fingerprint(endpoint_id) {
            anyhow::bail!("pairing ticket signing-key fingerprint does not match endpoint ID");
        }
        self.group_secret = group_secret;
        self.allow_list.insert(
            ticket.endpoint_id.clone(),
            ticket.signing_key_fingerprint.clone(),
        );
        self.persist()?;
        Ok(ticket)
    }

    pub fn authorize_endpoint(&self, endpoint: EndpointId, signer_key_id: &str) -> bool {
        self.allow_list
            .get(&endpoint.to_string())
            .is_some_and(|fingerprint| fingerprint == signer_key_id)
    }

    fn sign_frame(&self, payload: Bytes) -> Result<AuthenticatedPeerFrame> {
        let mut frame = AuthenticatedPeerFrame {
            schema_version: "1".into(),
            frame_id: uuid::Uuid::new_v4().to_string(),
            issued_at_ms: unix_time_ms()?,
            endpoint_id: self.endpoint_id().to_string(),
            signing_key_fingerprint: self.signing_key_fingerprint(),
            payload: payload.to_vec(),
            signature: String::new(),
        };
        frame.signature =
            URL_SAFE_NO_PAD.encode(self.secret_key.sign(&frame.signable_bytes()?).to_bytes());
        Ok(frame)
    }

    fn verify_frame(
        &self,
        delivered_from: EndpointId,
        encoded: &[u8],
        replay_ids: &mut HashSet<String>,
    ) -> Result<Bytes> {
        let frame: AuthenticatedPeerFrame =
            serde_json::from_slice(encoded).context("P2P message is not an authenticated frame")?;
        if frame.schema_version != "1" {
            anyhow::bail!("unsupported P2P frame schema {}", frame.schema_version);
        }
        if frame.endpoint_id != delivered_from.to_string() {
            anyhow::bail!("P2P delivery endpoint does not match signed frame endpoint");
        }
        if !self.authorize_endpoint(delivered_from, &frame.signing_key_fingerprint) {
            anyhow::bail!("P2P endpoint/signing-key binding is not enrolled");
        }
        let now = unix_time_ms()?;
        let max_age = PEER_FRAME_MAX_AGE.as_millis() as u64;
        if frame.issued_at_ms > now.saturating_add(30_000)
            || now.saturating_sub(frame.issued_at_ms) > max_age
        {
            anyhow::bail!("P2P frame is stale or issued too far in the future");
        }
        if replay_ids.contains(&frame.frame_id) {
            anyhow::bail!("P2P frame ID was already received");
        }
        let signature_bytes: [u8; Signature::LENGTH] = URL_SAFE_NO_PAD
            .decode(&frame.signature)
            .context("invalid P2P frame signature encoding")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid P2P frame signature length"))?;
        let signature = Signature::from_bytes(&signature_bytes);
        delivered_from
            .verify(&frame.signable_bytes()?, &signature)
            .map_err(|_| anyhow::anyhow!("P2P frame signature verification failed"))?;
        if replay_ids.len() >= PEER_REPLAY_WINDOW {
            replay_ids.clear();
        }
        replay_ids.insert(frame.frame_id);
        Ok(Bytes::from(frame.payload))
    }
}

fn endpoint_signing_fingerprint(endpoint: EndpointId) -> String {
    format!("ed25519:{}", blake3::hash(endpoint.as_bytes()).to_hex())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthenticatedPeerFrame {
    schema_version: String,
    frame_id: String,
    issued_at_ms: u64,
    endpoint_id: String,
    signing_key_fingerprint: String,
    payload: Vec<u8>,
    signature: String,
}

impl AuthenticatedPeerFrame {
    fn signable_bytes(&self) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Signable<'a> {
            schema_version: &'a str,
            frame_id: &'a str,
            issued_at_ms: u64,
            endpoint_id: &'a str,
            signing_key_fingerprint: &'a str,
            payload: &'a [u8],
        }
        Ok(serde_jcs::to_vec(&Signable {
            schema_version: &self.schema_version,
            frame_id: &self.frame_id,
            issued_at_ms: self.issued_at_ms,
            endpoint_id: &self.endpoint_id,
            signing_key_fingerprint: &self.signing_key_fingerprint,
            payload: &self.payload,
        })?)
    }
}

fn unix_time_ms() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX))
}

fn write_private_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("identity path has no parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".p2p-identity-{}.tmp", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

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
        identity: P2PIdentity,
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
                        identity,
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
    identity: P2PIdentity,
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
        let initialized = P2PNode::new_with_identity(&identity, &peers_config).await;
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
        let mut replay_ids = HashSet::new();
        loop {
            tokio::select! {
                command = commands.recv() => match command {
                    Some(P2PCommand::Broadcast { payload, reply }) => {
                        let result = match identity.sign_frame(payload) {
                            Ok(frame) => match serde_json::to_vec(&frame) {
                                Ok(encoded) => node.broadcast(Bytes::from(encoded)).await.map_err(|error| error.to_string()),
                                Err(error) => Err(format!("failed to encode authenticated P2P frame: {error}")),
                            },
                            Err(error) => Err(format!("failed to sign P2P frame: {error}")),
                        };
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
                    Some(mut message) => {
                        let verified = match identity.verify_frame(message.from, &message.payload, &mut replay_ids) {
                            Ok(payload) => payload,
                            Err(error) => {
                                warn!(peer = %message.from, %error, "dropping unauthenticated P2P frame");
                                continue;
                            }
                        };
                        if !payload_targets_local_endpoint(&verified, identity.endpoint_id()) {
                            debug!(peer = %message.from, "ignoring authenticated P2P payload targeted to other endpoints");
                            continue;
                        }
                        message.payload = verified;
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

fn payload_targets_local_endpoint(payload: &[u8], local_endpoint: EndpointId) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(payload) else {
        return true;
    };
    let Some(targets) = value
        .get("target_endpoint_ids")
        .and_then(serde_json::Value::as_array)
    else {
        return true;
    };
    targets.is_empty()
        || targets
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|target| target == local_endpoint.to_string())
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
    /// The random group secret derives the gossip topic; the durable iroh key
    /// preserves this endpoint's identity across restarts.
    pub async fn new_with_identity(
        identity: &P2PIdentity,
        _peers_config: &PeersConfig,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let topic = Self::derive_topic(&identity.group_secret);

        // N0 preset: pkarr DNS discovery + relay mode, with bundled crypto provider.
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(identity.secret_key.clone())
            .bind()
            .await?;

        Ok(Self::from_endpoint(topic, endpoint))
    }

    /// Disposable constructor with an ephemeral endpoint key. Production code
    /// must use `new_with_identity` so endpoint identity survives restarts.
    pub async fn new(
        group_secret: &[u8; 32],
        _peers_config: &PeersConfig,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let topic = Self::derive_topic(group_secret);
        let endpoint = Endpoint::builder(presets::N0).bind().await?;
        Ok(Self::from_endpoint(topic, endpoint))
    }

    #[cfg(test)]
    pub(crate) async fn new_with_memory_lookup(
        group_secret: &[u8; 32],
        address_lookup: MemoryLookup,
    ) -> Result<(Self, mpsc::Receiver<PeerMessage>)> {
        let topic = Self::derive_topic(group_secret);
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

    /// Derive a deterministic TopicId from a random 256-bit group secret.
    pub fn derive_topic(group_secret: &[u8; 32]) -> TopicId {
        let mut input = group_secret.to_vec();
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

    #[test]
    fn targeted_payloads_are_filtered_before_domain_delivery() {
        let local = SecretKey::generate().public();
        let other = SecretKey::generate().public();
        let broadcast = serde_json::to_vec(&serde_json::json!({
            "target_endpoint_ids": []
        }))
        .unwrap();
        let targeted_local = serde_json::to_vec(&serde_json::json!({
            "target_endpoint_ids": [local.to_string()]
        }))
        .unwrap();
        let targeted_other = serde_json::to_vec(&serde_json::json!({
            "target_endpoint_ids": [other.to_string()]
        }))
        .unwrap();
        assert!(payload_targets_local_endpoint(&broadcast, local));
        assert!(payload_targets_local_endpoint(&targeted_local, local));
        assert!(!payload_targets_local_endpoint(&targeted_other, local));
    }

    #[test]
    fn identity_is_private_stable_and_pairing_enrolls_endpoint_binding() {
        let directory = tempfile::tempdir().unwrap();
        let first_path = directory.path().join("first.json");
        let second_path = directory.path().join("second.json");
        let first = P2PIdentity::load_or_create(&first_path).unwrap();
        let endpoint = first.endpoint_id();
        let ticket = first.export_ticket().unwrap();
        assert!(!format!("{first:?}").contains(&URL_SAFE_NO_PAD.encode(first.group_secret)));
        let reopened = P2PIdentity::load_or_create(&first_path).unwrap();
        assert_eq!(reopened.endpoint_id(), endpoint);
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&first_path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let mut second = P2PIdentity::load_or_create(&second_path).unwrap();
        second.import_ticket(&ticket).unwrap();
        assert_eq!(
            P2PNode::derive_topic(&first.group_secret),
            P2PNode::derive_topic(&second.group_secret)
        );
        assert!(second.authorize_endpoint(endpoint, &first.signing_key_fingerprint()));
        assert!(!second.authorize_endpoint(endpoint, "ed25519:wrong"));

        let mut mismatched: PairingTicket =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&ticket).unwrap()).unwrap();
        mismatched.signing_key_fingerprint = "ed25519:wrong".into();
        let mismatched = URL_SAFE_NO_PAD.encode(serde_jcs::to_vec(&mismatched).unwrap());
        assert!(second.import_ticket(&mismatched).is_err());

        let frame = first
            .sign_frame(Bytes::from_static(b"authenticated"))
            .unwrap();
        let encoded = serde_json::to_vec(&frame).unwrap();
        let mut replay_ids = HashSet::new();
        assert_eq!(
            second
                .verify_frame(endpoint, &encoded, &mut replay_ids)
                .unwrap(),
            Bytes::from_static(b"authenticated")
        );
        assert!(second
            .verify_frame(endpoint, &encoded, &mut replay_ids)
            .is_err());

        let unknown = P2PIdentity::load_or_create(directory.path().join("unknown.json")).unwrap();
        let unknown_frame =
            serde_json::to_vec(&unknown.sign_frame(Bytes::from_static(b"unknown")).unwrap())
                .unwrap();
        assert!(second
            .verify_frame(unknown.endpoint_id(), &unknown_frame, &mut HashSet::new())
            .is_err());
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
