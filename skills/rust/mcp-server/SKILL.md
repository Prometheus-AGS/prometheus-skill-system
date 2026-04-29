---
license: MIT
name: mcp-server
version: '1.0.0'
description: >
  Canonical Axum MCP server pattern for Prometheus AGS projects. Implements the
  MCP protocol as JSON-RPC 2.0 over HTTP POST /mcp with optional SSE event stream
  at GET /events. Covers tool registration, handler dispatch, SSE fan-out via
  broadcast, and stdio transport for Claude Desktop integration. Use when building
  any new MCP-enabled service in the Prometheus stack.
language: rust
---

# MCP Server — Rust

## Transport Modes

Prometheus MCP servers support two transports:

| Mode | Use Case |
|---|---|
| HTTP SSE (`POST /mcp` + `GET /events`) | Cherry Studio, UAR, Claude.ai remote MCP |
| stdio (`stdin`/`stdout` JSON-RPC) | Claude Desktop `.mcp.json` local tools |

The same tool handler logic is shared across both transports.

## Canonical Structure

```
my-mcp/
└── src/
    ├── lib.rs          ← re-exports server + tool registry
    ├── server.rs       ← Axum router, SSE stream, /mcp handler
    ├── tools.rs        ← tool definitions + dispatch table
    ├── events.rs       ← LibrarianEvent / ServerEvent types + broadcast
    └── stdio.rs        ← stdio transport loop (for Claude Desktop)
```

## Server Setup

```rust
use axum::{Router, routing::{get, post}, extract::State};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct McpState {
    pub event_tx: broadcast::Sender<ServerEvent>,
    // Add domain services here
}

pub fn router(state: McpState) -> Router {
    Router::new()
        .route("/mcp",    post(handle_mcp))
        .route("/events", get(handle_events))
        .route("/health", get(health))
        .with_state(Arc::new(state))
}
```

## JSON-RPC 2.0 Dispatch

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use axum::{extract::State, response::Json};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub async fn handle_mcp(
    State(state): State<Arc<McpState>>,
    Json(req): Json<McpRequest>,
) -> Json<Value> {
    let result = dispatch(&state, &req.method, req.params.as_ref()).await;

    Json(match result {
        Ok(value) => json!({ "jsonrpc": "2.0", "id": req.id, "result": value }),
        Err(e)    => json!({
            "jsonrpc": "2.0",
            "id": req.id,
            "error": { "code": -32603, "message": e.to_string() }
        }),
    })
}
```

## Tool Registry Pattern

Define tools in a structured registry so `tools/list` is always in sync with `tools/call`.

```rust
pub fn tool_definitions() -> Value {
    json!({
        "tools": [
            {
                "name": "my_tool",
                "description": "Does something useful.",
                "inputSchema": {
                    "type": "object",
                    "required": ["input"],
                    "properties": {
                        "input": { "type": "string", "description": "The input value" }
                    }
                }
            }
        ]
    })
}

async fn dispatch(state: &Arc<McpState>, method: &str, params: Option<&Value>) -> anyhow::Result<Value> {
    match method {
        "tools/list" => Ok(tool_definitions()),
        "tools/call" => {
            let name = params.and_then(|p| p["name"].as_str())
                .ok_or_else(|| anyhow::anyhow!("tool name required"))?;
            let args = params.and_then(|p| p.get("arguments"));
            dispatch_tool(state, name, args).await
        }
        _ => Err(anyhow::anyhow!("unknown method: {method}")),
    }
}

async fn dispatch_tool(state: &Arc<McpState>, name: &str, args: Option<&Value>) -> anyhow::Result<Value> {
    match name {
        "my_tool" => {
            let input = args.and_then(|a| a["input"].as_str())
                .ok_or_else(|| anyhow::anyhow!("input required"))?;
            let result = do_work(state, input).await?;
            Ok(json!({ "content": [{ "type": "text", "text": result }] }))
        }
        _ => Err(anyhow::anyhow!("unknown tool: {name}")),
    }
}
```

## SSE Event Stream

```rust
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

pub async fn handle_events(
    State(state): State<Arc<McpState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| match msg {
            Ok(event) => Some(Ok(
                Event::default().data(serde_json::to_string(&event).unwrap_or_default())
            )),
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                tracing::warn!("SSE client lagged, skipped {n} events");
                None
            }
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

## Stdio Transport (Claude Desktop)

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn run_stdio(state: Arc<McpState>) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() { continue; }
        let req: McpRequest = serde_json::from_str(&line)?;
        let result = dispatch(&state, &req.method, req.params.as_ref()).await;
        let response = match result {
            Ok(v) => json!({ "jsonrpc": "2.0", "id": req.id, "result": v }),
            Err(e) => json!({ "jsonrpc": "2.0", "id": req.id,
                              "error": { "code": -32603, "message": e.to_string() } }),
        };
        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }
    Ok(())
}
```

## Startup (Combined HTTP + Stdio)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (event_tx, _) = tokio::sync::broadcast::channel(256);
    let state = Arc::new(McpState { event_tx });

    if std::env::args().any(|a| a == "--stdio") {
        run_stdio(state).await
    } else {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8943".into()).parse::<u16>()?;
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        axum::serve(listener, router((*state).clone())).await?;
        Ok(())
    }
}
```

## Forbidden Patterns

- Tool names in `tools/call` that don't exist in `tools/list`
- Panicking inside a tool handler — always return `Err`
- Blocking I/O in tool handlers — use `tokio::task::spawn_blocking`
- Raw string `println!` for stdio transport — use structured JSON output only
