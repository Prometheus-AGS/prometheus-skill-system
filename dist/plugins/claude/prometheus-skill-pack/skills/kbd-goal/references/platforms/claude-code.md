# Platform: Claude Code

**Detection:** `$TOOL == "claude-code"` OR `claude` binary in PATH and no other tool env var set.

Claude Code is the reference platform. KBD **augments** native `/goal`, not replaces it.
The key rule: Ideation and Specification phases are always KBD-owned. Creation phases
delegate to native `/goal` where it provides value.

## Routing Decision Table

| Goal Type | Strategy |
|---|---|
| Single-phase Creation with explicit stop | Delegate: `claude /goal --tokens <N> "<stopping_condition>"` |
| Multi-phase (ideation + spec + creation) | KBD orchestrates phases; delegates Creation per-phase |
| Ideation phase | Always KBD (kbd-idea-critic subagent loop) |
| Specification phase | Always KBD (kbd-spec-reviewer subagent loop) |
| Creation phase (within multi-phase) | Delegate: `claude /goal --worktree "<phase-stopping-condition>"` |
| Deployment phase | KBD orchestrates; may delegate specific deploy tasks |

## Single-Phase Creation Delegation

When the user runs:
```
/kbd-goal "fix all auth test failures" --phases creation --stop "all tests in tests/auth pass"
```

`kbd-goal-start.sh` detects `tool == claude-code` and emits:
```bash
claude /goal --tokens 200000 --worktree "all tests in tests/auth/ pass with exit 0 and eslint reports 0 errors"
```

The native `/goal` loop handles the execution. KBD writes `STATE.md` updates
when the goal completes by reading the session transcript.

## Multi-Phase Delegation (Per-Phase)

For a full pipeline `/kbd-goal "build standup generator" --phases ideation,spec,creation`:

1. **Ideation phase** — KBD runs its discovery + critic loop natively
2. **Spec phase** — KBD runs its writer + reviewer loop natively
3. **Creation phase** — KBD invokes:
   ```bash
   claude /goal --worktree "all tasks in TASKS.md are [x]; tests pass; lint clean"
   ```
   The native `/goal` evaluator (Haiku) grades this condition after each turn.

## Evaluator

On Claude Code, KBD does **not** invoke `kbd-goal-evaluator` for the Creation
phase — the native `/goal` evaluator (Haiku instance) handles it.

`kbd-goal-evaluator` is still used for:
- Checking Ideation convergence (≥3 survivors)
- Checking Specification PASS (before human gate)
- Any phase that doesn't delegate to native `/goal`

## Token Budget

Pass the budget through: `--tokens <goal.json → token_budget>`.

If `--worktree` is used, each phase spawns a clean git checkout. KBD merges
the result back to main after PASS and cleans up the worktree.

## Human Gates

On Claude Code, human gates use `AskUserQuestion` synchronously — no checkpoint file needed.

**Ideation → Spec gate:** After `kbd-goal-evaluator` returns PASS for the ideation phase,
present the candidate list via `AskUserQuestion` with each candidate name as an option plus
"Request revision" and "Stop here". The chosen candidate is recorded in
`goal.json → phases[ideation].human_gate_result` immediately.

**Spec → Creation gate:** After `kbd-goal-evaluator` returns PASS for the spec phase,
present three options via `AskUserQuestion`: "Approve — begin Creation", "Request revision",
"Stop here". Record in `goal.json → phases[spec].human_gate_result`.

**`--auto-gates`:** Both gates are skipped; `provenance: implicit` is recorded automatically.

## Setup Requirements

- Claude Code ≥ v2.1.139
- No additional configuration required — `/goal` is built-in
