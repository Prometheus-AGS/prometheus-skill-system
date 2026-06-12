# Plan — memory-write-transport

Backend: **native-kbd** (consistent with all ~30 prior changes this effort;
no project.json pins openspec). Two changes, ordered so the bridge fix lands
and is verified before the consumers that depend on it are corrected. Each
carries a `scope:` declaration.

Decisions carried from analysis (decision-log.md):
- D-001 adopt cand-001 — REST `POST /api/v1/memory` is the write path (verified 201).
- D-002 task-streams/compress have NO REST route → MCP-tool-only, stay outbox.
- D-003 prometheus-cli editable but a Rust subcommand is NOT needed — reference only.

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-rest-write-path | T1 | `library: cand-001`. Make `_mem_call` in `shared/scripts/lib/memory-bridge.sh` dispatch by tool name to the REST API: `add_memory`→`POST <base>/api/v1/memory`, and (future-proof) `create_entity`→`/api/v1/entities`, `create_relation`→`/api/v1/entities/relations`; accept **200 or 201** as success. Tools with no REST route (`create_task_stream`, `add_task_step`, `complete_step`, `compress_memories`) return non-zero so callers outbox them (unchanged contract). Derive `<base>` = scheme://host:port from `MEM_URL` (same as `mem_available`). Update `test-memory-bridge.sh`: the fake-curl asserts the REST URL + body for add_memory, and that a task-stream call still outboxes. LIVE-verify a real add_memory POST → 201 and a `search_memories` round-trip; clean up the probe record. |
| 2 | change-002-outbox-flush-and-compress | T2 | Fix `memory-outbox-flush.sh` so it drains via the (now REST-capable) `_mem_call`: `add_memory` lines flush to the server; lines whose method has no REST route (task-stream/compress) are KEPT in the outbox with a one-line notice rather than re-failing silently (or dropped if telemetry-only — decide in change). Fix `mem0-compress.sh`: `compress_memories` has no REST route, so either (a) detect-and-skip with a clear notice, or (b) route through the agent — document it's MCP-tool-only, don't POST JSON-RPC to the SSE stream (the current 405 path). Update/extend `test-memory-writeback.sh` to cover: REST-drainable line flushes, MCP-only line is retained, endpoint-down preserves all. |

Out of scope (recorded, not built): a `prometheus memory add` Rust subcommand
(cand-002, reference) — the REST fix is sufficient for the hook use case; revisit
only if unattended task-stream writes become required. Direct SurrealDB writes
(cand-005, rejected — loses embeddings/scoping).

Completion per change: change.md tasks checked, tests green, LIVE round-trip
verified against the running server (not just fake-curl), commit. Phase end:
`npm run validate:strict`, full memory-test sweep, reflection gated. First:
`/kbd-apply change-001-rest-write-path`.
