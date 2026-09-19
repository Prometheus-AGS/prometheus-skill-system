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
            "service": "prometheus-research",
            // What the daemon would actually run, resolved now. A stale install
            // is visible here before a job is started, instead of after one has
            // failed (defects D-A and D-B).
            "execution": crate::job::daemon::self_check()
        })),
    )
}
