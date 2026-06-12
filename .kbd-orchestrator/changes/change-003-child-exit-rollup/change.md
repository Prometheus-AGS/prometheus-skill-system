---
id: change-003-child-exit-rollup
title: kbd-child-exit skill + progress rollup
phase: child-loops-and-capabilities
gaps: [C3]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-child-exit/SKILL.md
  - skills/process/kbd-process-orchestrator/skills/kbd-child-exit/kbd-child-exit.sh
  - skills/process/kbd-process-orchestrator/shared/lib/rollup.sh
  - skills/process/kbd-process-orchestrator/references/schemas/progress.schema.json
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-child-exit.sh
---

# change-003 — Child exit + rollup

## Context

An outer loop spawning an inner loop needs a graceful return: the child writes a
handoff, its progress aggregates up to the parent, and control pops back. None
of this exists today.

## Scope

In:

- New `KBD/skills/kbd-child-exit/` (SKILL.md + kbd-child-exit.sh):
  - Precondition: child `reflection.md` exists (else refuse with
    "/kbd-reflect the child first").
  - Write `handoff-out.md` in the child dir: deliverables (paths), goal
    completion status, unresolved items, recommendations to the parent.
  - Call rollup; pop the last element of path[]; restore the parent's
    currentTask/exactNextCommand (next pending from parent plan).
  - Fire `child:after`; emit Progress Signals + Declares them.
- New `KBD/shared/lib/rollup.sh` — `kbd_rollup_children <node-dir>`: recompute a
  `children` block in the node's progress.json from each child's progress.json
  ({status, changes_completed, changes_total, handoff, completed_at}); recurse
  up the ancestor chain.
- `progress.schema.json`: add optional `children` object (non-breaking; top
  level is open).
- New `test-child-exit.sh`: child with reflection → exit writes handoff-out,
  parent progress.json gains a children block with correct counts, path[]
  popped; refuse when child has no reflection.

## Tasks

- [ ] 1. Write rollup.sh + progress.schema.json children block
- [ ] 2. Write kbd-child-exit SKILL.md + .sh (handoff-out, rollup, pop, signals)
- [ ] 3. Write test; run green

## Verification

Test green; exiting a child rolls its completion into the parent's children
block and pops path[]; refuses without a child reflection.
