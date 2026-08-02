/// Axum REST API for sovereign-sync daemon / server modes.
///
/// Routes:
///   GET  /health
///   GET  /ready
///   GET  /api/v1/skills/search?q=<query>
///   GET  /api/v1/sync/status
///   GET  /api/v1/sync/peers
///   POST /api/v1/sync/push     { "domain": "<name>" }
///   POST /api/v1/stream        (AG-UI SSE — delegates to ag_ui module)
///   GET  /api/v1/stream/ping   (AG-UI SSE ping)
use axum::{
    extract::{Path as AxumPath, Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event as SseEvent, KeepAlive},
        IntoResponse, Response, Sse,
    },
    routing::{any, get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::stream;
use kbd_runtime::{CommandKind, SignedCommandEnvelope};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock},
    time::Duration,
};
use storage_provider::{CrdtEngine, LoroAdapter, SyncDomain, SyncManifest};
use tokio::sync::RwLock as AsyncRwLock;
use tower::ServiceExt;
use tracing::info;

use crate::ag_ui::{ag_ui_events, ag_ui_ping, ag_ui_stream, AgUiEvent, AgUiState};
use crate::domains::{self, DomainAdapter, LearnerModelAdapter, SkillIndexAdapter, SyncEnvelope};
use crate::kbd_control::KbdProjectRouter;
use crate::kbd_single_writer::QuorumPolicy;
use crate::kbd_sync::{KbdAuthorityPayload, KbdPresenceDocument};
use crate::mcp_server::SkillIndex;
use crate::p2p::P2PNode;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    skill_index: Arc<SkillIndex>,
    ag_ui: AgUiState,
    kbd_projects: Arc<KbdProjectRouter>,
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
    /// `kbd-control` auxiliary presence — handled directly here, not through
    /// `adapters`/`docs` (see `domains.rs`'s module comment on why it isn't
    /// a `DomainAdapter`).
    presence: Arc<StdRwLock<BTreeMap<String, Arc<KbdPresenceDocument>>>>,
}

/// Hot-swappable application router used only during daemon startup. The
/// static liveness endpoint is available as soon as the socket is bound;
/// every stateful route fails closed with 503 until initialization installs
/// the full application router.
#[derive(Clone, Default)]
pub struct StartupGate {
    app: Arc<AsyncRwLock<Option<Router>>>,
}

impl StartupGate {
    pub async fn install(&self, state: AppState) {
        *self.app.write().await = Some(build_router(state));
    }
}

impl AppState {
    pub async fn new(skills_dir: &Path) -> Self {
        Self::try_new(skills_dir, None)
            .await
            .expect("cannot open KBD control plane")
    }

