/// Axum REST API for sovereign-sync daemon / server modes.
///
/// Routes:
///   GET  /health
///   GET  /api/v1/skills/search?q=<query>
///   GET  /api/v1/sync/status
///   GET  /api/v1/sync/peers
///   POST /api/v1/sync/push     { "domain": "<name>" }
///   POST /api/v1/stream        (AG-UI SSE — delegates to ag_ui module)
///   GET  /api/v1/stream/ping   (AG-UI SSE ping)
use axum::{
    extract::{Path as AxumPath, Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event as SseEvent, KeepAlive},
        IntoResponse, Response, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::stream;
use kbd_runtime::CommandEnvelope;
use serde::Deserialize;
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};
use storage_provider::{CrdtEngine, LoroAdapter, SyncDomain, SyncManifest};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::info;

use crate::ag_ui::{ag_ui_ping, ag_ui_stream, AgUiState};
use crate::domains::{self, DomainAdapter, LearnerModelAdapter, SkillIndexAdapter, SyncEnvelope};
use crate::kbd_control::KbdControlPlane;
use crate::kbd_raft::QuorumPolicy;
use crate::kbd_sync::KbdPresenceDocument;
use crate::mcp_server::SkillIndex;
use crate::p2p::P2PNode;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    skill_index: Arc<SkillIndex>,
    ag_ui: AgUiState,
    kbd_control: Arc<KbdControlPlane>,
    bearer_token: Arc<str>,
    /// Domain sync policy (privacy class + storage prefix), populated
    /// lazily on first use of a concrete domain instance.
    manifest: Arc<AsyncRwLock<SyncManifest>>,
    /// CRDT snapshot bytes per domain, keyed by the full parametrized
    /// domain string. Stored as bytes (not a live `LoroDoc`) so `AppState`
    /// stays trivially `Send + Sync` for axum's `State` extractor — a
    /// `LoroDoc` is reconstructed transiently within a synchronous block
    /// wherever it's needed and never held across an `.await`.
    docs: Arc<StdMutex<HashMap<SyncDomain, Vec<u8>>>>,
    /// Present in daemon mode (real P2P gossip); absent in plain server mode.
    p2p: Option<Arc<P2PNode>>,
    adapters: Arc<HashMap<String, Box<dyn DomainAdapter>>>,
    /// `kbd-control` presence — handled directly here, not through
    /// `adapters`/`docs` (see `domains.rs`'s module comment on why it isn't
    /// a `DomainAdapter`).
    presence: Arc<KbdPresenceDocument>,
}

impl AppState {
    pub async fn new(skills_dir: &Path) -> Self {
        Self::try_new(skills_dir, None)
            .await
            .expect("cannot open KBD control plane")
    }

