---
id: rest-api
title: REST API Reference
sidebar_label: REST API
---

# REST API Reference

Sovereign Sync exposes a REST API on `http://127.0.0.1:7892` when running in `daemon` or
`server` mode.

## GET /health

Returns node health and version.

```bash
curl -s http://127.0.0.1:7892/health | jq .
```

**Response:**

```json
{
  "status": "ok",
  "service": "sovereign-sync",
  "version": "0.1.0"
}
```

## GET /api/v1/sync/status

Returns node state, connected peers, and domain privacy summary.

```bash
curl -s http://127.0.0.1:7892/api/v1/sync/status | jq .
```

**Response:**

```json
{
  "node_state": "Connected",
  "peers": ["a1b2c3...", "d4e5f6..."],
  "domains": {
    "kbd-orchestrator": { "privacy": "sync_encrypted_only", "peers": 2 },
    "open-spec":        { "privacy": "sync_encrypted_only", "peers": 2 },
    "surreal-memory":   { "privacy": "local_only",          "peers": 0 },
    "learner-model":    { "privacy": "sync_encrypted_only", "peers": 2 }
  }
}
```

**Node states:**

| State | Meaning |
|-------|---------|
| `Disconnected` | Not yet joined the gossip network |
| `Bootstrapping` | Connecting to bootstrap peers |
| `Connected` | At least one peer reachable |
| `Syncing` | Actively exchanging CRDT deltas |
| `Idle` | Connected, nothing to exchange |

## GET /api/v1/sync/peers

Lists connected peer node IDs.

```bash
curl -s http://127.0.0.1:7892/api/v1/sync/peers | jq .
```

**Response:**

```json
{
  "peers": [
    { "node_id": "a1b2c3...", "addr": "192.168.1.42:7892" }
  ]
}
```

## GET /api/v1/skills/search?q=&lt;query&gt;&amp;limit=&lt;n&gt;

Keyword search over the local skill index. Default limit is 10.

```bash
curl -s "http://127.0.0.1:7892/api/v1/skills/search?q=feynman&limit=5" | jq .
```

**Response:**

```json
{
  "query": "feynman",
  "count": 3,
  "results": [
    { "name": "feynman-loop", "description": "Core Feynman PMPO loop" },
    { "name": "learn-grade",  "description": "Sycophancy-corrected external grader" },
    { "name": "learn-retain", "description": "FSRS-6 spaced retrieval" }
  ]
}
```

## POST /api/v1/sync/push

Queues a sync domain for broadcast to all connected peers.

```bash
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "learner-model"}' | jq .
```

**Response:**

```json
{
  "status": "queued",
  "domain": "learner-model"
}
```

**Privacy protection:** Requesting `"domain": "surreal-memory"` returns HTTP 400 — the
`LocalOnly` invariant is enforced at the REST layer as well as the CRDT layer.

## POST /api/v1/stream

Start an AG-UI task stream. Returns Server-Sent Events.

See [AG-UI SSE Reference](./ag-ui-sse) for full documentation.

## GET /api/v1/stream/ping

SSE ping — returns a single `ping` event and closes.

```bash
curl -s http://127.0.0.1:7892/api/v1/stream/ping
```

**Response:**

```
data: {"type":"ping"}

```