    pub async fn try_new(skills_dir: &Path, p2p: Option<Arc<P2PNode>>) -> anyhow::Result<Self> {
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_projects = Arc::new(KbdProjectRouter::open_registered(quorum).await?);
        if let Some(project_root) = discover_manifest_project_root(skills_dir) {
            kbd_projects.register_path(&project_root).await?;
        }
        let learner_model_dir = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".prometheus")
            .join("learn")
            .join("learner-model");
        Self::from_project_router(skills_dir, kbd_projects, learner_model_dir, p2p)
    }

    pub async fn try_new_at(
        skills_dir: &Path,
        project_root: &Path,
        data_root: &Path,
        p2p: Option<Arc<P2PNode>>,
    ) -> anyhow::Result<Self> {
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_projects = Arc::new(
            KbdProjectRouter::open_with_project_at(project_root, data_root, quorum).await?,
        );
        Self::from_project_router(
            skills_dir,
            kbd_projects,
            learner_model_dir_at(data_root),
            p2p,
        )
    }

    fn from_project_router(
        skills_dir: &Path,
        kbd_projects: Arc<KbdProjectRouter>,
        learner_model_dir: PathBuf,
        p2p: Option<Arc<P2PNode>>,
    ) -> anyhow::Result<Self> {
        let skill_index = Arc::new(SkillIndex::load_from_dir(skills_dir));
        let presence = kbd_projects
            .project_ids()
            .into_iter()
            .map(|project_id| {
                let document = Arc::new(KbdPresenceDocument::new(project_id.clone()));
                (project_id, document)
            })
            .collect();

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
            kbd_projects,
            manifest: Arc::new(AsyncRwLock::new(SyncManifest::new())),
            docs: Arc::new(StdMutex::new(HashMap::new())),
            p2p,
            adapters: Arc::new(adapters),
            presence: Arc::new(StdRwLock::new(presence)),
        })
    }

    /// Inspection accessor for tests — not part of the wire API.
    pub fn skill_index(&self) -> &SkillIndex {
        &self.skill_index
    }

    /// Inspection accessor for tests — not part of the wire API.
    pub fn presence(&self) -> Option<Arc<KbdPresenceDocument>> {
        self.presence
            .read()
            .expect("presence map lock poisoned")
            .values()
            .next()
            .cloned()
    }

    fn presence_for(&self, project_id: &str) -> Option<Arc<KbdPresenceDocument>> {
        self.presence
            .read()
            .expect("presence map lock poisoned")
            .get(project_id)
            .cloned()
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

fn discover_manifest_project_root(skills_dir: &Path) -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(root) = cwd
            .ancestors()
            .find(|candidate| candidate.join(".prometheus/project.json").is_file())
        {
            return Some(root.to_path_buf());
        }
    }
    skills_dir
        .ancestors()
        .find(|candidate| candidate.join(".prometheus/project.json").is_file())
        .map(Path::to_path_buf)
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// GET /health — process liveness only; deliberately touches no state.
///
/// An earlier version probed persistent state from this handler. `status()` is a
/// SYNCHRONOUS read, so each probe parked a tokio worker on a file lock; under
/// concurrent polling every worker parked and the daemon accepted connections
/// it could never answer. It looked alive to `lsof` and hung every client.
///
/// A liveness endpoint must not be able to hang the server it reports on: it
/// fails exactly when it is being relied upon. Store reachability belongs on a
/// separate readiness route that is allowed to be slow.
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "sovereign-sync",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// GET /ready — asynchronous journal reachability and replay validation.
async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let mut projects = Vec::new();
    let mut all_ready = true;
    let mut checks = tokio::task::JoinSet::new();
    for project_id in state.kbd_projects.project_ids() {
        match state.kbd_projects.control(&project_id) {
            Ok(control) => {
                checks.spawn(async move {
                    let result = tokio::time::timeout(
                        Duration::from_millis(400),
                        control.authority_status_async(),
                    )
                    .await;
                    (project_id, result)
                });
            }
            Err(error) => {
                all_ready = false;
                projects.push(serde_json::json!({
                    "projectId": project_id,
                    "ready": false,
                    "error": error.to_string()
                }));
            }
        }
    }
    while let Some(check) = checks.join_next().await {
        match check {
            Ok((project_id, Ok(Ok(runtime)))) => projects.push(serde_json::json!({
                "projectId": project_id,
                "ready": true,
                "revision": runtime.revision
            })),
            Ok((project_id, Ok(Err(error)))) => {
                all_ready = false;
                projects.push(serde_json::json!({
                    "projectId": project_id,
                    "ready": false,
                    "error": error.to_string()
                }));
            }
            Ok((project_id, Err(_))) => {
                all_ready = false;
                projects.push(serde_json::json!({
                    "projectId": project_id,
                    "ready": false,
                    "error": "authority replay exceeded 400 ms"
                }));
            }
            Err(error) => {
                all_ready = false;
                projects.push(serde_json::json!({
                    "projectId": null,
                    "ready": false,
                    "error": format!("readiness task failed: {error}")
                }));
            }
        }
    }
    projects.sort_by(|left, right| left["projectId"].as_str().cmp(&right["projectId"].as_str()));
    let status = if all_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(serde_json::json!({
            "status": if all_ready { "ready" } else { "not_ready" },
            "projectCount": projects.len(),
            "projects": projects
        })),
    )
        .into_response()
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
        return build_kbd_authority_push_envelope(state, domain_name).await;
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
    // see build_kbd_authority_push_envelope).
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

