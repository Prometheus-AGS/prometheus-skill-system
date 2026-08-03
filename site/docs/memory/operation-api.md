---
title: Operation API
description: Submit, replay, reconcile, and stream durable v2 operations.
---

# Operation API

The checked-in [OpenAPI 3.1 document](/openapi/surreal-memory-v2.openapi.json) is the machine-readable contract. The default local base URL is `http://127.0.0.1:23001`.

## Submit an operation

`POST /api/v2/operations` accepts:

| Field | Meaning |
| --- | --- |
| `operation_id` | Stable caller-generated reconciliation key. |
| `schema_version` | Must be `2`. |
| `kind` | `add_memory`, `create_task_stream`, `add_task_step`, or `complete_step`. |
| `dependencies` | Operation IDs that must commit first. Empty by default. |
| `payload_hash` | Lowercase SHA-256 of canonical compact JSON payload bytes. |
| `payload` | Kind-specific object. `add_memory` accepts `content`, optional scope IDs, and categories. |

Canonical hashing sorts object keys lexicographically, keeps array order, emits compact UTF-8 JSON, and hashes those exact bytes. For example:

```bash
payload='{"categories":["architecture","durability"],"content":"Receipts are the acknowledgement boundary.","user_id":"prometheus-skill-pack"}'
payload_hash="$(printf '%s' "$payload" | shasum -a 256 | awk '{print $1}')"
```

The resulting hash is `a026d7fd122070a74973fa27e3ba92d438ba60e7fa84fd7fd3c8d769e3339a09`.

```json
{
  "dependencies": [],
  "kind": "add_memory",
  "operation_id": "memory-release-20260803",
  "payload": {
    "categories": ["architecture", "durability"],
    "content": "Receipts are the acknowledgement boundary.",
    "user_id": "prometheus-skill-pack"
  },
  "payload_hash": "a026d7fd122070a74973fa27e3ba92d438ba60e7fa84fd7fd3c8d769e3339a09",
  "schema_version": 2
}
```

New acceptance returns `202`. Exact replay returns `200` and the stored receipt. Invalid input returns `400`; a reused ID with a different hash returns `409`; unavailable durable storage returns `503`.

## Reconcile response loss

If the POST response disappears, do not invent another ID:

```bash
curl -fsS http://127.0.0.1:23001/api/v2/operations/memory-release-20260803
```

`GET /api/v2/operations/{operation_id}` returns the latest receipt, `404` when no receipt was durably accepted, and `503` when storage cannot answer. Poll until `state` is `committed` or `rejected`.

## Resume ordered events

`GET /api/v2/operations/{operation_id}/events?after=41` first replays persisted events with `sequence > 41`, then follows live events. Each SSE `id` is the event sequence.

```text
id: 42
event: operation_state
data: {"operation_id":"memory-release-20260803","sequence":42,"to_state":"committed",...}
```

Persist the last processed ID only after applying the event. Reconnect with that ID as `after`; deduplicate by `(operation_id, sequence)`.

## Dependency scheduling

Dependencies identify prerequisite operation receipts. An accepted operation with unfinished dependencies becomes `blocked` and lists them in `blocked_by`. It resumes when prerequisites commit. A rejected dependency prevents silent success and remains visible in the receipt history.

## Tested use cases

- Same ID and same hash: exact receipt replay without duplicate memory.
- Same ID and different hash: `409`, preserving the original request.
- Lost POST response: GET by ID reconciles authoritative state.
- Long logical memory: persisted tokenizer parts resume independently and commit one memory.
- SSE reconnect: durable history begins strictly after the caller’s last sequence.

