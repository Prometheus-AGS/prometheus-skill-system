use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
};
use futures::stream::{self, StreamExt};
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use super::rest::AppState;
use crate::agui::{emit::emit_to_surface_bridge, AguiEvent};

pub type EventBroadcast = broadcast::Sender<AguiEvent>;

/// Length-then-content comparison with no early exit on the content: the
/// token is the only thing between a local process and event forgery.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn event_type(event: &AguiEvent) -> &'static str {
    match event {
        AguiEvent::AgentStatus { .. } => "agent.status",
        AguiEvent::AgentMessage { .. } => "agent.message",
        AguiEvent::AgentError { .. } => "agent.error",
        AguiEvent::A2uiComponent { .. } => "a2ui.component",
    }
}

fn event_job_id(event: &AguiEvent) -> &str {
    match event {
        AguiEvent::AgentStatus { job_id, .. }
        | AguiEvent::AgentMessage { job_id, .. }
        | AguiEvent::AgentError { job_id, .. }
        | AguiEvent::A2uiComponent { job_id, .. } => job_id,
    }
}

pub async fn sse_handler(
    Path(job_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rx = state.broadcast.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let job_id = job_id.clone();
        async move {
            match msg {
                Ok(event) => {
                    if event_job_id(&event) == job_id {
                        let data = serde_json::to_string(&event).ok()?;
                        Some(Ok::<Event, Infallible>(
                            Event::default().event(event_type(&event)).data(data),
                        ))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
    });

    let connected = stream::once(async {
        Ok::<Event, Infallible>(
            Event::default()
                .event("connected")
                .data(r#"{"status":"connected"}"#),
        )
    });
    let merged = connected.chain(stream);

    Sse::new(merged).keep_alive(KeepAlive::default())
}

/// `POST /api/v1/jobs/{id}/events`: the event sink the `--daemon-job` process
/// posts to (change-rah-004). The daemon runs in its own process and cannot
/// reach this server's broadcast channel directly, so it delivers AG-UI
/// events here; the handler broadcasts them to every SSE subscriber for the
/// job and forwards them to the surface bridge, exactly as the server's own
/// events are.
///
/// Two refusals: an event whose `job_id` does not match the path (a misrouted
/// daemon cannot write into another job's stream), and a request without the
/// per-server token in `x-research-event-token`. The server generates
/// `RESEARCH_EVENT_TOKEN` at startup and the daemons it spawns inherit it, so
/// another local process that merely knows the port cannot forge events.
pub async fn ingest_handler(
    Path(job_id): Path<String>,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(event): Json<AguiEvent>,
) -> impl IntoResponse {
    // Read once for the life of the process, so the accepted token cannot
    // drift under a running server; compared in constant time.
    static EXPECTED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let expected = EXPECTED.get_or_init(|| std::env::var("RESEARCH_EVENT_TOKEN").unwrap_or_default());
    let presented = headers
        .get(crate::job::daemon::EVENT_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if expected.is_empty() || !constant_time_eq(presented.as_bytes(), expected.as_bytes()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "missing or invalid x-research-event-token (only the daemons this server spawned hold it)"
            })),
        );
    }
    if event_job_id(&event) != job_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("event job_id {} does not match path {job_id}", event_job_id(&event))
            })),
        );
    }
    let delivered = state.broadcast.send(event.clone()).unwrap_or(0);
    let bridge = state.surface_bridge_url.clone();
    tokio::spawn(async move {
        emit_to_surface_bridge(&event, &bridge).await;
    });
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "accepted": true, "subscribers": delivered })),
    )
}
