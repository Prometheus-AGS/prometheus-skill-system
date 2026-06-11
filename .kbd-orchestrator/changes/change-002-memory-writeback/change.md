---
id: change-002-memory-writeback
title: Automatic memory write-back on execute/reflect + outbox flush
phase: memory-and-karpathy
gaps: [M1, M2]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/hooks/hooks.json
  - shared/scripts/memory-writeback.sh
  - shared/scripts/memory-outbox-flush.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-memory-writeback.sh
---

# change-002 — Automatic memory write-back

## Context

memory-bridge.sh (change-001) provides the write primitives; nothing calls
them automatically yet. Wire write-back into the orchestrator's builtin event
bus (the same place kbd-memory-log lives), plus a Claude Code SessionStart
outbox flush.

## Scope

In:

- `KBD/hooks/hooks.json` builtin entries (mode augment, on_failure ignore):
  - `execute:before` → `mem_create_task_stream "kbd:<project>:<phase>"`.
  - `reflect:end` → `mem_add_memory` of the reflection's Delta + Corrective
    Actions, scoped global-vs-project per the [GLOBAL] rule.
  - (A small wrapper script may be needed since builtin actions are single
    commands — add `shared/scripts/memory-writeback.sh` invoked by the entry.)
- New `shared/scripts/memory-writeback.sh` — dual use:
  - As the orchestrator reflect:end action: read the active phase reflection.md,
    extract Delta/Corrective-Actions, call mem_add_memory.
  - As a Claude Code PostToolUse(Write|Edit) on reflection.md (`|| true`): only
    fire when progress.json reflect_gate is absent/passed (never persist a
    rejected reflection).
- New `shared/scripts/memory-outbox-flush.sh` (SessionStart, `|| true`): drain
  `.kbd-orchestrator/memory-outbox.jsonl` when the endpoint is reachable; leave
  it untouched otherwise.
- Wire memory-writeback (PostToolUse) + memory-outbox-flush (SessionStart) into
  root hooks.json.
- New `shared/scripts/tests/test-memory-writeback.sh`: fake curl; assert
  reflect-gate-rejected reflection is NOT persisted; accepted one IS; outbox
  flush drains lines on success and leaves them on failure.

## Tasks

- [x] 1. Write memory-writeback.sh + memory-outbox-flush.sh
- [x] 2. Wire orchestrator builtin entries (execute:before, reflect:end)
- [x] 3. Wire PostToolUse + SessionStart into root hooks.json
- [x] 4. Write test; run green

## Verification

Test green; rejected reflections never reach memory; outbox flush is
idempotent and non-destructive on failure.
