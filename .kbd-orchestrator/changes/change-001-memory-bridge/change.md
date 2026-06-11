---
id: change-001-memory-bridge
title: memory-bridge.sh — surreal-memory writes with mandatory outbox fallback
phase: memory-and-karpathy
gaps: [M1, M2]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/lib/memory-bridge.sh
  - .gitignore
  - shared/scripts/tests/test-memory-bridge.sh
---

# change-001 — memory-bridge.sh

## Context

Memory reads are automated (kbd-memory-recall) but writes are manual prose, so
cross-project learning does not actually happen. surreal-memory timed out
during this project's own assessment, so any write path MUST degrade to a
durable outbox rather than losing the write.

## Scope

In:

- New `shared/scripts/lib/memory-bridge.sh` (sourceable; no import side effects):
  - `mem_available` — curl `--max-time 2` probe of
    `${SURREAL_MEMORY_URL:-http://localhost:23001/mcp/sse}`.
  - `mem_add_memory <content> <user_id>` — JSON-RPC tools/call add_memory.
  - `mem_create_task_stream <name>`, `mem_add_task_step <stream> <desc>`,
    `mem_complete_step <stream> <step>`.
  - Pattern reused from mem0-compress.sh (curl POST + python parse).
  - **Every failure (endpoint down, curl missing, non-200) appends the intended
    call as one JSON line to `.kbd-orchestrator/memory-outbox.jsonl` and returns
    0.** Never blocks.
  - Scoping helper: content lines/sections prefixed `[GLOBAL]` →
    user_id="global"; everything else → project scope (KBD_PROJECT_NAME or
    "prometheus-skill-pack").
- `.gitignore`: add `.kbd-orchestrator/memory-outbox.jsonl`.
- New `shared/scripts/tests/test-memory-bridge.sh`: PATH-shimmed fake `curl`
  (success + failure variants) — assert add_memory POST body carries correct
  user_id; failure path writes an outbox line; mem_available probe; scoping
  helper routes [GLOBAL] vs project.

## Tasks

- [x] 1. Write memory-bridge.sh (functions + outbox fallback + scoping)
- [x] 2. .gitignore the outbox
- [x] 3. Write test with fake curl; run green

## Verification

Test green; with a failing fake curl, every function returns 0 and the outbox
grows; with a success fake, the add_memory body has the right user_id.