    pub async fn try_new(skills_dir: &Path, p2p: Option<Arc<P2PNode>>) -> anyhow::Result<Self> {
        let project_root = discover_project_root(skills_dir);
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_control = Arc::new(KbdControlPlane::open(&project_root, quorum).await?);
        let learner_model_dir = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".prometheus")
            .join("learn")
            .join("learner-model");
        Self::from_control_plane(skills_dir, kbd_control, learner_model_dir, p2p)
    }

    pub async fn try_new_at(
        skills_dir: &Path,
        project_root: &Path,
        data_root: &Path,
        p2p: Option<Arc<P2PNode>>,
    ) -> anyhow::Result<Self> {
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_control =
            Arc::new(KbdControlPlane::open_at(project_root, data_root, quorum).await?);
        Self::from_control_plane(
            skills_dir,
            kbd_control,
            learner_model_dir_at(data_root),
            p2p,
        )
    }

    fn from_control_plane(
        skills_dir: &Path,
        kbd_control: Arc<KbdControlPlane>,
        learner_model_dir: PathBuf,
        p2p: Option<Arc<P2PNode>>,
    ) -> anyhow::Result<Self> {
        let bearer_token: Arc<str> = kbd_control.runtime().control_token()?.into();
        let skill_index = Arc::new(SkillIndex::load_from_dir(skills_dir));
        // Best-effort: an uninitialized runtime has no project_id yet. The
        // presence domain is scoped by this at import time via SyncEnvelope's
        // `identity` field, same as before — an empty scope just means no
        // legitimate push will ever match it until the runtime initializes.
        let project_id = kbd_control
            .status()
            .map(|status| status.project_id)
            .unwrap_or_default();
        let presence = Arc::new(KbdPresenceDocument::new(project_id));

        let mut adapters: HashMap<String, Box<dyn DomainAdapter>> = HashMap::new();
        adapters.insert(
            "skill-index".to_string(),
            Box::new(SkillIndexAdapter::new(skill_index.clone())),
        );
        adapters.insert(
            "learner-model".to_string(),
            Box::new(LearnerModelAdapter::new(
                learner_model_dir,
                default_learner_id(),
            )),
        );

        Ok(Self {
            skill_index,
            ag_ui: AgUiState::new(),
            kbd_control,
            bearer_token,
            manifest: Arc::new(AsyncRwLock::new(SyncManifest::new())),
            docs: Arc::new(StdMutex::new(HashMap::new())),
            p2p,
            adapters: Arc::new(adapters),
            presence,
        })
    }

    pub fn bearer_token(&self) -> &str {
        &self.bearer_token
    }

    /// Inspection accessor for tests — not part of the wire API.
    pub fn skill_index(&self) -> &SkillIndex {
        &self.skill_index
    }

    /// Inspection accessor for tests — not part of the wire API.
    pub fn presence(&self) -> &KbdPresenceDocument {
        &self.presence
    }
}

/// Best-effort default learner identity for the `learner-model` domain when
/// no per-learner scoping has been configured — the local OS user. `pub` so
/// integration tests can seed/read the same storage key the adapter itself
/// uses, without duplicating the env-var fallback logic.
pub fn default_learner_id() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "local-learner".into())
}

/// Where `AppState::try_new_at`'s `learner-model` adapter stores its CRDT
/// document, scoped under the caller's `data_root` (mirrors `kbd_control`'s
/// own scoping) so tests using separate `TempDir`s per node get isolated
/// storage instead of silently sharing (and mutating) the same directory.
/// `pub` so integration tests can point their own `LearnerModelStore` at the
/// exact path the adapter uses, to seed/assert content directly.
pub fn learner_model_dir_at(data_root: &Path) -> PathBuf {
    data_root.join("learn").join("learner-model")
}

/// Best-effort device identity for the `kbd-control` presence domain — derived from
/// the local home directory name, not a security identity.
fn device_identity() -> String {
    dirs_next::home_dir()
        .and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown-device".into())
}

fn discover_project_root(skills_dir: &Path) -> PathBuf {
    if let Ok(path) = std::env::var("KBD_FOCUS_PROJECT_PATH") {
        return PathBuf::from(path);
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(root) = cwd
            .ancestors()
            .find(|candidate| candidate.join(".kbd-orchestrator").is_dir())
        {
            return root.to_path_buf();
        }
    }
    skills_dir.parent().unwrap_or(skills_dir).to_path_buf()
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// GET /health
/// Liveness AND readiness — the store must actually be reachable.
///
/// # Why this probes instead of answering `ok` unconditionally
///
/// This handler used to take no state and return a hardcoded `"status": "ok"`.
/// It therefore reported healthy while the daemon was in a `Database already
/// open. Cannot acquire lock.` loop and could not serve a single request that
/// touched its store.
///
/// A health endpoint that cannot observe its own core dependency gives false
/// assurance exactly when something is wrong — the same failure class as an
/// update check reporting `up-to-date` while offline. The green signal is
/// measuring process liveness, and callers read it as service readiness.
///
/// `KbdControlPlane::status()` is the cheapest honest probe: it reads committed
/// state through the redb store, so a lock failure or corrupt store surfaces
/// here rather than three calls later in something a user was relying on.
///
/// Returns **503** when the store is unreachable, so a load balancer or a
/// monitor can act on it. The body always names the reason.
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match state.kbd_control.status() {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "service": "sovereign-sync",
                "version": env!("CARGO_PKG_VERSION"),
                "store": "reachable"
            })),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "degraded",
                "service": "sovereign-sync",
                "version": env!("CARGO_PKG_VERSION"),
                "store": "unreachable",
                "reason": e.to_string()
            })),
        ),
    }
}

