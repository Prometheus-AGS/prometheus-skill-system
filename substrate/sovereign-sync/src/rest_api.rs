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
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{net::SocketAddr, path::Path, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::ag_ui::{ag_ui_ping, ag_ui_stream, AgUiState};
use crate::mcp_server::SkillIndex;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    skill_index: Arc<SkillIndex>,
    ag_ui: AgUiState,
}

impl AppState {
    pub fn new(skills_dir: &Path) -> Self {
        Self {
            skill_index: Arc::new(SkillIndex::load_from_dir(skills_dir)),
            ag_ui: AgUiState::new(),
        }
    }
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

// ---------------------------------------------------------------------------
// Router builder
// ---------------------------------------------------------------------------

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // AG-UI state needs to be extracted separately.
    let ag_state = state.ag_ui.clone();

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/skills/search", get(skills_search))
        .route("/api/v1/sync/status", get(sync_status))
        .route("/api/v1/sync/peers", get(sync_peers))
        .route("/api/v1/sync/push", post(sync_push))
        .route(
            "/api/v1/stream",
            post(ag_ui_stream).with_state(ag_state.clone()),
        )
        .route("/api/v1/stream/ping", get(ag_ui_ping))
        .with_state(state)
        .layer(cors)
}

// ---------------------------------------------------------------------------
// Server entry point
// ---------------------------------------------------------------------------

pub async fn serve(port: u16, skills_dir: &Path) -> anyhow::Result<()> {
    let state = AppState::new(skills_dir);
    let app = build_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("sovereign-sync REST API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
