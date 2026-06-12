---
id: change-002-outbox-flush-and-compress
title: Fix outbox-flush to drain via REST; correct mem0-compress transport
phase: memory-write-transport
gaps: [T2]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - shared/scripts/memory-outbox-flush.sh
  - shared/scripts/mem0-compress.sh
  - shared/scripts/tests/test-memory-writeback.sh
---

# change-002 — Outbox flush + mem0-compress

## Context

`memory-outbox-flush.sh` drains by calling the SAME `_mem_call` — before
change-001 that 405'd, so the outbox NEVER drained (only grew). After
change-001, `add_memory` lines drain via REST; lines for MCP-only tools
(task-stream/compress) still can't go through bash and must be handled
honestly. `mem0-compress.sh` independently POSTs JSON-RPC to the SSE stream
(also 405) for `compress_memories`, which has no REST route.

## Scope

In:

- `memory-outbox-flush.sh`: with change-001's REST-capable `_mem_call`,
  `add_memory` lines flush. For lines whose method has no REST route
  (`create_task_stream`/`add_task_step`/`complete_step`/`compress_memories`),
  `_mem_call` returns non-zero → they are KEPT in the outbox today, which would
  make the outbox grow forever with un-drainable lines. Decide + implement:
  either (a) KEEP them with a one-line "N MCP-only lines retained (no REST
  route)" notice, or (b) DROP them as best-effort telemetry. Recommend (b) for
  task-streams (they're transient telemetry) — document the choice.
- `mem0-compress.sh`: `compress_memories` has no REST route. Stop POSTing
  JSON-RPC to `…/mcp/sse` (405). Replace with: detect that compress is
  MCP-tool-only, log a clear "compress is MCP-tool-only — run via agent/MCP, not
  this script" notice, and exit 0 (or remove the broken HTTP fallback and keep
  only the `pk memory compress` path that already precedes it). Keep graceful.
- `test-memory-writeback.sh`: cover (a) a REST-drainable add_memory outbox line
  flushes on success; (b) an MCP-only line is handled per the chosen policy
  (retained-with-notice or dropped); (c) endpoint-down preserves all lines.

## Tasks

- [ ] 1. Fix memory-outbox-flush.sh: drain add_memory via REST; handle MCP-only lines (keep-with-notice or drop) — document the policy
- [ ] 2. Fix mem0-compress.sh: stop the broken SSE POST; compress is MCP-only — notice + graceful exit
- [ ] 3. Extend test-memory-writeback.sh; run green; LIVE-verify outbox drains an add_memory line against the server

## Verification

Test green; an outbox containing an `add_memory` line drains to the live server
on flush; MCP-only lines don't loop forever; mem0-compress no longer 405s.
