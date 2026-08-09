# Creation Phase Template

The Creation phase decomposes `SPEC.md` into a `TASKS.md` checklist and runs
a per-task implement→verify loop. A `kbd-task-verifier` subagent checks each
task against `SPEC.md` acceptance criteria independently from the implementer.
Tasks that fail 3+ times are auto-promoted to child KBD phases.

## Loop Flow

```
SPEC.md (acceptance criteria table)
    │
    ▼
[1] Decomposition (one-time, phase start)
    - Implementer reads SPEC.md user stories + acceptance criteria
    - Creates TASKS.md: one task per user story or logical unit of work
    - Each task entry: [ ] task-NNN: <description> [AC-IDs covered]
    - Writes initial STATE.md with total count and all tasks as pending
    │
    ▼
[2] Per-task loop (repeat until all tasks [x] or escalation)
    │
    ├─ [2a] Pick next [ ] task from TASKS.md
    │
    ├─ [2b] Implementer executes task
    │       - Writes code + tests
    │       - Runs tests + linter (captures output to .goal-run/<task-id>/)
    │       - On Claude Code: runs in --worktree isolation where possible
    │       - Updates STATE.md: active_task = task-NNN
    │
    ├─ [2c] kbd-task-verifier subagent
    │       - Reads SPEC.md acceptance criteria for this task
    │       - Reads .goal-run/<task-id>/test-output.txt + lint-output.txt
    │       - Returns {verdict: PASS|FAIL, reason}
    │
    ├─ [2d] On PASS:
    │       - Mark task [x] in TASKS.md
    │       - Commit changes (message: "task-NNN: <description>")
    │       - Update STATE.md: completed++, active_task = null
    │       - Advance to next task
    │
    └─ [2e] On FAIL (retry up to max_retries=3):
            - Increment STATE.md tasks[task-NNN].fail_count
            - Feed failure reason back to implementer
            - Retry implementation
            - If fail_count >= 3:
                → AUTO-PROMOTE: run kbd-goal-promote.sh <task-NNN>
                → Mark task [~] in TASKS.md
                → Continue with other tasks
    │
    ▼ (all tasks [x] or [~])
[3] Stopping condition check
    - Invoke kbd-goal-evaluator with stopping condition from goal.json
    - PASS → phase complete → human gate (STATE.md summary)
    - FAIL → investigate: are there unresolved [~] promoted tasks?
              If yes: wait for child phases to complete, re-check
              If no: escalate with STATE.md to human
```

## TASKS.md Format

```markdown
# Tasks — <goal slug>

Generated from SPEC.md on <date>.

## Checklist

- [ ] task-001: Scaffold Go module with `go mod init standup` [AC-00]
- [ ] task-002: Implement git log parser returning structured commits [AC-01]
- [ ] task-003: Implement day-grouping logic with 5-bullet cap [AC-02, AC-03]
- [ ] task-004: Implement CLI flag parsing (--since, --repo, --format) [AC-04]
- [ ] task-005: Implement markdown output formatter [AC-05]
- [ ] task-006: Implement Slack mrkdwn output formatter [AC-06]
- [ ] task-007: Add exit code handling (0/1/2) [AC-07]
- [ ] task-008: Write integration test suite [AC-01 through AC-07]
```

## STATE.md Updates

The implementer updates `STATE.md` after every task attempt:

```markdown
## Tasks

| ID | Status | Fail Count |
|----|--------|-----------|
| task-001 | complete | 0 |
| task-002 | in_progress | 1 |
```

The `fail_count` field is the auto-promotion trigger.

## Output Capture

Test and lint output goes to `.goal-run/<task-id>/`:

```
.goal-run/
└── task-002/
    ├── test-output.txt    # stdout+stderr of test run
    ├── lint-output.txt    # linter output
    └── exit-code.txt      # exit code of test command
```

The implementer creates this directory and captures output before invoking
the verifier. The verifier reads these files — it does NOT re-run tests.

## Worktree Isolation (Claude Code)

On Claude Code, the implementer uses `--worktree` for task isolation:
- Each task runs in a fresh git worktree
- Completed task worktrees are merged back on PASS and cleaned up
- Failed task worktrees are preserved for retry (up to max_retries)

On other platforms: implementer works in the main checkout with commits
per task providing the isolation boundary.

## Auto-Promotion

When `STATE.md tasks[task-NNN].fail_count >= 3`:

```bash
scripts/kbd-goal-promote.sh task-NNN
```

This:
1. Calls `kbd-new-child task-NNN-<slug>` to spawn a child KBD phase
2. Writes `handoff-in.md` with: task description, last 3 failure reasons,
   relevant SPEC.md acceptance criteria
3. Marks parent TASKS.md: `[~] task-NNN: promoted to child: task-NNN-<slug>`
4. Logs to `STATE.md → promotions[]`

The promoted task gets its own assess→analyze→plan→execute→reflect cycle,
giving it the depth it needs without stalling the parent loop.

## Platform-Specific Notes

| Platform | Worktree | Verifier | Loop Driver |
|---|---|---|---|
| Claude Code | `--worktree` flag | `kbd-task-verifier` subagent | native `/goal` per phase (goal-007) |
| Codex | Manual worktree | `kbd-task-verifier` subagent | `codex /goal` + continuation.md (goal-008) |
| OpenCode | Manual worktree | `kbd-task-verifier` subagent | goal plugin session.idle continuation |
| Kimi | No worktree | `kbd-goal-check` skill | `/goal next` queue |
| Zed | No worktree | `kbd-goal-evaluator` subagent | ACP delegation or session/prompt loop |
