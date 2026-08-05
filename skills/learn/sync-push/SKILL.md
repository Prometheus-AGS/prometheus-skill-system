---
name: sync-push
description: Push a sync domain to connected P2P peers. Triggers CRDT delta exchange for skill indexes, learner models, or other enabled domains.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [sovereign-sync, p2p, sync, push, crdt, learn, loro]
---

# /sync-push

Push the current state of a sync domain to all connected peers via the
sovereign-sync P2P network. Uses Loro 1.13 CRDT snapshot + delta export
so only changed operations are transmitted.

## When to use

Trigger on:
- "sync my skills", "push to peers", "sync learning progress", "share skill index"
- After completing a Feynman loop (`learn-grade` passes) to propagate mastery updates
- After installing new skills to advertise them to the group
- Explicitly requested: "sync now", "push sync"

## Prerequisites

A `sovereign-sync` daemon must be running and reporting state `Ready`. That is
a rollup over the internal `Connected`/`Syncing`/`Idle` P2P states — the REST
API surfaces only `Ready`, never those three names.

Check health first — this resolves the transport for you and exits nonzero when
the node is unreachable:

```bash
sovereign-sync --mode status
```

Then confirm the state. Note `/health` returns only `{service, status, version}`
— the state lives on the sync-status endpoint, not on `/health`:

```bash
curl -s --unix-socket "$SOCK" http://localhost/api/v1/sync/status \
  | jq '{node_state, transport: .transport.state, peers: (.peers | length)}'
```

### Transport: Unix socket by default

The daemon listens on a **Unix domain socket**, NOT loopback TCP, so every REST
call below needs `--unix-socket "$SOCK"`. See [`/sync-status`](../sync-status/SKILL.md)
for the full transport contract.

```bash
SOCK="${HOME}/Library/Application Support/prometheus/run/sovereign-sync.sock"  # macOS
# SOCK="${HOME}/.local/share/prometheus/run/sovereign-sync.sock"               # Linux
```

Only when the daemon was started with `--tcp` does `http://127.0.0.1:7892`
apply, and it then also requires the `--token-file` bearer token.

## Sync domains

| Domain | Content | Privacy class |
|--------|---------|---------------|
| `skill-index` | Skill name + description keyword index | Shareable |
| `learner-model` | Mastery levels, FSRS cards, gap records | Shareable |
| `surreal-memory` | Entity graph and scoped memories | **LocalOnly** — never synced |

The `surreal-memory` domain is **always excluded from sync** regardless of what
the caller requests. This is enforced structurally in `SyncManifest` via
`PrivacyClass::LocalOnly`.

## Instructions

### Push skill index

```bash
curl -s --unix-socket "$SOCK" -X POST http://localhost/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "skill-index"}' | jq .
```

### Push learner model

```bash
curl -s --unix-socket "$SOCK" -X POST http://localhost/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "learner-model"}' | jq .
```

### Stream progress via AG-UI

For interactive harnesses (Claude Code, Tauri apps), stream push progress
via the AG-UI endpoint:

```bash
curl -s --unix-socket "$SOCK" -X POST http://localhost/api/v1/stream \
  -H 'Content-Type: application/json' \
  -d '{"kind": "SyncPush", "domain": "skill-index"}' \
  --no-buffer
```

SSE events arrive in this order:
1. `task_accepted` — task queued
2. `progress` — bytes serialized, peers contacted
3. `done` — delta broadcast complete
4. `error` — failure details if push fails

### Via MCP (stdio mode)

When `sovereign-sync --mode mcp` is configured in `mcp-servers.json`:

```
Tool: sync-push
Arguments: { "domain": "skill-index" }
```

## Conflict resolution

All sync domains use Loro CRDT. Concurrent edits on multiple devices are
automatically merged. No manual conflict resolution is needed. The merge is:
- **skill-index**: last-write-wins per skill entry
- **learner-model**: per-card merge using FSRS timestamp + PFA mastery formula

## Privacy guarantee

The sovereign-sync daemon's `SyncManifest` enforces `PrivacyClass::LocalOnly`
for `SurrealMemory`. A push call requesting `surreal-memory` will be rejected
at the server with HTTP 400. This is not convention — it is enforced in code.

No KB content (Dify, palace RAG, local markdown) is included in sync payloads.
Only indexed metadata (skill names, descriptions, mastery levels) is transmitted.
