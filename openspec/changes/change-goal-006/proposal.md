# change-goal-006: Creation Loop Enhancement

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** A (core)
**Depends on:** change-goal-005

## Problem

The existing KBD execute loop has no TASKS.md decomposition, no per-task verifier agent, no STATE.md tracking, and no escalation path for tasks that repeatedly fail.

## Solution

Build a Creation phase template that decomposes SPEC.md into TASKS.md, runs a per-task implement→verify loop with a dedicated verifier agent, tracks state in STATE.md, and auto-escalates after 3 consecutive failures.

## Files

- `skills/process/kbd-goal/references/templates/creation-phase.md` (CREATE)
- `agents/kbd-task-verifier.md` (CREATE)

## Tasks

- [ ] Write `creation-phase.md` template documenting the 5-step build loop
- [ ] Define `TASKS.md` format: `[ ] task-NNN: <description> [acceptance criteria]`
- [ ] Define `STATE.md` format: `{completed: N, total: N, active_task, tasks: [{id, status, fail_count}], escalations[], promotions[], budget_summary}`
- [ ] Write `agents/kbd-task-verifier.md`: reads SPEC.md criteria + task description + test/lint output → PASS or FAIL + failure reason
- [ ] Document worktree isolation: when `tool == claude-code`, use `--worktree` per task; document fallback for other platforms
- [ ] Document auto-escalation: `fail_count >= 3` → write to `STATE.md → escalations[]`, pause loop for human
- [ ] Document auto-promotion trigger: sets up for change-goal-012
- [ ] Update `kbd-goal/SKILL.md` with Creation Phase section