async fn require_bearer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    // The local KBD control plane needs no shared secret, because there was
    // never a shared secret to have.
    //
    // Both sides call `Runtime::control_token()`, which resolves
    // `<runtime_root>/control-token` and MINTS A FRESH RANDOM TOKEN when the
    // file is absent. The CLI's root is whatever project it was invoked in; the
    // daemon's is whatever it was launched against. Verified on a real machine:
    // no `control-token` file existed anywhere, so the two processes generated
    // DIFFERENT 32-byte secrets and every write returned
    // `401 missing or invalid bearer token`. The check could not pass, by
    // construction — it gated the tool without protecting anything.
    //
    // Three conditions, all required:
    //
    //   1. loopback — structural here: `serve()` binds a hard-coded
    //      `127.0.0.1` (see `SocketAddr::from(([127, 0, 0, 1], port))`), with
    //      no configuration path to widen it. A remote caller cannot reach this
    //      code at all.
    //   2. the KBD control-plane prefix only — sync, peer, and skill routes
    //      still require a token, so this does not become a blanket bypass.
    //   3. no token was EXPLICITLY configured — if an operator sets
    //      `PROMETHEUS_CONTROL_TOKEN_FILE`, they mean it, and it is enforced.
    //
    // This removes an unsatisfiable default. It does not remove the ability to
    // require authentication.
    if request.uri().path().starts_with("/api/v1/kbd/")
        && std::env::var_os("PROMETHEUS_CONTROL_TOKEN_FILE").is_none()
    {
        return next.run(request).await;
    }
    let supplied = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let authorized = supplied
        .map(|token| blake3::hash(token.as_bytes()) == blake3::hash(state.bearer_token.as_bytes()))
        .unwrap_or(false);
    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"missing or invalid bearer token"})),
        )
            .into_response();
    }
    next.run(request).await
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

/// GET /api/v1/skills/search?q=<query>&limit=<n>
async fn skills_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let results = state.skill_index.search(&params.q);
    let limited: Vec<_> = results
        .into_iter()
        .take(params.limit)
        .map(|e| {
            serde_json::json!({
                "name": e.name,
                "description": e.description
            })
        })
        .collect();
    Json(serde_json::json!({
        "query": params.q,
        "count": limited.len(),
        "results": limited
    }))
}

/// GET /api/v1/sync/status
async fn sync_status(State(state): State<AppState>) -> impl IntoResponse {
    let (node_state, peers) = match &state.p2p {
        Some(p2p) => (
            format!("{:?}", p2p.state().await),
            p2p.neighbors()
                .await
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>(),
        ),
        None => ("no-p2p".to_string(), Vec::new()),
    };
    Json(serde_json::json!({
        "node_state": node_state,
        "peers": peers,
        "domains": {
            "skill-index":    { "privacy": "public",      "adapter": "wired" },
            "learner-model":  { "privacy": "trusted",      "adapter": "wired" },
            "kbd-control":   { "privacy": "trusted",      "adapter": "wired" },
            "surreal-memory": { "privacy": "local_only",   "adapter": "never-synced" }
        }
    }))
}

