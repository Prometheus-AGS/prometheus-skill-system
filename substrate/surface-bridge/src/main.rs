use axum::{routing::get, routing::post, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod handlers;
mod types;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route(
            "/mcp/detect-surface-tier",
            post(handlers::detect_surface_tier),
        )
        .route("/mcp/render-ui-intent", post(handlers::render_ui_intent))
        .route("/mcp/submit-response", post(handlers::submit_response))
        .route("/mcp/collect-response", post(handlers::collect_response))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 7890));
    eprintln!("[surface-bridge] Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
