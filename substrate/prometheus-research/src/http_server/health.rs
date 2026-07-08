use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn health_handler() -> (StatusCode, Json<Value>) {
    let pid = std::process::id();
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "pid": pid,
            "service": "prometheus-research"
        })),
    )
}
