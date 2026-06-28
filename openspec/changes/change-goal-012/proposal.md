# change-goal-012: Inner-Loop Auto-Promotion

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-006 (Creation loop), existing `kbd-new-child`

## Problem

When a task in the Creation loop fails repeatedly (3+ times), the current pattern stalls indefinitely or requires manual intervention. Complex sub-tasks need to be automatically promoted to child KBD phases where they can be fully assessed, analyzed, and planned before retrying.

## Solution

Build `kbd-goal-promote.sh` that detects the promotion trigger in STATE.md, calls `kbd-new-child`, writes `handoff-in.md` with full context, and marks the parent task as promoted.

## Files

- `scripts/kbd-goal-promote.sh` (CREATE)
- `skills/process/kbd-goal/references/templates/creation-phase.md` (UPDATE: add promotion section)

## Tasks

- [ ] Write `kbd-goal-promote.sh`: read STATE.md; find tasks with `fail_count >= 3` or `NEEDS_CHILD_PHASE: true`; for each: call `kbd-new-child <task-slug>`, write `handoff-in.md` with task description + last 3 failure reasons + SPEC.md acceptance criteria
- [ ] Update parent `TASKS.md`: mark promoted task as `[~] task-NNN: promoted to child: <task-slug>`
- [ ] Update `STATE.md`: add `promotions[]` entry with `{task_id, child_phase, promoted_at}`
- [ ] Integrate into Creation loop tick: after each task attempt, check promotion condition before deciding continue/stop
- [ ] Update `creation-phase.md` template with promotion section
- [ ] Document: agent can also set `NEEDS_CHILD_PHASE: true` in STATE.md to force early promotion