/// `kbd-control` push: export the persisted project's complete Loro update
/// set, attach auxiliary presence, sign the envelope with the project device
/// identity, and broadcast. Replica journals never leave the machine.
async fn build_kbd_authority_push_envelope(
    state: &AppState,
    domain_name: &str,
) -> Result<PushOutcome, (StatusCode, serde_json::Value)> {
    let project_id = domain_name
        .strip_prefix("kbd-control:")
        .filter(|project_id| !project_id.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                serde_json::json!({"error":"kbd-control domain requires a project ID"}),
            )
        })?;
    let control = state.kbd_projects.control(project_id).map_err(|error| {
        (
            if error.kind() == std::io::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::CONFLICT
            },
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let status = control.status_async().await.map_err(|error| {
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
    };
    let presence_document = state.presence_for(project_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            serde_json::json!({"error":"project presence document is unavailable"}),
        )
    })?;
    presence_document.update(&presence).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let presence = presence_document.entries().map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let project_updates = control.export_project_updates().await.map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let payload =
        KbdAuthorityPayload::encode(project_id, project_updates, presence).map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"error": error.to_string()}),
            )
        })?;

    if state.p2p.is_none() {
        return Ok(PushOutcome::LocalOnly {
            snapshot_bytes: payload.len(),
        });
    }

    let signer = control.runtime().device_signer().map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error.to_string()}),
        )
    })?;
    let mut envelope = SyncEnvelope {
        schema_version: "2".into(),
        domain: domain_name.to_string(),
        identity: Some(status.project_id),
        payload,
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
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.status_async().await {
        Ok(runtime) if runtime.revision > 0 => {
            (StatusCode::OK, Json(serde_json::json!(runtime))).into_response()
        }
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

async fn kbd_projects(State(state): State<AppState>) -> impl IntoResponse {
    match state.kbd_projects.routes() {
        Ok(routes) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schemaVersion": "1",
                "projects": routes
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterProjectBody {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoptProjectBody {
    path: PathBuf,
    into_project_id: String,
    #[serde(default)]
    apply: bool,
}

async fn kbd_register_project(
    State(state): State<AppState>,
    Json(body): Json<RegisterProjectBody>,
) -> impl IntoResponse {
    match state.kbd_projects.register_path(&body.path).await {
        Ok(outcome) => (StatusCode::OK, Json(serde_json::json!(outcome))).into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_adopt_project(
    State(state): State<AppState>,
    Json(body): Json<AdoptProjectBody>,
) -> impl IntoResponse {
    let registry = state.kbd_projects.registry().clone();
    let path = body.path.clone();
    let project_id = body.into_project_id.clone();
    let apply = body.apply;
    let result = tokio::task::spawn_blocking(move || {
        if apply {
            registry
                .apply_adoption(&path, &project_id)
                .and_then(|outcome| serde_json::to_value(outcome).map_err(Into::into))
        } else {
            registry
                .plan_adoption(&path, &project_id)
                .and_then(|plan| serde_json::to_value(plan).map_err(Into::into))
        }
    })
    .await;
    match result {
        Ok(Ok(outcome)) => {
            if apply {
                if let Err(error) = state.kbd_projects.reload().await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error":error.to_string()})),
                    )
                        .into_response();
                }
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "dryRun": !apply,
                    "outcome": outcome
                })),
            )
                .into_response()
        }
        Ok(Err(error)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":format!("adoption task failed: {error}")})),
        )
            .into_response(),
    }
}

async fn kbd_submodules(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.status_async().await {
        Ok(runtime) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "projectId": project_id,
                "pins": runtime.submodule_pins,
                "replicaView": runtime.replica_view
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_replicas(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    match state.kbd_projects.registry().lookup_project(&project_id) {
        Ok(replicas) if !replicas.is_empty() => (
            StatusCode::OK,
            Json(serde_json::json!({
                "projectId": project_id,
                "replicas": replicas.into_iter().map(|(path, replica)| {
                    serde_json::json!({"path":path, "registration":replica})
                }).collect::<Vec<_>>()
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"unknown KBD project"})),
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
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    let (runtime, events) = tokio::join!(control.status_async(), control.events_async(1));
    match (runtime, events) {
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

async fn kbd_audit_export(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.signed_audit_jsonl().await {
        Ok(bytes) => (
            StatusCode::OK,
            [("content-type", "application/x-ndjson")],
            bytes,
        )
            .into_response(),
        Err(error) => (
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
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.diagnostics_async().await {
        Ok(diagnostics)
            if diagnostics["runtime"]["projectId"].as_str() == Some(project_id.as_str()) =>
        {
            (StatusCode::OK, Json(diagnostics)).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"unknown KBD project"})),
        )
            .into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_conflicts(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.status_async().await {
        Ok(runtime) => (StatusCode::OK, Json(serde_json::json!(runtime.conflicts))).into_response(),
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_command(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    Json(signed): Json<SignedCommandEnvelope>,
) -> impl IntoResponse {
    let envelope = &signed.command;
    if envelope.project_id != project_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "path projectId does not match command envelope"
            })),
        )
            .into_response();
    }
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    let command_id = signed.command.command_id.clone();
    match control.submit_signed(signed).await {
        Ok(committed) => {
            if let Ok(events) = control.events_async(0).await {
                if let Some(event) = events
                    .iter()
                    .find(|event| event.command_id.as_deref() == Some(&command_id))
                {
                    publish_kbd_event(&state.ag_ui, event, &committed.result.state);
                }
            }
            (StatusCode::OK, Json(serde_json::json!(committed))).into_response()
        }
        Err(error) => (
            match error.kind() {
                std::io::ErrorKind::WouldBlock => StatusCode::SERVICE_UNAVAILABLE,
                std::io::ErrorKind::PermissionDenied => StatusCode::UNAUTHORIZED,
                std::io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
                _ => StatusCode::CONFLICT,
            },
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

fn publish_kbd_event(
    ag_ui: &AgUiState,
    event: &kbd_runtime::Event,
    state: &kbd_runtime::KbdStateV2,
) {
    ag_ui.publish(AgUiEvent::EventAppended {
        project_id: event.project_id.clone(),
        event_id: event.event_id.clone(),
        replica_id: event.replica_id.clone(),
        lamport: event.lamport,
        frontier: state.frontier.clone(),
    });
    if matches!(event.kind, kbd_runtime::EventKind::ClaimAcquired { .. }) {
        if let Some(claim) = state
            .claims
            .values()
            .find(|claim| claim.acquired_event_id == event.event_id)
        {
            ag_ui.publish(AgUiEvent::ClaimAcquired {
                project_id: event.project_id.clone(),
                claim: claim.clone(),
            });
        }
    }
    for conflict in state.conflicts.values().filter(|conflict| {
        conflict
            .candidates
            .iter()
            .any(|candidate| candidate.event_id == event.event_id)
    }) {
        match conflict.kind {
            kbd_runtime::ConflictKind::Claim => ag_ui.publish(AgUiEvent::ClaimConflict {
                project_id: event.project_id.clone(),
                conflict: conflict.clone(),
            }),
            kbd_runtime::ConflictKind::Lifecycle
            | kbd_runtime::ConflictKind::ActivePath
            | kbd_runtime::ConflictKind::Completion => {
                ag_ui.publish(AgUiEvent::SingletonViolation {
                    project_id: event.project_id.clone(),
                    conflict: conflict.clone(),
                })
            }
            _ => {}
        }
    }
}

async fn kbd_resolve_conflict(
    State(state): State<AppState>,
    AxumPath((project_id, conflict_id)): AxumPath<(String, String)>,
    Json(signed): Json<SignedCommandEnvelope>,
) -> impl IntoResponse {
    match &signed.command.command {
        CommandKind::ConflictResolve {
            conflict_id: supplied,
            ..
        } if supplied == &conflict_id => {}
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error":"resolution path requires a matching conflict_resolve command"
                })),
            )
                .into_response()
        }
    }
    kbd_command(State(state), AxumPath(project_id), Json(signed))
        .await
        .into_response()
}

async fn kbd_claims(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.status_async().await {
        Ok(runtime) => {
            let conflicts = runtime
                .conflicts
                .values()
                .filter(|conflict| conflict.kind == kbd_runtime::ConflictKind::Claim)
                .collect::<Vec<_>>();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "claims": runtime.claims,
                    "conflicts": conflicts,
                    "frontier": runtime.frontier
                })),
            )
                .into_response()
        }
        Err(error) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":error.to_string()})),
        )
            .into_response(),
    }
}