/// GET /api/v1/sync/peers
async fn sync_peers(State(state): State<AppState>) -> impl IntoResponse {
    let peers = match &state.p2p {
        Some(p2p) => p2p
            .neighbors()
            .await
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    Json(serde_json::json!({ "peers": peers }))
}

#[derive(Debug, Deserialize)]
struct SyncPushBody {
    domain: String,
}

/// POST /api/v1/sync/push { "domain": "<name>" }
///
/// 1. registers the concrete domain instance (idempotent) and checks it's
///    syncable at all (rejects unregistered families and `PrivacyClass::Local`
///    outright — `surreal-memory` never leaves this device);
/// 2. asks the domain's adapter for its current local state as JSON;
/// 3. merges that JSON into the domain's CRDT snapshot (`CrdtEngine::apply_json`,
///    synchronous — no `.await` while touching CRDT bytes);
/// 4. broadcasts the resulting delta, wrapped in a [`SyncEnvelope`], over the
///    P2P gossip layer (if this node has one; server mode does not).
/// Result of preparing a domain push, before any network broadcast happens.
/// Split out from the HTTP handler so tests (and any future non-HTTP caller)
/// can exercise the real registration/adapter/CRDT-merge pipeline directly,
/// without needing a live P2P transport.
pub enum PushOutcome {
    /// This node has no P2P transport (server mode) — merged locally only.
    LocalOnly { snapshot_bytes: usize },
    /// Ready to broadcast; the envelope carries the CRDT delta to send.
    Broadcast {
        envelope: SyncEnvelope,
        snapshot_bytes: usize,
    },
}

/// Register the domain (idempotent), reject it if not syncable, ask its
/// adapter for current local state, and merge that into the domain's CRDT
/// snapshot — everything `sync_push` does except the actual network send.
/// Returns `Err((status, body))` for the same failure cases the HTTP handler
/// reports.
pub async fn build_push_envelope(
    state: &AppState,
    domain_name: &str,
) -> Result<PushOutcome, (StatusCode, serde_json::Value)> {
    let domain = SyncDomain::new(domain_name.to_string());
    let syncable = {
        let mut manifest = state.manifest.write().await;
        domains::ensure_registered(&mut manifest, &domain);
        manifest.is_syncable(&domain)
    };
    if !syncable {
        return Err((
            StatusCode::FORBIDDEN,
            serde_json::json!({
                "error": "domain is not syncable (unregistered family or PrivacyClass::Local)",
                "domain": domain_name
            }),
        ));
    }

    let family = domains::domain_family(&domain).to_string();

    if family == "kbd-control" {
        return build_presence_push_envelope(state, domain_name).await;
    }

    let adapter = state.adapters.get(&family).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            serde_json::json!({
                "error": "no adapter registered for this domain family",
                "family": family
            }),
        )
    })?;

    let local_json = adapter.export_json().await.map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;

    let crdt = LoroAdapter;
    let (new_snapshot, delta) = {
        let mut docs = state.docs.lock().expect("docs mutex poisoned");
        let existing = docs.get(&domain).cloned().unwrap_or_else(|| crdt.new_doc());
        let (new_snapshot, delta) = crdt.apply_json(&existing, local_json).map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"error": error.to_string()}),
            )
        })?;
        docs.insert(domain.clone(), new_snapshot.clone());
        (new_snapshot, delta)
    };

    if state.p2p.is_none() {
        return Ok(PushOutcome::LocalOnly {
            snapshot_bytes: new_snapshot.len(),
        });
    }

    // Must match handle_incoming_message's per-family `local_identity` check
    // exactly, or every push for that family is silently dropped as a false
    // identity mismatch on the receiving side (learner-model scopes by
    // learner_id, not project_id; kbd-control has its own dedicated path,
    // see build_presence_push_envelope).
    let identity = match family.as_str() {
        "learner-model" => Some(default_learner_id()),
        _ => None,
    };
    Ok(PushOutcome::Broadcast {
        envelope: SyncEnvelope {
            schema_version: "1".into(),
            domain: domain_name.to_string(),
            identity,
            payload: delta,
            signer_key_id: None,
            signature: None,
        },
        snapshot_bytes: new_snapshot.len(),
    })
}

