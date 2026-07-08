pub mod health;
pub mod rest;
pub mod sse;

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::{a2ui::registry::ComponentRegistry, config::ResearchConfig};
use rest::AppState;

pub async fn run_server(port: u16, cfg: ResearchConfig) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel(128);

    let state = AppState {
        broadcast: tx,
        registry: ComponentRegistry::new(),
        surface_bridge_url: cfg.surface_bridge_url().to_string(),
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
        .route("/static/{*path}", get(rest::static_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("prometheus-research HTTP server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
