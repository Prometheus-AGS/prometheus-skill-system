/// AG-UI and A2UI streaming endpoint for sovereign-sync.
///
/// AG-UI (Agent-UI) is the protocol for streaming agent events to a UI surface.
/// A2UI (Agent-to-UI) carries task schemas for managing sync domains.
///
/// Endpoint: POST /api/v1/stream
/// Returns: text/event-stream (Server-Sent Events)
use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive},
    response::{IntoResponse, Response, Sse},
    Json,
};
use futures::stream::{self, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info};

// ---------------------------------------------------------------------------
// A2UI task schemas
// ---------------------------------------------------------------------------

/// The root task categories sovereign-sync exposes via AG-UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncTaskKind {
    /// Push CRDT state for a domain to peers.
    SyncPush,
    /// Query peer status.
    PeerStatus,
    /// Search local skill index.
    SkillSearch,
    /// Generic node-to-node relay message.
    NodeRelay,
}

/// An A2UI task request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2uiTask {
    /// Unique task ID (caller-generated).
    pub task_id: String,
    /// Task kind.
    pub kind: SyncTaskKind,
    /// JSON payload specific to the task kind.
    pub payload: serde_json::Value,
}

/// An AG-UI event emitted during task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgUiEvent {
    /// Task accepted and queued.
    TaskAccepted { task_id: String },
    /// Intermediate progress message.
    Progress {
        task_id: String,
        message: String,
        percent: u8,
    },
    /// Task completed successfully.
    Done {
        task_id: String,
        result: serde_json::Value,
    },
    /// Task failed.
    Error { task_id: String, error: String },
    /// Heartbeat (keepalive).
    Ping,
}

// ---------------------------------------------------------------------------
// SSE streaming handler
// ---------------------------------------------------------------------------

/// SSE stream state shared between Axum handlers.
#[derive(Clone)]
pub struct AgUiState {
    _placeholder: Arc<()>,
}

impl AgUiState {
    pub fn new() -> Self {
        Self {
            _placeholder: Arc::new(()),
        }
    }
}

/// POST /api/v1/stream
///
/// Accepts an A2UI task, executes it (or queues it), and streams AG-UI events
/// back as Server-Sent Events.
pub async fn ag_ui_stream(
    State(_state): State<AgUiState>,
    Json(task): Json<A2uiTask>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("AG-UI task received: {} ({:?})", task.task_id, task.kind);

    let task_id = task.task_id.clone();
    let kind = task.kind.clone();

    // Build a channel-backed SSE stream.
    let (tx, rx) = mpsc::channel::<AgUiEvent>(16);

    // Spawn task executor.
    tokio::spawn(async move {
        let _ = tx
            .send(AgUiEvent::TaskAccepted {
                task_id: task_id.clone(),
            })
            .await;

        let result = execute_task(kind, task.payload, &task_id, &tx).await;

        let event = match result {
            Ok(val) => AgUiEvent::Done {
                task_id: task_id.clone(),
                result: val,
            },
            Err(e) => AgUiEvent::Error {
                task_id: task_id.clone(),
                error: e,
            },
        };
        let _ = tx.send(event).await;
    });

    let stream = ReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
        Ok::<_, Infallible>(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// GET /api/v1/stream/ping — sanity check endpoint (returns a single ping event).
pub async fn ag_ui_ping() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let ping = AgUiEvent::Ping;
    let data = serde_json::to_string(&ping).unwrap_or_else(|_| "{}".into());
    let event = Ok::<_, Infallible>(Event::default().data(data));
    Sse::new(stream::once(async move { event }))
}

// ---------------------------------------------------------------------------
// Task executor (stub implementations — wired to full logic in later changes)
// ---------------------------------------------------------------------------

async fn execute_task(
    kind: SyncTaskKind,
    payload: serde_json::Value,
    task_id: &str,
    tx: &mpsc::Sender<AgUiEvent>,
) -> Result<serde_json::Value, String> {
    match kind {
        SyncTaskKind::SyncPush => {
            let domain = payload
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("all");
            let _ = tx
                .send(AgUiEvent::Progress {
                    task_id: task_id.to_string(),
                    message: format!("Queuing sync-push for domain: {domain}"),
                    percent: 50,
                })
                .await;
            // Full implementation wired in change-sync-015.
            Ok(serde_json::json!({
                "status": "queued",
                "domain": domain
            }))
        }
        SyncTaskKind::PeerStatus => {
            let _ = tx
                .send(AgUiEvent::Progress {
                    task_id: task_id.to_string(),
                    message: "Checking peer connections…".into(),
                    percent: 50,
                })
                .await;
            Ok(serde_json::json!({
                "peers": [],
                "node_state": "idle"
            }))
        }
        SyncTaskKind::SkillSearch => {
            let query = payload.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let _ = tx
                .send(AgUiEvent::Progress {
                    task_id: task_id.to_string(),
                    message: format!("Searching skills for: {query}"),
                    percent: 50,
                })
                .await;
            // Full skill search wired via mcp_server::SkillIndex.
            Ok(serde_json::json!({
                "query": query,
                "results": []
            }))
        }
        SyncTaskKind::NodeRelay => {
            let _ = tx
                .send(AgUiEvent::Progress {
                    task_id: task_id.to_string(),
                    message: "Relaying message to peers…".into(),
                    percent: 50,
                })
                .await;
            Ok(serde_json::json!({ "relayed": true }))
        }
    }
}
