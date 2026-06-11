---
id: change-002-position-sync-wiring
title: Wire kbd_position_sync into kbd-apply end-task
phase: canonical-lifecycle
gaps: [G4]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-apply-native.sh
---

# change-002 — Wire position sync into the apply loop

## Context

Phase 1 carry-forward CF-5: `kbd_position_sync` derives position.json but is
invoked manually only, so the breadcrumb goes stale during execution. The
apply loop's `end-task` is the natural single call site (every task boundary).

## Scope

In:

- `kbd-apply.sh`: source `shared/lib/position.sh` best-effort alongside
  hooks.sh; in `end-task`, after `sync_progress`, call `kbd_position_sync`
  (guarded `command -v`, never aborts the driver on failure).
- Extend `test-kbd-apply-native.sh` (or add a focused case): drive a fixture
  change through begin-task/end-task and assert position.json's cursor task
  fraction advances and matches progress.json.

Out: deeper waypoint-write-path wiring (documented, not coded, this change).

## Tasks

- [ ] 1. Source position.sh in kbd-apply; call kbd_position_sync in end-task
- [ ] 2. Add position-advance assertion to the native test
- [ ] 3. Run green

## Verification

Driving a fixture change updates position.json task fraction in lockstep with
progress.json; no regression in existing apply tests.
