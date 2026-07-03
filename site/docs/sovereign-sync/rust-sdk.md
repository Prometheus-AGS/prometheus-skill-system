---
id: rust-sdk
title: Rust SDK
sidebar_label: Rust SDK
---

# Sovereign Client — Rust SDK

`sovereign-client` is the official Rust SDK for connecting to a `sovereign-sync` node.

## Add to your project

```toml
[dependencies]
sovereign-client = { path = "../substrate/sovereign-client" }
```

## Usage

```rust
use sovereign_client::SovereignClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SovereignClient::new("http://127.0.0.1:7892")?;

    // Health check
    let health = client.health().await?;
    println!("Service: {}", health["service"]);

    // Search skills
    let results = client.search_skills("feynman", 5).await?;
    for r in &results {
        println!("{}: {}", r.name, r.description);
    }

    // Get sync status
    let status = client.sync_status().await?;
    println!("State: {}", status.node_state);
    println!("Peers: {}", status.peers.len());

    // Push a domain
    let resp = client.sync_push("learner-model").await?;
    println!("Push status: {}", resp["status"]);

    Ok(())
}
```

## AG-UI SSE streaming

```rust
use futures::StreamExt;
use serde_json::json;
use sovereign_client::{AgUiEvent, SovereignClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SovereignClient::new("http://127.0.0.1:7892")?;

    let task = json!({ "kind": "SyncPush", "domain": "skill-index" });
    let mut stream = client.stream_task(task).await?;

    while let Some(event) = stream.next().await {
        match event? {
            AgUiEvent::TaskAccepted { task_id } => println!("Task accepted: {task_id}"),
            AgUiEvent::Progress { task_id, message, percent } => {
                println!("[{percent}%] {message}");
            }
            AgUiEvent::Done { task_id, result } => {
                println!("Done: {result}");
                break;
            }
            AgUiEvent::Error { task_id, error } => {
                eprintln!("Error: {error}");
                break;
            }
            AgUiEvent::Ping => {}
        }
    }

    Ok(())
}
```

## API reference

### `SovereignClient::new(base_url: &str) -> Result<Self, ClientError>`

Create a client. Parses the base URL; returns `ClientError::Url` on invalid URL.

### `health() -> Result<serde_json::Value, ClientError>`

GET `/health` — returns the raw JSON response.

### `search_skills(query: &str, limit: usize) -> Result<Vec<SkillResult>, ClientError>`

GET `/api/v1/skills/search` — returns a list of matching skills.

### `sync_status() -> Result<SyncStatus, ClientError>`

GET `/api/v1/sync/status` — returns node state and peer list.

### `sync_push(domain: &str) -> Result<serde_json::Value, ClientError>`

POST `/api/v1/sync/push` — queues a domain for broadcast.

### `stream_task(task: Value) -> Result<impl Stream<Item = Result<AgUiEvent, ClientError>>, ClientError>`

POST `/api/v1/stream` — returns an async stream of AG-UI SSE events.

## Error types

```rust
pub enum ClientError {
    Http(reqwest::Error),    // HTTP request failed
    Json(serde_json::Error), // Deserialization failed
    Stream(String),          // SSE stream error
    Url(url::ParseError),    // Invalid base URL
    Api(String),             // Server returned error body
}
```
