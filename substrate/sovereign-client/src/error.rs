use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON deserialize failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("SSE stream error: {0}")]
    Stream(String),
    #[error("Invalid base URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("sovereign-sync returned error: {0}")]
    Api(String),
}