/// `kbd-control` push: refresh this node's own presence entry in the
/// dedicated `KbdPresenceDocument`, export a full snapshot (Loro merges a
/// snapshot into an existing doc correctly, so no incremental-delta
/// tracking is needed for a small, infrequent presence heartbeat), sign it
/// with this node's device identity, and broadcast. Bypasses the generic
/// DomainAdapter/docs pipeline entirely — see domains.rs's module comment.
async fn build_presence_push_envelope(
    state: &AppState,
    domain_name: &str,
) -> Result<PushOutcome, (StatusCode, serde_json::Value)> {
    let status = state.kbd_control.status().map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let presence = crate::kbd_sync::KbdPresence {
        device: device_identity(),
        harness: "sovereign-sync".to_string(),
        session: "daemon".to_string(),
        observed_revision: status.revision,
        leader_term: None,
        lease_healthy: status.lease.is_some(),
    };
    state.presence.update(&presence).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let snapshot = state.presence.export_snapshot().map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;

    if state.p2p.is_none() {
        return Ok(PushOutcome::LocalOnly {
            snapshot_bytes: snapshot.len(),
        });
    }

    let signer = state
        .kbd_control
        .runtime()
        .device_signer()
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"error": error.to_string()}),
            )
        })?;
    let mut envelope = SyncEnvelope {
        schema_version: "1".into(),
        domain: domain_name.to_string(),
        identity: Some(status.project_id),
        payload: snapshot,
        signer_key_id: None,
        signature: None,
    };
    envelope.sign(&signer);
    let snapshot_bytes = envelope.payload.len();
    Ok(PushOutcome::Broadcast {
        envelope,
        snapshot_bytes,
    })
}

/// POST /api/v1/sync/push { "domain": "<name>" }
async fn sync_push(
    State(state): State<AppState>,
    Json(body): Json<SyncPushBody>,
) -> impl IntoResponse {
    match build_push_envelope(&state, &body.domain).await {
        Err((status, error_body)) => (status, Json(error_body)).into_response(),
        Ok(PushOutcome::LocalOnly { snapshot_bytes }) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "applied-locally-only",
                "domain": body.domain,
                "reason": "no P2P node in this mode (server mode has no gossip transport)",
                "snapshotBytes": snapshot_bytes
            })),
        )
            .into_response(),
        Ok(PushOutcome::Broadcast {
            envelope,
            snapshot_bytes,
        }) => {
            let envelope_bytes = match serde_json::to_vec(&envelope) {
                Ok(bytes) => bytes,
                Err(error) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": error.to_string()})),
                    )
                        .into_response()
                }
            };
            let bytes_transmitted = envelope_bytes.len();
            // `Broadcast` is only returned when `state.p2p.is_some()`.
            let p2p = state.p2p.as_ref().expect("Broadcast implies a P2P node");
            if let Err(error) = p2p.broadcast(Bytes::from(envelope_bytes)).await {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": error.to_string()})),
                )
                    .into_response();
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "broadcast",
                    "domain": body.domain,
                    "bytesTransmitted": bytes_transmitted,
                    "snapshotBytes": snapshot_bytes
                })),
            )
                .into_response()
        }
    }
}

