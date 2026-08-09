---
name: kbd-goal-check
version: '1.0.0'
license: MIT
description: >
  Kimi Code goal-condition evaluator skill. After each Kimi turn, checks
  whether the active phase's stopping condition has been met. Returns PASS
  (with evidence) or CONTINUE (with next action from TASKS.md). Implements
  the maker≠evaluator pattern on Kimi Code, where /goal next is a queue
  (not a condition-based loop) and requires an external evaluator.
metadata:
  author: Prometheus AGS
  category: process
  tags: [process, goal, evaluator, kimi, loop, condition-check]
triggers:
  keywords:
    - /kbd-goal-check
    - check goal condition
    - evaluate goal
    - is goal met
    - goal stopping condition
  semantic: >
    Used after each Kimi Code turn to evaluate whether the current phase's
    stopping condition has been met, without the implementer agent grading
    its own work.
---

# /kbd-goal-check

Evaluate whether the active KBD goal phase's stopping condition is met.
Invoke this after each execution turn — before deciding whether to continue
the loop or advance to the next phase.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting kbd-goal-check — <goal slug>
```

When the stopping-condition assessment is complete, emit:

```
Completed kbd-goal-check — <goal slug> (status: <met|not-met>)
```

Emit to plain response text — no tool call needed.

## When to Use

- After each Kimi Code execution turn within a `/kbd-goal` phase
- Whenever you need an impartial check of goal completion
- Before calling `/goal next <next-phase-condition>` in Kimi Code

## What This Does

1. **Reads the stopping condition** from `.kbd-orchestrator/goals/<slug>/goal.json`
   → `phases[active_phase].stopping_condition`
2. **Reads current STATE.md** to understand what has been completed
3. **Checks the condition** against evidence files (test output, lint log,
   file content)
4. **Returns PASS or CONTINUE** with specific evidence

## Output Format

```
PASS
Evidence: All 23 tests in tests/ exit 0 (see .goal-run/test-output.txt line 1).
Lint: 0 errors (see .goal-run/lint-output.txt).
Stopping condition: "all tests pass, lint clean" — SATISFIED.
```

or

```
CONTINUE
Next action: task-004 (implement error handling for edge case in auth.go)
Blocker: 3 tests still failing: TestAuthExpiry, TestTokenRefresh, TestLogout
See: .goal-run/test-output.txt lines 47, 83, 112
```

## Steps to Follow

### 1. Find the active goal

```bash
# Read current waypoint for active goal slug
cat .kbd-orchestrator/current-waypoint.json | grep goal_slug
```

If no `goal_slug` is set, read `.kbd-orchestrator/goals/` to find the most
recently modified `goal.json` and check its `status == "running"`.

### 2. Read the stopping condition

```bash
cat .kbd-orchestrator/goals/<slug>/goal.json | jq -r '.phases[] | select(.status == "running") | .stopping_condition'
```

### 3. Read STATE.md

```bash
cat .kbd-orchestrator/goals/<slug>/STATE.md
```

### 4. Check evidence

Read the evidence files referenced by the stopping condition. Common patterns:

- **"all tests pass"** → read `.goal-run/test-output.txt`, check exit code
- **"lint clean"** → read `.goal-run/lint-output.txt`, count errors
- **"TASKS.md complete"** → read TASKS.md, count unchecked `[ ]` entries
- **"file X contains Y"** → read the file directly

### 5. Return verdict

**If PASS:**
- Write your PASS verdict and evidence to the response
- Update `.kbd-orchestrator/goals/<slug>/STATE.md`:
  append a `## Phase Complete` section: `<timestamp>: PASS — <evidence-summary>`
- The orchestrator will advance to the next phase or end the goal

**If CONTINUE:**
- Write your CONTINUE verdict with next action
- Identify the next unchecked task in TASKS.md
- The implementer agent will continue work on the next turn

## Kimi Code Integration

In Kimi Code, invoke this skill after every execution turn:

```
/kbd-goal-check
```

When this returns PASS, advance to the next phase:

```
/goal next <next-phase-stopping-condition>
```

When this returns CONTINUE, the current turn's work is insufficient — Kimi
continues with the next action identified by the evaluator.

## Separation of Concerns

This skill enforces **maker ≠ evaluator**: the agent that did the work should
NOT grade whether the goal is met. This skill is always a separate evaluation
step from the implementation turn.

Do NOT call this skill from within the same turn that did the implementation.
The orchestrator calls it as a separate turn.

## References

- [Goal Directory Layout](../kbd-goal/references/goal-directory-layout.md)
- [Kimi Platform Guide](../kbd-goal/references/platforms/kimi.md)
- [Goal Evaluator Agent](../../agents/kbd-goal-evaluator.md)