async fn kbd_claim_command(
    state: AppState,
    project_id: String,
    expected_action: &'static str,
    signed: SignedCommandEnvelope,
) -> axum::response::Response {
    let matches = matches!(
        (&signed.command.command, expected_action),
        (CommandKind::ClaimAcquire { .. }, "acquire")
            | (CommandKind::ClaimRenew { .. }, "renew")
            | (CommandKind::ClaimRelease { .. }, "release")
    );
    if !matches {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":format!(
                "claim endpoint requires a claim_{expected_action} command"
            )})),
        )
            .into_response();
    }
    kbd_command(State(state), AxumPath(project_id), Json(signed))
        .await
        .into_response()
}

async fn kbd_claim_acquire(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    Json(signed): Json<SignedCommandEnvelope>,
) -> impl IntoResponse {
    kbd_claim_command(state, project_id, "acquire", signed).await
}

async fn kbd_claim_renew(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    Json(signed): Json<SignedCommandEnvelope>,
) -> impl IntoResponse {
    kbd_claim_command(state, project_id, "renew", signed).await
}

async fn kbd_claim_release(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    Json(signed): Json<SignedCommandEnvelope>,
) -> impl IntoResponse {
    kbd_claim_command(state, project_id, "release", signed).await
}

async fn kbd_event_stream(
    State(state): State<AppState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let control = match state.kbd_projects.control(&project_id) {
        Ok(control) => control,
        Err(error) => {
            return (
                if error.kind() == std::io::ErrorKind::NotFound {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::CONFLICT
                },
                Json(serde_json::json!({"error":error.to_string()})),
            )
                .into_response()
        }
    };
    match control.status_async().await {
        Ok(_) => {}
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
    let events = stream::unfold((control, last_event_id), |(control, revision)| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let fresh = control.events(revision).unwrap_or_default();
        let next_revision = revision.saturating_add(fresh.len() as u64);
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
        .route("/ready", get(ready))
        .route("/api/v1/skills/search", get(skills_search))
        .route("/api/v1/sync/status", get(sync_status))
        .route("/api/v1/sync/peers", get(sync_peers))
        .route("/api/v1/sync/push", post(sync_push))
        .route("/api/v1/kbd/projects", get(kbd_projects))
        .route("/api/v1/kbd/projects/register", post(kbd_register_project))
        .route("/api/v1/kbd/projects/adopt", post(kbd_adopt_project))
        .route(
            "/api/v1/kbd/projects/{project_id}/replicas",
            get(kbd_replicas),
        )
        .route("/api/v1/kbd/projects/{project_id}/status", get(kbd_status))
        .route("/api/v1/kbd/projects/{project_id}/events", get(kbd_events))
        .route(
            "/api/v1/kbd/projects/{project_id}/audit",
            get(kbd_audit_export),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/submodules",
            get(kbd_submodules),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/diagnostics",
            get(kbd_diagnostics),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/conflicts",
            get(kbd_conflicts),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/conflicts/{conflict_id}/resolve",
            post(kbd_resolve_conflict),
        )
        .route("/api/v1/kbd/projects/{project_id}/claims", get(kbd_claims))
        .route(
            "/api/v1/kbd/projects/{project_id}/claims/acquire",
            post(kbd_claim_acquire),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/claims/renew",
            post(kbd_claim_renew),
        )
        .route(
            "/api/v1/kbd/projects/{project_id}/claims/release",
            post(kbd_claim_release),
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
        .route(
            "/api/v1/events",
            get(ag_ui_events).with_state(ag_state.clone()),
        )
        .with_state(state.clone())
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
        return import_kbd_authority_message(state, &envelope).await;
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
async fn import_kbd_authority_message(state: &AppState, envelope: &SyncEnvelope) {
    let Some(project_id) = envelope.identity.as_deref() else {
        tracing::warn!("dropping kbd-control message — missing project identity");
        return;
    };
    let Ok(control) = state.kbd_projects.control(project_id) else {
        tracing::warn!("dropping kbd-control message — project is not registered");
        return;
    };
    let Ok(status) = control.status_async().await else {
        tracing::warn!("dropping kbd-control message — local runtime status unavailable");
        return;
    };
    let local_identity = Some(status.project_id.clone());
    if envelope.identity != local_identity {
        tracing::warn!(
            "dropping kbd-control message — project identity mismatch (local={:?}, remote={:?})",
            local_identity,
            envelope.identity
        );
        return;
    }

    let peer_authorized = kbd_peer_is_authorized(&status.devices, envelope);
    if !peer_authorized {
        tracing::warn!("dropping kbd-control authority message — signer is not authorized");
        return;
    }

    let payload = match KbdAuthorityPayload::decode(&envelope.payload) {
        Ok(payload) if payload.project_id == project_id => payload,
        Ok(payload) => {
            tracing::warn!(
                "dropping kbd-control authority message — payload project {} does not match {}",
                payload.project_id,
                project_id
            );
            return;
        }
        Err(error) => {
            tracing::warn!("dropping invalid kbd-control authority payload: {error}");
            return;
        }
    };
    let before = control
        .events_async(0)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|event| event.event_id)
        .collect::<std::collections::HashSet<_>>();
    let merged = match control
        .import_project_updates(payload.project_updates)
        .await
    {
        Ok((_, merged)) => merged,
        Err(error) => {
            tracing::warn!("dropping invalid kbd-control project updates: {error}");
            return;
        }
    };
    if let Ok(events) = control.events_async(0).await {
        for event in events
            .iter()
            .filter(|event| !before.contains(&event.event_id))
        {
            publish_kbd_event(&state.ag_ui, event, &merged);
        }
    }

    let Some(presence) = state.presence_for(project_id) else {
        tracing::warn!("dropping kbd-control message — presence document is unavailable");
        return;
    };
    for entry in payload.presence {
        if let Err(error) = presence.update(&entry) {
            tracing::warn!("failed to merge kbd-control presence: {error}");
        }
    }
}

/// The actual authorization decision, factored out as a pure function so it
/// can be unit-tested directly against hand-built `DeviceRecord`/`SyncEnvelope`
/// fixtures without standing up a live `AppState`/daemon. True only when the
/// envelope's claimed signer resolves to an `Active` enrolled device AND the
/// signature verifies against that device's own public key — an unknown
/// signer, a revoked device, or a tampered/forged signature all fail closed.
fn kbd_peer_is_authorized(
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

async fn startup_dispatch(State(gate): State<StartupGate>, request: Request) -> Response {
    let app = gate.app.read().await.clone();
    let Some(app) = app else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "initializing",
                "error": "sovereign-sync state is not ready"
            })),
        )
            .into_response();
    };
    match app.oneshot(request).await {
        Ok(response) => response,
        Err(error) => match error {},
    }
}

