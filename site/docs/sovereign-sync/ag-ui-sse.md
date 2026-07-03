---
id: ag-ui-sse
title: AG-UI / A2UI SSE
sidebar_label: AG-UI SSE
---

# AG-UI SSE Endpoint

The AG-UI endpoint (`POST /api/v1/stream`) implements the Agent-to-UI (A2UI) streaming
protocol. It enables Tauri desktop apps, web clients, and other frontends to receive
real-time progress updates for sync operations.

## Task kinds

| Kind | Description |
|------|-------------|
| `SyncPush` | Push a CRDT domain to peers |
| `PeerStatus` | Poll peer connection state |
| `SkillSearch` | Search skills with streaming results |
| `NodeRelay` | Relay a message to another node |

## Event types

```json
{ "type": "task_accepted", "task_id": "uuid" }
{ "type": "progress",      "task_id": "uuid", "message": "...", "percent": 42 }
{ "type": "done",          "task_id": "uuid", "result": {} }
{ "type": "error",         "task_id": "uuid", "error": "..." }
{ "type": "ping" }
```

## Example: Push with progress stream

```bash
curl -s -X POST http://127.0.0.1:7892/api/v1/stream \
  -H 'Content-Type: application/json' \
  -d '{"kind": "SyncPush", "domain": "skill-index"}' \
  --no-buffer
```

**SSE output:**

```
data: {"type":"task_accepted","task_id":"a1b2-..."}

data: {"type":"progress","task_id":"a1b2-...","message":"Serializing skill index","percent":25}

data: {"type":"progress","task_id":"a1b2-...","message":"Broadcasting to 2 peers","percent":75}

data: {"type":"done","task_id":"a1b2-...","result":{"bytes":4096,"peers_reached":2}}

```

## Ping endpoint

A GET endpoint is available for SSE health checks:

```bash
curl -s http://127.0.0.1:7892/api/v1/stream/ping
```

Returns a single `ping` event and closes the stream.

## Using with the Rust SDK

```rust
use futures::StreamExt;
use sovereign_client::{AgUiEvent, SovereignClient};

let client = SovereignClient::new("http://127.0.0.1:7892")?;
let mut stream = client.stream_task(serde_json::json!({
    "kind": "SyncPush",
    "domain": "skill-index"
})).await?;

while let Some(event) = stream.next().await {
    match event? {
        AgUiEvent::Progress { message, percent, .. } => {
            println!("[{percent}%] {message}");
        }
        AgUiEvent::Done { .. } => break,
        _ => {}
    }
}
```

## Tauri integration

The AG-UI SSE endpoint is designed to be consumed by a Tauri frontend via
`eventsource` or `EventSource` in the webview:

```typescript
const source = new EventSource('http://127.0.0.1:7892/api/v1/stream', {
  method: 'POST',
  body: JSON.stringify({ kind: 'SyncPush', domain: 'skill-index' }),
  headers: { 'Content-Type': 'application/json' }
});

source.onmessage = (e) => {
  const event = JSON.parse(e.data);
  if (event.type === 'done') source.close();
};
```

The `sovereign-client` Rust crate can also be embedded in the Tauri backend sidecar.
