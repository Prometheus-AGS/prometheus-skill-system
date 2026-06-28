use axum::{http::StatusCode, Json};
use crate::types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// In-memory pending responses store (keyed by request_id)
// In production this would be a proper state layer
static PENDING: std::sync::OnceLock<Arc<Mutex<HashMap<String, serde_json::Value>>>> =
    std::sync::OnceLock::new();

fn pending_store() -> &'static Arc<Mutex<HashMap<String, serde_json::Value>>> {
    PENDING.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        pid: std::process::id(),
    })
}

pub async fn detect_surface_tier() -> Json<SurfaceTierResponse> {
    // Read SURFACE_TIER env var (set by detect-surface-tier.sh)
    // If surface-bridge is running, by definition we are at Tier 2
    let tier = std::env::var("SURFACE_TIER")
        .unwrap_or_else(|_| "tier2_mcp_app".to_string());
    let harness = std::env::var("CLAUDE_HARNESS")
        .unwrap_or_else(|_| "unknown".to_string());

    Json(SurfaceTierResponse { tier, harness })
}

pub async fn render_ui_intent(
    Json(intent): Json<UiIntent>,
) -> (StatusCode, Json<RenderResponse>) {
    // In Tier 2: serve an HTML shell via MCP App iframe
    // For now: store the intent and return "rendered" — a real implementation
    // would push the intent to a WebSocket / SSE stream consumed by the iframe
    eprintln!(
        "[surface-bridge] render_ui_intent: {} ({})",
        intent.title, intent.request_id
    );

    (
        StatusCode::OK,
        Json(RenderResponse {
            request_id: intent.request_id,
            status: "rendered".to_string(),
            message: Some(
                "Intent queued for display (stub implementation)".to_string(),
            ),
        }),
    )
}

pub async fn collect_response(Json(req): Json<CollectRequest>) -> Json<CollectResponse> {
    // Poll the pending store for a response
    let store = pending_store();
    let response = store
        .lock()
        .ok()
        .and_then(|m| m.get(&req.request_id).cloned());

    match response {
        Some(r) => Json(CollectResponse {
            request_id: req.request_id,
            status: "ready".to_string(),
            response: Some(r),
        }),
        None => Json(CollectResponse {
            request_id: req.request_id,
            status: "pending".to_string(),
            response: None,
        }),
    }
}
