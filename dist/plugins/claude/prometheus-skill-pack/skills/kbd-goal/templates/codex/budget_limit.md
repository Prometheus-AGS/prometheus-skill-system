# KBD Goal Budget Limit Prompt (Codex)

<!-- This file is injected by Codex's goal loop when the token budget is nearly
exhausted. It instructs Codex to wrap up gracefully. -->

The token budget for this KBD goal loop is nearly exhausted. Stop implementing
new features. Write a graceful wrap-up.

## What to Do Now

1. Read `.kbd-orchestrator/goals/{{GOAL_SLUG}}/STATE.md`.
2. Append a `## Budget Summary` section to `STATE.md`:
   - List completed tasks (all `[x]` entries in TASKS.md)
   - List in-progress task (current `[/]` entry) and what was accomplished
   - List remaining tasks (all `[ ]` entries)
   - State the stopping condition and whether it was met
3. Commit any uncommitted changes with message: `chore: budget wrap-up for {{GOAL_SLUG}}`
4. Do NOT attempt to complete the current task if it requires significant work.
   Leave it in a clean, resumable state.

## Resume Instructions

To resume this goal in a new session:
```
/kbd-goal --resume {{GOAL_SLUG}}
```

This will read `goal.json` and `STATE.md` to continue from the last checkpoint.
