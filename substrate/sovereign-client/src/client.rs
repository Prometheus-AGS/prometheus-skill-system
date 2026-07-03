use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::pin::Pin;
use tracing::debug;
use url::Url;

use crate::error::ClientError;
use crate::types::{AgUiEvent, SkillResult, SyncStatus};

/// Client for a running sovereign-sync node.
pub struct SovereignClient {
    base_url: Url,
    http: Client,
}

impl SovereignClient {
    /// Create a client pointing at the given base URL (e.g. "http://127.0.0.1:7892").
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        Ok(Self {
            base_url: Url::parse(base_url)?,
            http: Client::new(),
        })
    }

    fn url(&self, path: &str) -> Result<Url, ClientError> {
        Ok(self.base_url.join(path)?)
    }

    // -----------------------------------------------------------------------
    // REST API
    // -----------------------------------------------------------------------

    /// Check node health.
    pub async fn health(&self) -> Result<Value, ClientError> {
        let url = self.url("/health")?;
        debug!("GET {}", url);
        let resp = self.http.get(url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    /// Search skills on the remote node.
    pub async fn search_skills(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SkillResult>, ClientError> {
        let url = self.url("/api/v1/skills/search")?;
        let resp = self
            .http
            .get(url)
            .query(&[("q", query), ("limit", &limit.to_string())])
            .send()
            .await?
            .error_for_status()?;
        let body: Value = resp.json().await?;
        let results = body
            .get("results")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
        Ok(results)
    }

    /// Get current sync status from the remote node.
    pub async fn sync_status(&self) -> Result<SyncStatus, ClientError> {
        let url = self.url("/api/v1/sync/status")?;
        let resp = self.http.get(url).send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    /// Push a sync domain to peers.
    pub async fn sync_push(&self, domain: &str) -> Result<Value, ClientError> {
        let url = self.url("/api/v1/sync/push")?;
        let resp = self
            .http
            .post(url)
            .json(&serde_json::json!({ "domain": domain }))
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // AG-UI SSE stream
    // -----------------------------------------------------------------------

    /// Submit an A2UI task and stream AG-UI events until completion.
    ///
    /// Returns a stream of `AgUiEvent`s. The caller drives it with `.next()`.
    pub async fn stream_task(
        &self,
        task: Value,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = Result<AgUiEvent, ClientError>> + Send>>,
        ClientError,
    > {
        let url = self.url("/api/v1/stream")?;
        let resp = self
            .http
            .post(url)
            .json(&task)
            .send()
            .await?
            .error_for_status()?;

        let byte_stream = resp.bytes_stream();
        let event_stream = byte_stream.eventsource().map(|result| match result {
            Ok(event) => serde_json::from_str::<AgUiEvent>(&event.data).map_err(ClientError::Json),
            Err(e) => Err(ClientError::Stream(e.to_string())),
        });

        Ok(Box::pin(event_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_construction_with_valid_url() {
        let client = SovereignClient::new("http://127.0.0.1:7892");
        assert!(client.is_ok());
    }

    #[test]
    fn client_construction_with_invalid_url() {
        let client = SovereignClient::new("not-a-url");
        assert!(client.is_err());
    }

    #[test]
    fn url_joining_works() {
        let client = SovereignClient::new("http://127.0.0.1:7892").unwrap();
        let url = client.url("/health").unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:7892/health");
    }
}
