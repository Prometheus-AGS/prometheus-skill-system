---
id: rust-sdk
title: Rust SDK
sidebar_label: Rust SDK
---

# Sovereign Client — Rust SDK

`sovereign-client` provides typed models and convenience methods for the
loopback Sovereign Sync REST and AG-UI surfaces.

## Add to a workspace

```toml
[dependencies]
sovereign-client = { path = "../substrate/sovereign-client" }
kbd-runtime = { path = "../substrate/kbd-runtime" }
```

## Basic use

```rust
use sovereign_client::SovereignClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SovereignClient::new("http://127.0.0.1:7892")?;
    let health = client.health().await?;
    let sync = client.sync_status().await?;
    println!("{}: {}", health["service"], sync.node_state);
    Ok(())
}
```

The daemon is loopback-only. Read routes do not use the removed bearer-token
scheme. KBD mutation methods accept a `SignedCommandEnvelope`; the caller must
sign a schema-v2 command with an active enrolled device key before submission.

## KBD status and signed commands

```rust
use kbd_runtime::{CommandEnvelope, SignedCommandEnvelope};

let state = client.kbd_status(project_id).await?;
let command: CommandEnvelope = build_command_with_frontier(state.frontier);
let signed = SignedCommandEnvelope::sign(command, &device_signer)?;
let committed = client.submit_kbd_command(project_id, &signed).await?;
```

Normal commands use the current causal frontier. Scalar revision is a derived
compatibility projection and is not the concurrency authority.

## Continuous operational events

```rust
use futures::StreamExt;

let mut events = client.stream_events().await?;
while let Some(event) = events.next().await {
    println!("{:?}", event?);
}
```

The typed stream includes `event_appended`, `claim_acquired`,
`claim_conflict`, and `singleton_violation` in addition to AG-UI task events.

## API surface

```text
SovereignClient::new(base_url)
health()
search_skills(query, limit)
sync_status()
sync_push(domain)
kbd_status(project_id)
submit_kbd_command(project_id, signed_command)
kbd_claims(project_id)
stream_task(task)
stream_events()
```

## Error types

```rust
pub enum ClientError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Stream(String),
    Url(url::ParseError),
    Api(String),
}
```
