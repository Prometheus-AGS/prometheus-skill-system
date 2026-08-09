# KBD Goal Continuation Prompt (Codex)

<!-- This file is injected by Codex's goal loop after each turn when the stopping
condition has not yet been met. It instructs Codex to continue working. -->

You are in an active KBD goal loop. The goal has not yet been satisfied.

## Current State

Read `.kbd-orchestrator/goals/{{GOAL_SLUG}}/STATE.md` to understand current progress.

## What to Do Next

1. Read `STATE.md` → find the first task with status `in_progress` or the next
   unchecked `[ ]` task in `TASKS.md`.
2. Implement the task: write code and tests.
3. Run the tests and capture output to `.goal-run/{{TASK_ID}}/test-output.txt`.
4. Run the linter and capture output to `.goal-run/{{TASK_ID}}/lint-output.txt`.
5. Update `STATE.md` with your progress.
6. Do NOT mark yourself as complete — the evaluator checks completion.

## Stopping Condition

{{STOPPING_CONDITION}}

You will continue working until this condition is met or the token budget is
exhausted. Do not give up early. Do not declare completion — the evaluator does that.

## Context Files

- Goal definition: `.kbd-orchestrator/goals/{{GOAL_SLUG}}/goal.json`
- Specification: `.kbd-orchestrator/goals/{{GOAL_SLUG}}/SPEC.md`
- Task list: `.kbd-orchestrator/goals/{{GOAL_SLUG}}/TASKS.md`
- Execution state: `.kbd-orchestrator/goals/{{GOAL_SLUG}}/STATE.md`
