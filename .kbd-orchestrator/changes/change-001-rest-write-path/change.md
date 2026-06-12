---
id: change-001-rest-write-path
title: Route bash memory writes through the REST API (POST /api/v1/memory)
phase: memory-write-transport
gaps: [T1]
library: cand-001
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - shared/scripts/lib/memory-bridge.sh
  - shared/scripts/tests/test-memory-bridge.sh
---

# change-001 — REST write path

## Context

`_mem_call` POSTs JSON-RPC `tools/call` to `MEM_URL` (`…/mcp/sse`), which is a
GET-only SSE stream → 405 → every bash write outboxes. Analysis verified the
server ALSO exposes a plain REST API: `POST /api/v1/memory` with
`AddMemoryRequest{content,user_id,…}` → 201 (the same REST API the in-repo Rust
`SurrealMemoryClient` uses). Re-point the bridge at REST.

## Scope

In:

- `_mem_call <tool> <args-json>` in `memory-bridge.sh`: dispatch by tool name to
  the REST route instead of JSON-RPC to the SSE stream:
  - `add_memory` → `POST <base>/api/v1/memory` (body = the args JSON as-is:
    `{content,user_id,...}`)
  - `create_entity` → `POST <base>/api/v1/entities` (future-proof; not currently
    called but keeps the map complete)
  - `create_relation` → `POST <base>/api/v1/entities/relations`
  - any other tool (`create_task_stream`, `add_task_step`, `complete_step`,
    `compress_memories`) → **return non-zero** (no REST route) so the caller
    outboxes it. Document this in a comment.
  - `<base>` = scheme://host:port derived from `MEM_URL` (reuse the sed already
    in `mem_available`). Accept **200 OR 201** as success.
- `mem_add_memory` already builds `{content,user_id}` — no change needed beyond
  `_mem_call` now sending it to REST. Confirm the 4 wrapper functions are
  unchanged (they still call `_mem_call <tool> <args>` then outbox on failure).
- `test-memory-bridge.sh`: extend the fake-curl shim to assert
  (a) add_memory POSTs to `…/api/v1/memory` with the right body, success on 201;
  (b) a task-stream call (`mem_create_task_stream`) still writes the outbox
  (no REST route → _mem_call returns non-zero).

## Tasks

- [ ] 1. Rewrite `_mem_call` to dispatch by tool name to REST routes (200/201 = ok); non-routed tools return non-zero
- [ ] 2. Extend test-memory-bridge.sh (REST add_memory success + task-stream outbox)
- [ ] 3. Run the test green; LIVE-verify a real add_memory POST → 201 + search_memories round-trip, then delete the probe record

## Verification

`bash shared/scripts/tests/test-memory-bridge.sh` green; a live
`mem_add_memory "…" "prometheus-skill-pack"` reaches the server (outbox stays
empty) and is retrievable via `search_memories`; a live
`mem_create_task_stream` still outboxes (no REST route).
