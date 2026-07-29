---
id: rust-sdk
title: Rust SDK
sidebar_label: Rust SDK
---

# Sovereign Client — Rust SDK

`sovereign-client` provides typed models and convenience methods for the
Sovereign Sync REST and AG-UI surfaces.

## Current authentication limitation

The server now requires a bearer token on every route except `/health`.
`SovereignClient::new(base_url)` currently has no token parameter and does not
attach an `Authorization` header.

Therefore, against the current daemon:

| Method | Current result |
|---|---|
| `health()` | Works |
| `search_skills()` | HTTP `401` |
| `sync_status()` | HTTP `401` |
| `sync_push()` | HTTP `401` |
| `stream_task()` | HTTP `401` |

This is a documented SDK gap, not a server configuration problem. Do not
disable server authentication to make the old client examples pass.

## Add to a workspace

```toml
[dependencies]
sovereign-client = { path = "../substrate/sovereign-client" }
```

## Health-only use

```rust
use sovereign_client::SovereignClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SovereignClient::new("http://127.0.0.1:7892")?;
    let health = client.health().await?;
    println!("Service: {}", health["service"]);
    Ok(())
}
```

## Authenticated workaround with reqwest

Keep token loading in a trusted backend:

```rust
use reqwest::Client;

async fn sync_status(
    base_url: &str,
    token: &str,
) -> anyhow::Result<serde_json::Value> {
    let response = Client::new()
        .get(format!("{base_url}/api/v1/sync/status"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(response.json().await?)
}
```

Read the token from the project-specific mode-`0600` file described in
[Tokens and authentication](/docs/kbd/tokens-and-authentication). Never embed
it in frontend JavaScript or a compiled web asset.

## Existing API surface

```text
SovereignClient::new(base_url)
health()
search_skills(query, limit)
sync_status()
sync_push(domain)
stream_task(task)
```

The next compatible SDK revision needs a constructor or builder that accepts a
secret bearer token and applies it to REST and SSE requests without exposing
the value through `Debug` or logs.

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

At present, `error_for_status()` reports authenticated-route `401` responses
through `ClientError::Http`.
