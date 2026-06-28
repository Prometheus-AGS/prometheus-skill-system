use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SurfaceTierResponse {
    pub tier: String,
    pub harness: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiIntent {
    pub intent_type: String, // "question"|"prompt"|"feedback"|"progress"
    pub title: String,
    pub body: String,
    pub options: Option<Vec<String>>,
    pub multiselect: bool,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderResponse {
    pub request_id: String,
    pub status: String, // "rendered" | "error"
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectRequest {
    pub request_id: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectResponse {
    pub request_id: String,
    pub status: String, // "ready" | "pending" | "timeout"
    pub response: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub pid: u32,
}