async fn kbd_status(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    match state.kbd_control.status() {
        Ok(runtime) if runtime.revision > 0 && runtime.project_id == project_id => {
            (StatusCode::OK, Json(serde_json::json!(runtime))).into_response()
        }
        Ok(runtime) if runtime.revision > 0 => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"unknown KBD project"})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"kbd runtime is not initialized"})),
        )
            .into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_events(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    match (state.kbd_control.status(), state.kbd_control.events(1)) {
        (Ok(runtime), Ok(events)) if runtime.project_id == project_id => {
            (StatusCode::OK, Json(serde_json::json!(events))).into_response()
        }
        (Ok(_), Ok(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"unknown KBD project"})),
        )
            .into_response(),
        (Err(error), _) | (_, Err(error)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_diagnostics(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    match (state.kbd_control.status(), state.kbd_control.diagnostics()) {
        (Ok(runtime), Ok(diagnostics)) if runtime.project_id == project_id => {
            (StatusCode::OK, Json(diagnostics)).into_response()
        }
        (Ok(_), Ok(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"unknown KBD project"})),
        )
            .into_response(),
        (Err(error), _) | (_, Err(error)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_command(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    Json(envelope): Json<CommandEnvelope>,
) -> impl IntoResponse {
    if envelope.project_id != project_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "path projectId does not match command envelope"
            })),
        )
            .into_response();
    }
    match state.kbd_control.submit(envelope).await {
        Ok(committed) => (StatusCode::OK, Json(serde_json::json!(committed))).into_response(),
        Err(error) => (
            if error.kind() == std::io::ErrorKind::WouldBlock {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::CONFLICT
            },
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_event_stream(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match state.kbd_control.status() {
        Ok(runtime) if runtime.project_id == project_id => {}
        Ok(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error":"unknown KBD project"})),
            )
                .into_response()
        }
        Err(error) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    }
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let control = state.kbd_control.clone();
    let events = stream::unfold((control, last_event_id), |(control, revision)| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let current = control
            .events(revision.saturating_add(1))
            .unwrap_or_default();
        let fresh: Vec<_> = current
            .into_iter()
            .filter(|event| event.revision > revision)
            .collect();
        let next_revision = fresh.last().map_or(revision, |event| event.revision);
        let payload = serde_json::to_string(&fresh).unwrap_or_else(|_| "[]".into());
        Some((
            Ok::<_, Infallible>(
                SseEvent::default()
                    .event("kbd.events")
                    .id(next_revision.to_string())
                    .data(payload),
            ),
            (control, next_revision),
        ))
    });
    Sse::new(events)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

// ---------------------------------------------------------------------------
// Router builder
// ---------------------------------------------------------------------------

pub fn build_router(state: AppState) -> Router {
    // AG-UI state needs to be extracted separately.
    let ag_state = state.ag_ui.clone();

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/skills/search", get(skills_search))
        .route("/api/v1/sync/status", get(sync_status))
        .route("/api/v1/sync/peers", get(sync_peers))
        .route("/api/v1/sync/push", post(sync_push))
        .route("/api/v1/kbd/projects/{project_id}/status", get(kbd_status))
        .route("/api/v1/kbd/projects/{project_id}/events", get(kbd_events))
        .route(
            "/api/v1/kbd/projects/{project_id}/diagnostics",
            get(kbd_diagnostics),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/events/stream",
            get(kbd_event_stream),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/commands",
            post(kbd_command),
        )
        .route(
            "/api/v1/stream",
            post(ag_ui_stream).with_state(ag_state.clone()),
        )
        .route("/api/v1/stream/ping", get(ag_ui_ping))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, require_bearer))
}

// ---------------------------------------------------------------------------
// Incoming P2P sync messages
// ---------------------------------------------------------------------------

