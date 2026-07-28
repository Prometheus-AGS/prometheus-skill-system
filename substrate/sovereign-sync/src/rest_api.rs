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
use futures::stream;
use kbd_runtime::CommandEnvelope;
use serde::Deserialize;
use std::{
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tracing::info;

use crate::ag_ui::{ag_ui_ping, ag_ui_stream, AgUiState};
use crate::kbd_control::KbdControlPlane;
use crate::kbd_raft::QuorumPolicy;
use crate::mcp_server::SkillIndex;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    skill_index: Arc<SkillIndex>,
    ag_ui: AgUiState,
    kbd_control: Arc<KbdControlPlane>,
    bearer_token: Arc<str>,
}

impl AppState {
    pub async fn new(skills_dir: &Path) -> Self {
        Self::try_new(skills_dir)
            .await
            .expect("cannot open KBD control plane")
    }

    pub async fn try_new(skills_dir: &Path) -> anyhow::Result<Self> {
        let project_root = discover_project_root(skills_dir);
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_control = Arc::new(KbdControlPlane::open(&project_root, quorum).await?);
        Self::from_control_plane(skills_dir, kbd_control)
    }

    pub async fn try_new_at(
        skills_dir: &Path,
        project_root: &Path,
        data_root: &Path,
    ) -> anyhow::Result<Self> {
        let quorum = QuorumPolicy::new(1, [1])?;
        let kbd_control =
            Arc::new(KbdControlPlane::open_at(project_root, data_root, quorum).await?);
        Self::from_control_plane(skills_dir, kbd_control)
    }

    fn from_control_plane(
        skills_dir: &Path,
        kbd_control: Arc<KbdControlPlane>,
    ) -> anyhow::Result<Self> {
        let bearer_token: Arc<str> = kbd_control.runtime().control_token()?.into();
        Ok(Self {
            skill_index: Arc::new(SkillIndex::load_from_dir(skills_dir)),
            ag_ui: AgUiState::new(),
            kbd_control,
            bearer_token,
        })
    }

    pub fn bearer_token(&self) -> &str {
        &self.bearer_token
    }
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
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "sovereign-sync",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn require_bearer(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.uri().path() == "/health" {
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
async fn sync_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "node_state": "idle",
        "peers": [],
        "domains": {
            "kbd-orchestrator": { "privacy": "sync_encrypted_only", "peers": 0 },
            "open-spec":        { "privacy": "sync_encrypted_only", "peers": 0 },
            "surreal-memory":   { "privacy": "local_only",          "peers": 0 },
            "learner-model":    { "privacy": "sync_encrypted_only", "peers": 0 }
        }
    }))
}

/// GET /api/v1/sync/peers
async fn sync_peers() -> impl IntoResponse {
    Json(serde_json::json!({
        "peers": []
    }))
}

#[derive(Debug, Deserialize)]
struct SyncPushBody {
    domain: String,
}

/// POST /api/v1/sync/push { "domain": "<name>" }
async fn sync_push(Json(body): Json<SyncPushBody>) -> impl IntoResponse {
    // Full CRDT push wired in change-sync-015.
    Json(serde_json::json!({
        "status": "queued",
        "domain": body.domain
    }))
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
// Server entry point
// ---------------------------------------------------------------------------

pub async fn serve(port: u16, skills_dir: &Path) -> anyhow::Result<()> {
    let state = AppState::try_new(skills_dir).await?;
    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("sovereign-sync REST API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
