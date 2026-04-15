//! Surreal-memory MCP server client.
//!
//! Provides HTTP client for surreal-memory operations: entity creation,
//! relation management, search, and Graph-RAG traversal. Used by the
//! learning pipeline to persist execution traces and knowledge graph entries.

use serde::{Deserialize, Serialize};

/// Client for the surreal-memory REST API.
pub struct SurrealMemoryClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct CreateEntityRequest {
    entity_type: String,
    name: String,
    observations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CreateRelationRequest {
    from: String,
    to: String,
    relation_type: String,
}

#[derive(Debug, Serialize)]
struct SearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub observations: Vec<String>,
}

impl SurrealMemoryClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create from environment or default URL.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("SURREAL_MEMORY_URL")
            .unwrap_or_else(|_| "http://localhost:23001".to_string());

        Some(Self::new(url))
    }

    /// Check if the server is reachable.
    pub async fn ping(&self) -> anyhow::Result<bool> {
        let resp = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    /// Create an entity in the knowledge graph.
    pub async fn create_entity(
        &self,
        entity_type: &str,
        name: &str,
        observations: &[String],
    ) -> anyhow::Result<()> {
        self.client
            .post(format!("{}/api/v1/entities", self.base_url))
            .json(&CreateEntityRequest {
                entity_type: entity_type.to_string(),
                name: name.to_string(),
                observations: observations.to_vec(),
            })
            .send()
            .await?;
        Ok(())
    }

    /// Create a relation between two entities.
    pub async fn create_relation(
        &self,
        from: &str,
        relation_type: &str,
        to: &str,
    ) -> anyhow::Result<()> {
        self.client
            .post(format!("{}/api/v1/relations", self.base_url))
            .json(&CreateRelationRequest {
                from: from.to_string(),
                to: to.to_string(),
                relation_type: relation_type.to_string(),
            })
            .send()
            .await?;
        Ok(())
    }

    /// Search entities by text query.
    pub async fn search(&self, query: &str, entity_type: Option<&str>) -> anyhow::Result<Vec<Entity>> {
        let resp = self.client
            .post(format!("{}/api/v1/search", self.base_url))
            .json(&SearchRequest {
                query: query.to_string(),
                entity_type: entity_type.map(|s| s.to_string()),
                limit: Some(20),
            })
            .send()
            .await?;

        let entities: Vec<Entity> = resp.json().await?;
        Ok(entities)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
