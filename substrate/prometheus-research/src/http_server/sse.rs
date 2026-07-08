use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::stream::{self, StreamExt};
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::agui::AguiEvent;
use super::rest::AppState;

pub type EventBroadcast = broadcast::Sender<AguiEvent>;

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
                    let matches = match &event {
                        AguiEvent::AgentStatus { job_id: jid, .. } => jid == &job_id,
                        AguiEvent::AgentMessage { job_id: jid, .. } => jid == &job_id,
                        AguiEvent::AgentError { job_id: jid, .. } => jid == &job_id,
                        AguiEvent::A2uiComponent { job_id: jid, .. } => jid == &job_id,
                    };
                    if matches {
                        let event_type = match &event {
                            AguiEvent::AgentStatus { .. } => "agent.status",
                            AguiEvent::AgentMessage { .. } => "agent.message",
                            AguiEvent::AgentError { .. } => "agent.error",
                            AguiEvent::A2uiComponent { .. } => "a2ui.component",
                        };
                        let data = serde_json::to_string(&event).ok()?;
                        Some(Ok::<Event, Infallible>(
                            Event::default().event(event_type).data(data),
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
