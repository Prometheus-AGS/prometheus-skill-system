---
id: change-005-apply-position-depth
title: Extend kbd-apply + position.sh to arbitrary child depth
phase: child-loops-and-capabilities
gaps: [C1]
priority: P2
effort: S
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh
  - skills/process/kbd-process-orchestrator/shared/lib/position.sh
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-apply-native.sh
---

# change-005 — apply + position at depth

## Context

kbd-apply `_phase_dir` and position.sh resolve only one child level. With
arbitrary nesting (changes 001-003) they must walk path[] to the active node.

## Scope

In:

- `kbd-apply.sh` `_phase_dir`: resolve the active node via kbd_current_node_dir
  (walk path[]) instead of the hard-coded one-child-level lookup. Source
  waypoint.sh's resolver; keep the existing behavior for depth-1 (regression).
- `position.sh`: extend cursor + tree construction to walk path[] for the full
  ancestor chain (today it handles parent + one child).
- Extend `test-kbd-apply-native.sh` (or add a case): drive a change inside a
  depth-2 child; assert progress.json and position.json resolve to the
  grandchild node.

Out: nothing — this closes the depth story for the execution surfaces.

## Tasks

- [x] 1. kbd-apply _phase_dir walks path[]
- [x] 2. position.sh cursor/tree walks path[]
- [x] 3. Add depth-2 apply+position assertion; run green (depth-1 stays green)

## Verification

Test green; a task applied inside a grandchild updates the grandchild's
progress.json and position.json; existing depth-0/1 apply tests unaffected.
