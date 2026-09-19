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

    // Jobs this server starts are run by a separate `--daemon-job` process,
    // which inherits this environment and posts its AG-UI events back here
    // (see job::daemon and sse::ingest_handler). `RESEARCH_EVENT_SINK` and
    // `RESEARCH_EVENT_TOKEN` are set by main() before the runtime starts;
    // nothing here mutates the environment. A missing token means every
    // ingest is refused, which is reported rather than silently accepted.
    if std::env::var_os("RESEARCH_EVENT_TOKEN").is_none_or(|v| v.is_empty()) {
        tracing::warn!(
            "RESEARCH_EVENT_TOKEN is unset: every job-event ingest POST will be refused. Set it in the service environment (the plist's EnvironmentVariables) before starting the daemon."
        );
    }

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
        .route("/", get(rest::index_handler))
        .route("/manifest.json", get(rest::manifest_handler))
        .route("/health", get(health::health_handler))
        .route("/api/v1/jobs", post(rest::create_job))
        .route("/api/v1/jobs/{id}", get(rest::get_job))
        .route("/api/v1/jobs/{id}", delete(rest::delete_job))
        .route("/api/v1/jobs/{id}/events", get(sse::sse_handler))
        .route("/api/v1/jobs/{id}/events", post(sse::ingest_handler))
        .route("/components/{name}", get(rest::get_component))
        .route("/static/{*path}", get(rest::static_handler))
        .route("/brand/{*path}", get(rest::brand_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("prometheus-research HTTP server listening on http://{addr}");

    // Report what would actually run, at startup. Under launchd this line is the
    // first place an operator sees that the plist granted no PATH (D-A) or that
    // the installed driver is a pre-contract stub (D-B) — both of which
    // previously surfaced only after a job had already failed.
    let check = crate::job::daemon::self_check();
    let harness_ok = check["harness"]["status"] == "ok";
    let driver_ok = check["driver"]["status"] == "ok";
    if harness_ok && driver_ok {
        tracing::info!(
            harness = %check["harness"]["path"].as_str().unwrap_or("?"),
            driver = %check["driver"]["path"].as_str().unwrap_or("?"),
            "execution self-check: ready"
        );
    } else {
        // A warning, not a refusal: refusing to start would make a stale install
        // harder to diagnose, not easier — /health is how an operator inspects it.
        tracing::warn!(
            harness_status = %check["harness"]["status"].as_str().unwrap_or("?"),
            driver_status = %check["driver"]["status"].as_str().unwrap_or("?"),
            detail = %check.to_string(),
            "execution self-check: NOT ready — research_start will fail until this is fixed"
        );
    }
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
