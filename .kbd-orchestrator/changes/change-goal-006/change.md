---
id: change-goal-006
title: Creation Loop Enhancement
phase: goal-loop-support
subphase: A (core)
depends_on: [change-goal-005]
agent: claude-code
status: done
scope:
  - skills/process/kbd-goal/references/templates/creation-phase.md
  - agents/kbd-task-verifier.md
  - kbd-goal/SKILL.md
---

# change-goal-006 — Creation Loop Enhancement

## Problem

The existing KBD execute loop has no TASKS.md decomposition, no per-task verifier agent, no STATE.md tracking, and no escalation path for tasks that repeatedly fail.

## Solution

Build a Creation phase template that decomposes SPEC.md into TASKS.md, runs a per-task implement→verify loop with a dedicated verifier agent, tracks state in STATE.md, and auto-escalates after 3 consecutive failures.

## Files

- `skills/process/kbd-goal/references/templates/creation-phase.md` (CREATE)
- `agents/kbd-task-verifier.md` (CREATE)

## Tasks

- Write `creation-phase.md` template documenting the 5-step build loop
- Define `TASKS.md` format: `[ ] task-NNN: <description> [acceptance criteria]`
- Define `STATE.md` format: `{completed: N, total: N, active_task, tasks: [{id, status, fail_count}], escalations[], promotions[], budget_summary}`
- Write `agents/kbd-task-verifier.md`: reads SPEC.md criteria + task description + test/lint output → PASS or FAIL + failure reason
- Document worktree isolation: when `tool == claude-code`, use `--worktree` per task; document fallback for other platforms
- Document auto-escalation: `fail_count >= 3` → write to `STATE.md → escalations[]`, pause loop for human
- Document auto-promotion trigger: sets up for change-goal-012
- Update `kbd-goal/SKILL.md` with Creation Phase section

## Tasks

- [x] 1. Write `creation-phase.md` template documenting the 5-step build loop
- [x] 2. Define `TASKS.md` format: `[ ] task-NNN: <description> [acceptance criteria]`
- [x] 3. Define `STATE.md` format: `{completed: N, total: N, active_task, tasks: [{id, status, fail_count}], escalations[], promotions[], budget_summary}`
- [x] 4. Write `agents/kbd-task-verifier.md`: reads SPEC.md criteria + task description + test/lint output → PASS or FAIL + failure reason
- [x] 5. Document worktree isolation: when `tool == claude-code`, use `--worktree` per task; document fallback for other platforms
- [x] 6. Document auto-escalation: `fail_count >= 3` → write to `STATE.md → escalations[]`, pause loop for human
- [x] 7. Document auto-promotion trigger: sets up for change-goal-012
- [x] 8. Update `kbd-goal/SKILL.md` with Creation Phase section