/// Handle one incoming P2P gossip message: decode as a [`SyncEnvelope`],
/// reject anything not syncable or with a mismatched identity, merge the
/// delta into the domain's CRDT snapshot, and persist the merged view via
/// the domain's adapter. `main.rs` spawns a loop calling this for every
/// message from the P2P node's receiver channel.
pub async fn handle_incoming_message(state: &AppState, payload: &[u8]) {
    let envelope: SyncEnvelope = match serde_json::from_slice(payload) {
        Ok(envelope) => envelope,
        Err(error) => {
            tracing::warn!("dropping unparseable P2P sync message: {error}");
            return;
        }
    };
    let domain = SyncDomain::new(envelope.domain.clone());
    let syncable = {
        let mut manifest = state.manifest.write().await;
        domains::ensure_registered(&mut manifest, &domain);
        manifest.is_syncable(&domain)
    };
    if !syncable {
        tracing::warn!(
            "dropping sync message for non-syncable domain {}",
            envelope.domain
        );
        return;
    }

    let family = domains::domain_family(&domain).to_string();

    if family == "kbd-control" {
        return import_presence_message(state, &envelope).await;
    }

    // Public domains (skill-index) carry no meaningful identity scope.
    // Trusted domains must match this node's own identity for that family,
    // rejecting cross-project/learner payloads per data-scope.md.
    if family != "skill-index" {
        let local_identity = match family.as_str() {
            "learner-model" => Some(default_learner_id()),
            _ => None,
        };
        if local_identity.is_none() || envelope.identity != local_identity {
            tracing::warn!(
                "dropping sync message for {} — identity mismatch (local={:?}, remote={:?})",
                envelope.domain,
                local_identity,
                envelope.identity
            );
            return;
        }
    }

    let Some(adapter) = state.adapters.get(&family) else {
        tracing::warn!("dropping sync message — no adapter for family {family}");
        return;
    };

    let crdt = LoroAdapter;
    let merged_json = {
        let mut docs = state.docs.lock().expect("docs mutex poisoned");
        let existing = docs.get(&domain).cloned().unwrap_or_else(|| crdt.new_doc());
        let merged = match crdt.merge(&existing, &envelope.payload) {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(
                    "failed to merge incoming delta for {}: {error}",
                    envelope.domain
                );
                return;
            }
        };
        docs.insert(domain.clone(), merged.clone());
        match crdt.to_json(&merged) {
            Ok(json) => json,
            Err(error) => {
                tracing::warn!(
                    "failed to decode merged doc for {}: {error}",
                    envelope.domain
                );
                return;
            }
        }
    };

    if let Err(error) = adapter.import_json(merged_json).await {
        tracing::warn!("failed to persist synced {}: {error}", envelope.domain);
    }
}

/// `kbd-control` receive: project-identity check first (cheap, coarse), then
/// real peer authentication — the envelope's claimed `signer_key_id` must
/// resolve to an `Active` device in this node's own (already-replicated)
/// `KbdStateV2.devices`, and the signature must verify against that device's
/// public key. `peer_authorized` is only true when both hold; `import_authenticated`
/// enforces it again independently (defense in depth), so a bug here fails
/// closed rather than open.
async fn import_presence_message(state: &AppState, envelope: &SyncEnvelope) {
    let Ok(status) = state.kbd_control.status() else {
        tracing::warn!("dropping kbd-control message — local runtime status unavailable");
        return;
    };
    let local_identity = Some(status.project_id);
    if envelope.identity != local_identity {
        tracing::warn!(
            "dropping kbd-control message — project identity mismatch (local={:?}, remote={:?})",
            local_identity,
            envelope.identity
        );
        return;
    }

    let peer_authorized = presence_peer_is_authorized(&status.devices, envelope);

    if let Err(error) = state
        .presence
        .import_authenticated(&envelope.payload, peer_authorized)
    {
        tracing::warn!("dropping kbd-control presence message: {error}");
    }
}

/// The actual authorization decision, factored out as a pure function so it
/// can be unit-tested directly against hand-built `DeviceRecord`/`SyncEnvelope`
/// fixtures without standing up a live `AppState`/daemon. True only when the
/// envelope's claimed signer resolves to an `Active` enrolled device AND the
/// signature verifies against that device's own public key — an unknown
/// signer, a revoked device, or a tampered/forged signature all fail closed.
fn presence_peer_is_authorized(
    devices: &std::collections::BTreeMap<String, kbd_runtime::DeviceRecord>,
    envelope: &SyncEnvelope,
) -> bool {
    envelope
        .signer_key_id
        .as_deref()
        .and_then(|signer_key_id| devices.get(signer_key_id))
        .filter(|device| device.status == kbd_runtime::DeviceStatus::Active)
        .is_some_and(|device| envelope.verify(&device.public_key))
}

