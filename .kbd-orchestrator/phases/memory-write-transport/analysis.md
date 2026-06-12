# Analysis — memory-write-transport

Engineering-landscape research for the "bash can't write to surreal-memory"
gap. **The assessment's framing was incomplete** — research found a much
simpler fix than any of its 5 candidates.

## The decisive finding: a plain REST write API exists

The surreal-memory server exposes **two** transports side by side
(`tools/surreal-memory-server/src/mcp/http.rs:49,52-54`):
- **SSE MCP** — `GET /sse` (stream) + `POST /messages` (session) — what the bash
  bridge wrongly POSTed JSON-RPC at (405).
- **Plain REST API** under `/api/v1/...` — what the in-repo Rust client
  (`prometheus_learn::memory::SurrealMemoryClient`) actually uses for
  ping/search/create.

The server's own contract (`contracts.rs`) declares the REST route:

```
POST /api/v1/memory   body: AddMemoryRequest { content: String, user_id?, agent_id?, session_id?, categories?[] }
```

**LIVE-VERIFIED:** a fire-and-forget
`curl -X POST http://localhost:23001/api/v1/memory -d '{"content":"…","user_id":"prometheus-skill-pack"}'`
returns **HTTP 201** with the embedded record (id `jy4hbqpuelc8w4rxi3us`). No
SSE session, no handshake — exactly what a bash hook needs.

This **dissolves the assessment's hard problem**: bash CAN write directly, just
to the REST route, not the SSE stream. None of candidates #1 (Rust CLI write
subcommand), #2 (bash SSE handshake), #4 (liter-llm proxy), or #5 (direct
SurrealDB) are needed for the primary write path.

## Coverage map (REST vs MCP-tool-only)

The server exposes **10 MCP tools** but only **10 REST routes**, and they do
NOT fully overlap:

| Bridge function | REST route | Status |
|---|---|---|
| `mem_add_memory` | `POST /api/v1/memory` | ✅ REST — verified 201 |
| `mem_create_task_stream` | (none) | ❌ MCP-tool-only — no REST route |
| `mem_add_task_step` | (none) | ❌ MCP-tool-only |
| `mem_complete_step` | (none) | ❌ MCP-tool-only |

Also REST-available (not currently used by the bridge): `POST /api/v1/entities`,
`/entities/relations`, `/mindmaps`, `PUT /api/v1/memory/{id}`.

## What this means for the fix

- **`add_memory` — the primary, most-used write** (Phase 4's `reflect:end`
  persistence) — gets a clean, direct REST fix: change `_mem_call` /
  `mem_add_memory` to POST `/api/v1/memory` and accept 200/201.
- **Task-stream functions have no REST route** — they stay MCP-tool-only.
  Options: keep outbox-buffering (and have an agent-drain), OR make them
  best-effort no-ops with a notice (task-streams are a nice-to-have telemetry
  layer, not the durable learning). The durable learning is `add_memory`.
- **`memory-outbox-flush.sh` (T2)** must be fixed regardless: today it re-POSTs
  via the broken `_mem_call`. After the REST fix, `add_memory` outbox lines
  drain correctly; task-stream lines still can't (document this, or drop them
  from the outbox).
- **`mem0-compress.sh`** has the same broken SSE POST — point it at the right
  endpoint or accept it's MCP-tool-only (`compress_memories` has no REST route
  either — check).

## prometheus-cli IS editable (the assessment's open question, answered)

`tools/prometheus-cli` is **NOT a submodule, NOT gitignored, has its own
`crates/` source here** (`.gitmodules` lists only surreal-memory-server,
prometheus-knowledge, liter-llm — not prometheus-cli). So a Rust CLI write
subcommand (`prometheus memory add`) IS in scope as a *durable* secondary path
— but the REST fix to the bash bridge is simpler and sufficient for the hook
use case, so the CLI addition is optional/nice-to-have, not required.

## health-probe (already fixed this deploy)

`/health` not `/healthz` — fixed in commit fa7e18e. The REST base is the same
host:port the health probe derives, so `mem_available` and the write path now
agree on the endpoint.

## Risks / watch-items

- `prometheus memory stats` errors with "missing field `name`" — CLI/server
  schema drift; out of scope for this fix but flagged.
- The REST `AddMemoryRequest` omits some fields the MCP `add_memory` tool
  accepts (e.g. no `metadata`/`importance` in the REST struct) — confirm the
  REST route covers what the bridge needs (content + user_id is enough for the
  `[GLOBAL]`-vs-project scoping the bridge does).
- A REST 4xx (e.g. validation) should still fall to the outbox — keep the
  graceful-degradation contract; only treat 2xx as success.