/// Build the liveness-only router installed while persistent state is opening.
/// `/health` remains static and store-independent; all other paths return 503
/// until `StartupGate::install` atomically exposes the full application.
pub fn build_startup_router() -> (Router, StartupGate) {
    let gate = StartupGate::default();
    let app = Router::new()
        .route("/health", get(health))
        .fallback(any(startup_dispatch))
        .with_state(gate.clone());
    (app, gate)
}

pub async fn serve(port: u16, skills_dir: &Path) -> anyhow::Result<()> {
    let listener = bind_loopback(port).await?;
    let (startup_app, gate) = build_startup_router();
    let server = tokio::spawn(async move { axum::serve(listener, startup_app).await });
    let state = AppState::try_new(skills_dir, None).await?;
    gate.install(state).await;
    server.await??;
    Ok(())
}

/// Serve using a caller-constructed `AppState` — lets `main.rs` hold a clone
/// of the same state (e.g. to spawn the incoming-P2P-message consumer with
/// access to the same `docs`/`manifest`/`adapters`) before the server starts.
pub async fn serve_with_state(port: u16, state: AppState) -> anyhow::Result<()> {
    let listener = bind_loopback(port).await?;
    serve_with_listener(listener, state).await
}

/// Acquire the unauthenticated loopback listener before opening KBD project
/// state or joining P2P gossip. Registry reconciliation can be deliberately
/// expensive; it must not leave the control-plane port unbound while launchd
/// considers the process alive.
pub async fn bind_loopback(port: u16) -> anyhow::Result<tokio::net::TcpListener> {
    // LOOPBACK ONLY — and this is load-bearing, not incidental.
    //
    // There is no authentication on this server. That is deliberate: the
    // previous bearer-token scheme resolved different runtime roots for CLI
    // and daemon processes and produced mismatched secrets. It returned false
    // 401s without protecting a service reachable only by processes that
    // already have local code execution. KBD mutation POSTs now have their own
    // device-signature authorization in addition to this transport boundary.
    //
    // IF YOU CHANGE THIS ADDRESS, YOU MUST ADD AUTHENTICATION FIRST. Binding
    // anything other than 127.0.0.1 exposes unauthenticated control-plane
    // writes — phase creation, command submission — to the
    // network. Design it against the threat model you actually have at that
    // point; do not resurrect the token scheme deleted here.
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(
        "sovereign-sync REST API bound on http://{} (no auth: loopback only)",
        listener.local_addr()?
    );
    Ok(listener)
}