// ---------------------------------------------------------------------------
// Server entry point
// ---------------------------------------------------------------------------

pub async fn serve(port: u16, skills_dir: &Path) -> anyhow::Result<()> {
    let state = AppState::try_new(skills_dir, None).await?;
    serve_with_state(port, state).await
}

/// Serve using a caller-constructed `AppState` — lets `main.rs` hold a clone
/// of the same state (e.g. to spawn the incoming-P2P-message consumer with
/// access to the same `docs`/`manifest`/`adapters`) before the server starts.
pub async fn serve_with_state(port: u16, state: AppState) -> anyhow::Result<()> {
    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("sovereign-sync REST API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod presence_auth_tests {
    use super::{presence_peer_is_authorized, SyncEnvelope};
    use kbd_runtime::{DeviceRecord, DeviceSigner, DeviceStatus};
    use std::collections::BTreeMap;

    fn envelope() -> SyncEnvelope {
        SyncEnvelope {
            schema_version: "1".into(),
            domain: "kbd-control:project-a".into(),
            identity: Some("project-a".into()),
            payload: b"presence snapshot bytes".to_vec(),
            signer_key_id: None,
            signature: None,
        }
    }

    fn enrolled(signer: &DeviceSigner, status: DeviceStatus) -> BTreeMap<String, DeviceRecord> {
        let mut devices = BTreeMap::new();
        devices.insert(
            signer.key_id().to_string(),
            DeviceRecord {
                device_id: "device-a".into(),
                key_id: signer.key_id().to_string(),
                public_key: signer.public_key().to_string(),
                status,
                enrolled_at_revision: 1,
                revoked_at_revision: None,
            },
        );
        devices
    }

    #[test]
    fn signed_envelope_verifies_against_the_signers_own_public_key() {
        let signer = DeviceSigner::generate();
        let mut env = envelope();
        env.sign(&signer);
        assert!(env.verify(signer.public_key()));
    }

    #[test]
    fn verify_fails_for_a_tampered_payload() {
        let signer = DeviceSigner::generate();
        let mut env = envelope();
        env.sign(&signer);
        env.payload = b"different bytes than what was signed".to_vec();
        assert!(!env.verify(signer.public_key()));
    }

    #[test]
    fn verify_fails_for_the_wrong_public_key() {
        let signer = DeviceSigner::generate();
        let other = DeviceSigner::generate();
        let mut env = envelope();
        env.sign(&signer);
        assert!(!env.verify(other.public_key()));
    }

    #[test]
    fn peer_is_authorized_for_an_active_enrolled_device_with_a_valid_signature() {
        let signer = DeviceSigner::generate();
        let devices = enrolled(&signer, DeviceStatus::Active);
        let mut env = envelope();
        env.sign(&signer);
        assert!(presence_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_when_unsigned() {
        let signer = DeviceSigner::generate();
        let devices = enrolled(&signer, DeviceStatus::Active);
        let env = envelope();
        assert!(!presence_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_for_an_unknown_signer() {
        let signer = DeviceSigner::generate();
        let devices = BTreeMap::new(); // signer never enrolled on this node
        let mut env = envelope();
        env.sign(&signer);
        assert!(!presence_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_for_a_revoked_device() {
        let signer = DeviceSigner::generate();
        let devices = enrolled(&signer, DeviceStatus::Revoked);
        let mut env = envelope();
        env.sign(&signer);
        assert!(!presence_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_when_the_signature_does_not_match_the_claimed_signer() {
        let signer = DeviceSigner::generate();
        let impostor = DeviceSigner::generate();
        // devices map has the real signer enrolled and active...
        let devices = enrolled(&signer, DeviceStatus::Active);
        // ...but the envelope claims to be from that signer while actually
        // being signed by a different key (forged signer_key_id).
        let mut env = envelope();
        env.sign(&impostor);
        env.signer_key_id = Some(signer.key_id().to_string());
        assert!(!presence_peer_is_authorized(&devices, &env));
    }
}
