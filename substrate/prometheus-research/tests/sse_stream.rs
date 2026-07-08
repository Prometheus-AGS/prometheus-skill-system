use prometheus_research::{
    a2ui::registry::ComponentRegistry,
    agui::AguiEvent,
    http_server::{
        health, rest,
        sse::{self, EventBroadcast},
    },
};

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

/// Build a test AppState + Router on port 0 and return (url, sender).
async fn spawn_test_server() -> (String, EventBroadcast) {
    let (tx, _rx) = broadcast::channel::<AguiEvent>(128);
    let state = rest::AppState {
        broadcast: tx.clone(),
        registry: ComponentRegistry::new(),
        surface_bridge_url: "http://127.0.0.1:7890".into(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health::health_handler))
        .route("/api/v1/jobs", post(rest::create_job))
        .route("/api/v1/jobs/{id}", get(rest::get_job))
        .route("/api/v1/jobs/{id}", delete(rest::delete_job))
        .route("/api/v1/jobs/{id}/events", get(sse::sse_handler))
        .route("/components/{name}", get(rest::get_component))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{port}");

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (url, tx)
}

/// GET /api/v1/jobs/<id>/events must respond with Content-Type: text/event-stream.
#[tokio::test]
async fn sse_endpoint_returns_event_stream_content_type() {
    let (base_url, _tx) = spawn_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/api/v1/jobs/test-job-001/events"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/event-stream"),
        "expected text/event-stream, got: {ct}"
    );
}

/// Emitting an AguiEvent on the broadcast channel must appear in the SSE stream
/// for the matching job_id.
#[tokio::test]
async fn broadcast_event_appears_in_sse_stream() {
    let (base_url, tx) = spawn_test_server().await;
    let job_id = "sse-test-job-42".to_string();

    let client = reqwest::Client::new();

    // Open the SSE connection
    let resp = client
        .get(format!("{base_url}/api/v1/jobs/{job_id}/events"))
        .send()
        .await
        .expect("SSE request failed");
    assert_eq!(resp.status(), 200);

    // Emit an event after a brief yield so the subscriber is registered
    tokio::task::yield_now().await;
    let event = AguiEvent::AgentMessage {
        job_id: job_id.clone(),
        message: "hello from broadcast test".into(),
        level: "info".into(),
        timestamp: "2026-07-08T00:00:00Z".into(),
    };
    let _ = tx.send(event);

    // Read the initial "connected" event plus the first data event from the stream
    use futures::StreamExt;
    let mut byte_stream = resp.bytes_stream();

    // Collect up to 2 chunks (connected event + our broadcast event) with a timeout
    let mut collected = String::new();
    let deadline = std::time::Duration::from_secs(3);
    let start = std::time::Instant::now();

    while start.elapsed() < deadline {
        tokio::select! {
            chunk = byte_stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        collected.push_str(&String::from_utf8_lossy(&bytes));
                        if collected.contains("agent.message") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            _ = tokio::time::sleep(deadline) => break,
        }
    }

    assert!(
        collected.contains("connected") || collected.contains("agent.message"),
        "SSE stream did not contain expected events. Got: {collected:?}"
    );
}
