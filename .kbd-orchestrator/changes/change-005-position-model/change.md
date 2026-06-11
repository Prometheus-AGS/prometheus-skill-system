---
id: change-005-position-model
title: Unified derived position model (position.json)
phase: position-and-handoff-guarantee
gaps: [F2]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/shared/lib/position.sh
  - skills/process/kbd-process-orchestrator/references/schemas/position.schema.json
  - skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md
  - shared/scripts/lib/waypoint-render.sh
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-position-sync.sh
---

# change-005 — Unified derived position model

## Context

The user's lost-place problem is rooted in fragmented state: waypoint +
per-phase progress.json + foreign `.evolver/`/`.zeespec/` dirs, with no single
machine-readable tree. position.json is DERIVED (never accumulated) so it can
be stale by at most one write but can never diverge.

## Scope

In:

- New `KBD/shared/lib/position.sh` — `kbd_position_sync`:
  - Rebuilds `.kbd-orchestrator/position.json` from waypoint (+ childPointer),
    each node's progress.json, and the active change's task surface when
    available. Atomic write.
  - Read-only ingest adapters: if `.evolver/` or `.zeespec/` exist, attach
    `annotations[]` entries (source, ref, one-line summary). Never writes there.
  - `cursor[]` = flattened active path (phase[, child][, change][, task:i]).
- New `KBD/references/schemas/position.schema.json`.
- `kbd-status` SKILL.md: render position.json tree first (fallback waypoint).
- `shared/scripts/lib/waypoint-render.sh`: prefer position.json cursor when
  present (one-line breadcrumb), keep waypoint fallback.
- Call sites: document `kbd_position_sync` invocation in waypoint.sh write path
  doc and kbd-apply end-task (instructional edits only this phase; deep wiring
  follows in the lifecycle phase).
- New test `test-position-sync.sh`: fixture orchestrator tree → assert cursor,
  progress rollup, annotation ingest, derive-twice idempotency.

## Tasks

- [x] 1. Write position.sh + schema
- [x] 2. kbd-status rendering edit
- [x] 3. waypoint-render position.json preference
- [x] 4. Test; run green

## Verification

Test green; running `kbd_position_sync` in this repo produces a position.json
whose cursor matches the live waypoint.
