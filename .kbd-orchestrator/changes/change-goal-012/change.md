---
id: change-goal-012
title: Inner-Loop Auto-Promotion
phase: goal-loop-support
subphase: B (integration)
depends_on: [change-goal-006, "existing `kbd-new-child`"]
agent: claude-code
status: done
scope:
  - scripts/kbd-goal-promote.sh
  - skills/process/kbd-goal/references/templates/creation-phase.md
---

# change-goal-012 — Inner-Loop Auto-Promotion

## Problem

When a task in the Creation loop fails repeatedly (3+ times), the current pattern stalls indefinitely or requires manual intervention. Complex sub-tasks need to be automatically promoted to child KBD phases where they can be fully assessed, analyzed, and planned before retrying.

## Solution

Build `kbd-goal-promote.sh` that detects the promotion trigger in STATE.md, calls `kbd-new-child`, writes `handoff-in.md` with full context, and marks the parent task as promoted.

## Files

- `scripts/kbd-goal-promote.sh` (CREATE)
- `skills/process/kbd-goal/references/templates/creation-phase.md` (UPDATE: add promotion section)

## Tasks

- Write `kbd-goal-promote.sh`: read STATE.md; find tasks with `fail_count >= 3` or `NEEDS_CHILD_PHASE: true`; for each: call `kbd-new-child <task-slug>`, write `handoff-in.md` with task description + last 3 failure reasons + SPEC.md acceptance criteria
- Update parent `TASKS.md`: mark promoted task as `[~] task-NNN: promoted to child: <task-slug>`
- Update `STATE.md`: add `promotions[]` entry with `{task_id, child_phase, promoted_at}`
- Integrate into Creation loop tick: after each task attempt, check promotion condition before deciding continue/stop
- Update `creation-phase.md` template with promotion section
- Document: agent can also set `NEEDS_CHILD_PHASE: true` in STATE.md to force early promotion

## Tasks

- [x] 1. Write `kbd-goal-promote.sh`: read STATE.md; find tasks with `fail_count >= 3` or `NEEDS_CHILD_PHASE: true`; for each: call `kbd-new-child <task-slug>`, write `handoff-in.md` with task description + last 3 failure reasons + SPEC.md acceptance criteria
- [x] 2. Update parent `TASKS.md`: mark promoted task as `[~] task-NNN: promoted to child: <task-slug>`
- [x] 3. Update `STATE.md`: add `promotions[]` entry with `{task_id, child_phase, promoted_at}`
- [x] 4. Integrate into Creation loop tick: after each task attempt, check promotion condition before deciding continue/stop
- [x] 5. Update `creation-phase.md` template with promotion section
- [x] 6. Document: agent can also set `NEEDS_CHILD_PHASE: true` in STATE.md to force early promotion