/// Serve a fully initialized application on an already-acquired listener.
pub async fn serve_with_listener(
    listener: tokio::net::TcpListener,
    state: AppState,
) -> anyhow::Result<()> {
    let app = build_router(state);

    info!(
        "sovereign-sync REST API ready on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod presence_auth_tests {
    use super::{
        build_push_envelope, handle_incoming_message, kbd_peer_is_authorized, AppState,
        PushOutcome, SyncEnvelope,
    };
    use crate::p2p::P2PNode;
    use bytes::Bytes;
    use iroh::address_lookup::MemoryLookup;
    use kbd_runtime::{
        Actor, ActorKind, ClaimMode, CommandEnvelope, CommandKind, DeviceRecord, DeviceSigner,
        DeviceStatus, Runtime,
    };
    use std::{collections::BTreeMap, fs, sync::Arc, time::Duration};
    use tempfile::tempdir;
    use uuid::Uuid;

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
        assert!(kbd_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_when_unsigned() {
        let signer = DeviceSigner::generate();
        let devices = enrolled(&signer, DeviceStatus::Active);
        let env = envelope();
        assert!(!kbd_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_for_an_unknown_signer() {
        let signer = DeviceSigner::generate();
        let devices = BTreeMap::new(); // signer never enrolled on this node
        let mut env = envelope();
        env.sign(&signer);
        assert!(!kbd_peer_is_authorized(&devices, &env));
    }

    #[test]
    fn peer_is_not_authorized_for_a_revoked_device() {
        let signer = DeviceSigner::generate();
        let devices = enrolled(&signer, DeviceStatus::Revoked);
        let mut env = envelope();
        env.sign(&signer);
        assert!(!kbd_peer_is_authorized(&devices, &env));
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
        assert!(!kbd_peer_is_authorized(&devices, &env));
    }

    #[tokio::test]
    async fn two_iroh_peers_exchange_a_signed_claim_authority_update() {
        let fixture = tempdir().unwrap();
        let project_a = fixture.path().join("project-a");
        let project_b = fixture.path().join("project-b");
        let data_a = fixture.path().join("data-a");
        let data_b = fixture.path().join("data-b");
        let skills = fixture.path().join("skills");
        let project_id = Uuid::new_v4().to_string();
        for project in [&project_a, &project_b] {
            fs::create_dir_all(project.join(".prometheus")).unwrap();
            fs::write(
                project.join(".prometheus/project.json"),
                serde_json::json!({
                    "schemaVersion":"1",
                    "projectId":project_id,
                    "repositoryFingerprint":"sha256:iroh-claim-test"
                })
                .to_string(),
            )
            .unwrap();
        }
        fs::create_dir_all(&skills).unwrap();

        let runtime_a = Runtime::open_canonical_at(&project_a, &data_a).unwrap();
        let initialized = runtime_a
            .initialize(
                project_id.clone(),
                "run-a",
                Actor::operator("operator", "test"),
            )
            .unwrap();
        let runtime_b = Runtime::open_canonical_at(&project_b, &data_b).unwrap();
        runtime_b
            .import_project_updates(&runtime_a.export_project_updates().unwrap())
            .unwrap();

        let lookup = MemoryLookup::new();
        let (p2p_a, _incoming_a) = P2PNode::new_with_memory_lookup(&[42; 32], lookup.clone())
            .await
            .unwrap();
        let (p2p_b, mut incoming_b) = P2PNode::new_with_memory_lookup(&[42; 32], lookup)
            .await
            .unwrap();
        let node_a_id = p2p_a.node_id();
        p2p_a.start(Vec::new()).await.unwrap();
        p2p_b.start(vec![node_a_id]).await.unwrap();
        let p2p_a = Arc::new(p2p_a);
        let p2p_b = Arc::new(p2p_b);

        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if !p2p_a.neighbors().await.is_empty() && !p2p_b.neighbors().await.is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("local iroh peers should become neighbors");

        let state_a = AppState::try_new_at(&skills, &project_a, &data_a, Some(Arc::clone(&p2p_a)))
            .await
            .unwrap();
        let state_b = AppState::try_new_at(&skills, &project_b, &data_b, Some(p2p_b))
            .await
            .unwrap();
        let actor = Actor {
            kind: ActorKind::Harness,
            id: "holder-a".into(),
            device: "device-a".into(),
            harness: "test".into(),
            session: "session-a".into(),
        };
        runtime_a
            .execute_command(CommandEnvelope {
                schema_version: "2".into(),
                project_id: project_id.clone(),
                run_id: initialized.run_id,
                command_id: "claim-over-iroh".into(),
                frontier: Some(initialized.frontier),
                expected_revision: 0,
                actor: actor.clone(),
                command: CommandKind::ClaimAcquire {
                    scope: "phase:iroh".into(),
                    mode: ClaimMode::Exclusive,
                    ttl_seconds: 300,
                    holder_id: actor.id,
                },
            })
            .unwrap();
        let outcome = build_push_envelope(&state_a, &format!("kbd-control:{project_id}"))
            .await
            .unwrap();
        let envelope = match outcome {
            PushOutcome::Broadcast { envelope, .. } => envelope,
            PushOutcome::LocalOnly { .. } => panic!("node A has an active P2P transport"),
        };
        p2p_a
            .broadcast(Bytes::from(serde_json::to_vec(&envelope).unwrap()))
            .await
            .unwrap();
        let received = tokio::time::timeout(Duration::from_secs(5), incoming_b.recv())
            .await
            .expect("peer B should receive the claim envelope")
            .expect("peer B receiver should stay open");
        handle_incoming_message(&state_b, &received.payload).await;

        let converged = runtime_b.replay().unwrap();
        assert!(converged
            .claims
            .values()
            .any(|claim| claim.scope == "phase:iroh" && claim.holder_id == "holder-a"));
    }
}
