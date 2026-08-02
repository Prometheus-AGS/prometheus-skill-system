---
id: ag-ui-sse
title: AG-UI / A2UI SSE
sidebar_label: AG-UI SSE
---

# AG-UI SSE Endpoint

The task endpoint (`POST /api/v1/stream`) exposes Agent-to-UI task progress.
The continuous operational endpoint (`GET /api/v1/events`) emits live KBD
authority and conflict notifications.

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
{ "type": "event_appended", "project_id": "uuid", "event_id": "uuid", "replica_id": "uuid", "lamport": 3, "frontier": {} }
{ "type": "claim_acquired", "project_id": "uuid", "claim": {} }
{ "type": "claim_conflict", "project_id": "uuid", "conflict": {} }
{ "type": "singleton_violation", "project_id": "uuid", "conflict": {} }
{ "type": "ping" }
```

## Example: Push with progress stream

```bash
curl -s -X POST http://127.0.0.1:7892/api/v1/stream \
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
curl -s http://127.0.0.1:7892/api/v1/stream/ping
```

Returns a single `ping` event and closes the stream.

## Rust and Tauri clients

`sovereign-client` exposes `stream_task()` for task progress and
`stream_events()` for typed operational events. Its KBD command method accepts
only a `SignedCommandEnvelope`; host code supplies and protects the device key.

## Tauri integration

Do not put device signing keys in browser JavaScript. A Tauri backend or
same-origin trusted proxy should sign commands and forward sanitized events to
the webview:

```typescript
import {listen} from '@tauri-apps/api/event';

await listen('sovereign-progress', ({payload}) => {
  console.log(payload);
});
```

The backend owns the device key; the webview receives only progress payloads.
