use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    a2ui::registry::ComponentRegistry,
    agui::{emit::emit_to_surface_bridge, AguiEvent},
    job::{cancel::cancel_job, checkpoint, spawn::spawn_job},
};

use super::sse::EventBroadcast;

#[derive(Debug, Clone)]
pub struct AppState {
    pub broadcast: EventBroadcast,
    pub registry: ComponentRegistry,
    pub surface_bridge_url: String,
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub query: String,
    #[serde(default = "default_depth")]
    pub depth: String,
    #[serde(default = "default_max_sources")]
    pub max_sources: u32,
    #[serde(default = "default_citation_style")]
    pub citation_style: String,
}

fn default_depth() -> String {
    "moderate".into()
}

fn default_max_sources() -> u32 {
    10
}

fn default_citation_style() -> String {
    "apa".into()
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

pub async fn create_job(
    State(state): State<AppState>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    match spawn_job(&req.query, &req.depth, req.max_sources, &req.citation_style) {
        Ok(job_id) => {
            let event = AguiEvent::AgentStatus {
                job_id: job_id.clone(),
                stage: 0,
                stage_name: "queued".into(),
                progress: 0,
                status: "running".into(),
                tokens: 0,
                timestamp: now_iso(),
            };
            let _ = state.broadcast.send(event.clone());
            let bridge = state.surface_bridge_url.clone();
            tokio::spawn(async move {
                emit_to_surface_bridge(&event, &bridge).await;
            });
            (
                StatusCode::CREATED,
                Json(json!({ "job_id": job_id, "status": "running" })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_job(
    Path(job_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    match checkpoint::read(&job_id) {
        Ok(cp) => (
            StatusCode::OK,
            Json(serde_json::to_value(&cp).unwrap_or_default()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn delete_job(
    Path(job_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match cancel_job(&job_id) {
        Ok(()) => {
            let event = AguiEvent::AgentStatus {
                job_id: job_id.clone(),
                stage: 0,
                stage_name: "cancelled".into(),
                progress: 0,
                status: "cancelled".into(),
                tokens: 0,
                timestamp: now_iso(),
            };
            let _ = state.broadcast.send(event);
            (
                StatusCode::OK,
                Json(json!({ "cancelled": true, "job_id": job_id })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

/// A2UI component endpoint — returns HTMX HTML fragment
#[derive(Debug, Deserialize)]
pub struct ComponentQuery {
    #[serde(default)]
    pub props: Option<String>,
}

pub async fn get_component(
    Path(name): Path<String>,
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<ComponentQuery>,
) -> impl IntoResponse {
    let props: Value = q
        .props
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(Value::Null);
    let html = state.registry.render(&name, props);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

// ---------------------------------------------------------------------------
// Static file serving — vendor assets embedded at build time
// ---------------------------------------------------------------------------

pub async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match path.as_str() {
        "htmx.min.js" => serve_bytes(
            include_bytes!("../static/htmx.min.js"),
            "application/javascript",
        ),
        "htmx-ext-sse.js" => serve_bytes(
            include_bytes!("../static/htmx-ext-sse.js"),
            "application/javascript",
        ),
        "htmx-ext-loading-states.js" => serve_bytes(
            include_bytes!("../static/htmx-ext-loading-states.js"),
            "application/javascript",
        ),
        "alpine.min.js" => serve_bytes(
            include_bytes!("../static/alpine.min.js"),
            "application/javascript",
        ),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .unwrap(),
    }
}

fn serve_bytes(bytes: &'static [u8], content_type: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(bytes))
        .unwrap()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_iso() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let s = d.as_secs();
            format!(
                "1970-01-01T{:02}:{:02}:{:02}Z",
                s / 3600 % 24,
                s / 60 % 60,
                s % 60
            )
        })
        .unwrap_or_else(|_| "unknown".into())
}
