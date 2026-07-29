---
id: ag-ui-sse
title: AG-UI / A2UI SSE
sidebar_label: AG-UI SSE
---

# AG-UI SSE Endpoint

The AG-UI endpoint (`POST /api/v1/stream`) exposes an Agent-to-UI task/event
schema over Server-Sent Events. In `0.1.0`, the executor emits synthetic
accept/progress/done events for the request shape; it is not connected to the
live P2P node, skill index, or domain replication pipeline.

## Task kinds

| Kind | Description |
|------|-------------|
| `sync_push` | Acknowledge a domain push request |
| `peer_status` | Return the scaffold empty-peer state |
| `skill_search` | Return the scaffold empty search result |
| `node_relay` | Return a scaffold relay acknowledgement |

## Event types

```json
{ "type": "task_accepted", "task_id": "uuid" }
{ "type": "progress",      "task_id": "uuid", "message": "...", "percent": 42 }
{ "type": "done",          "task_id": "uuid", "result": {} }
{ "type": "error",         "task_id": "uuid", "error": "..." }
{ "type": "ping" }
```

## Example: Push with progress stream

Derive `AUTH_HEADER` as shown in the [REST authentication
helper](./rest-api#authentication-helper), then:

```bash
curl -s -X POST http://127.0.0.1:7892/api/v1/stream \
  -H "$AUTH_HEADER" \
  -H 'Content-Type: application/json' \
  -d '{
    "task_id": "example-sync-push-1",
    "kind": "sync_push",
    "payload": {"domain": "skill-index"}
  }' \
  --no-buffer
```

**SSE output:**

```
data: {"type":"task_accepted","task_id":"example-sync-push-1"}

data: {"type":"progress","task_id":"example-sync-push-1","message":"Queuing sync-push for domain: skill-index","percent":50}

data: {"type":"done","task_id":"example-sync-push-1","result":{"status":"queued","domain":"skill-index"}}

```

The final event confirms only that the stub executor completed. It is not
domain export, broadcast, peer receipt, or apply confirmation.

## Ping endpoint

A GET endpoint is available for SSE health checks:

```bash
curl -s -H "$AUTH_HEADER" http://127.0.0.1:7892/api/v1/stream/ping
```

Returns a single `ping` event and closes the stream.

## Rust and Tauri clients

The current `sovereign-client` crate predates mandatory bearer authentication:
`health()` still works, but its authenticated REST/SSE methods receive `401`
against the current daemon. Until the crate gains a bearer-token constructor,
use `reqwest::Client::bearer_auth` directly or place authenticated calls in a
trusted Tauri backend.

## Tauri integration

Do not put the bearer token in browser JavaScript. Standard browser
`EventSource` also cannot set the required `Authorization` header. A Tauri
backend or same-origin trusted proxy should make the authenticated request and
forward sanitized progress events to the webview:

```typescript
import {listen} from '@tauri-apps/api/event';

await listen('sovereign-progress', ({payload}) => {
  console.log(payload);
});
```

The backend owns the token file and sends `Authorization: Bearer …`; the
webview receives only progress payloads.
