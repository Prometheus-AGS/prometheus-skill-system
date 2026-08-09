---
license: MIT
name: axum-patterns
version: '1.0.0'
description: >
  Axum 0.8 router, middleware, extractor, and state injection patterns for Prometheus AGS
  projects. Enforces the canonical Axum structure: typed state via Extension/State extractors,
  Tower middleware composition, structured error types, and Axum-native SSE for MCP servers.
  Use when building any Axum HTTP service, MCP server, or API gateway in the Prometheus stack.
language: rust
metadata:
  tags: [rust, patterns]
---

# Axum Patterns — Rust

Canonical patterns for Axum 0.8 in the Prometheus AGS Rust stack.

## Router Structure

Always use `Router::new()` with typed state. State must be `Clone + Send + Sync + 'static`.
Prefer `Arc<T>` for shared mutable state; inject it via `.with_state()`.

```rust
use axum::{Router, routing::{get, post}, extract::State};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SomeDb>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/resource", post(create_resource))
        .with_state(state)
}
```

**Never** put unbounded state in the router without `Arc`. Cloning a large struct on every
request is a performance regression that will not be caught by the compiler.

## Extractor Ordering

Axum extractors must be ordered: `State` first, then `Path`/`Query`, then `Json` (consumes body).
Put `State` as the first parameter — it is always present. Consuming extractors (`Json`, `Bytes`)
must be last.

```rust
async fn create_resource(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateRequest>,
) -> Result<Json<Resource>, AppError> {
    // ...
}
```

## Error Handling

Define a single `AppError` type that implements `IntoResponse`. Never return bare `StatusCode`
from handlers — it loses context. Never call `unwrap()` or `expect()` in handler bodies.

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

## Tower Middleware

Add middleware via `tower_http` layers. Always apply `TraceLayer` for request tracing and
`CorsLayer` for cross-origin support. Apply in order: outermost layer runs first on request,
last on response.

```rust
use tower_http::{cors::CorsLayer, trace::TraceLayer};

let app = router(state)
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive()); // Restrict in production
```

For custom middleware, implement as an `async fn` via `axum::middleware::from_fn_with_state`
when state access is needed, or `from_fn` for stateless middleware.

## SSE for MCP Servers

MCP servers in the Prometheus stack use Axum SSE for the `/events` stream and HTTP POST
for `/mcp` JSON-RPC dispatch. See `rust/mcp-server` skill for the complete pattern.

```rust
use axum::response::sse::{Event, Sse};
use tokio_stream::wrappers::BroadcastStream;

async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg| Ok(Event::default().data(serde_json::to_string(&msg?).unwrap_or_default())));
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

## Startup Pattern

Use `tokio::net::TcpListener::bind` + `axum::serve`. Never use `axum::Server` (deprecated
in 0.7+). Always bind before handing off to graceful shutdown.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new().await?;
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
```

## Forbidden Patterns

- `unwrap()` / `expect()` in handler bodies — use `?` and `AppError`
- `axum::Server` — deprecated; use `axum::serve`
- Returning bare `StatusCode` — use `AppError` or a typed response
- Cloning non-`Arc` state in routes — always wrap shared state in `Arc`
