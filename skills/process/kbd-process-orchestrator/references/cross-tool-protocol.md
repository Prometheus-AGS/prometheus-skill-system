# Cross-Tool Reporting Protocol

> Extracted from the orchestrator SKILL.md. The contract any dispatched AI tool (Roo, Cursor, Cline, Codex, etc.) follows when executing a KBD change.

## Completion dimensions

All tools MUST treat `completion.implementation` as the KBD N/N counter.
`changes_completed/changes_total` are exact legacy aliases of that dimension.
Evidence, certification, authorization, external/time-bound gates, and
publication are recorded independently and never reopen implemented code.

Before handing work to another tool, run
`scripts/kbd-validate-progress.sh <progress.json>`. A receiving tool must not
infer a code gap from unchecked evidence tasks when
`implementation_status: COMPLETE`.

When an AI tool (Roo, Cursor, Cline, Codex, etc.) is dispatched to execute a
KBD change, it MUST follow this protocol:

### On Start of a Change

1. Read `.kbd-orchestrator/current-waypoint.json`
2. Read the change spec (OpenSpec or `.kbd-orchestrator/changes/<id>/change.md`)
3. Update `progress.json`: set status → `IN_PROGRESS`, `started_by` → `<tool-name>`
4. Update waypoint: `last_updated_by` → `<tool-name>`

### During Execution (on each task completion)

1. Update `progress.json`: increment `tasks_done`, update `last_task_completed` and `next_task_pending`
2. Commit the progress file to git: `git add .kbd-orchestrator && git commit -m "kbd: progress update [<tool>] <change-id> task N/M"`

### On Change Completion

1. Atomically mark implementation complete with
   `scripts/kbd-validate-progress.sh --mark-implementation-complete <progress.json> <change-id>`.
   Set legacy `status → DONE` only when the overall change lifecycle is also
   complete; pending evidence remains in its independent dimension.
2. If OpenSpec: run `/opsx:verify` then `/opsx:archive`
3. If native KBD: move change to `.kbd-orchestrator/changes/archive/<date>-<id>/`
4. Update waypoint: advance `last_completed_change` and `next_pending_change`
5. Commit all state: `git add .kbd-orchestrator && git commit -m "kbd: change complete [<tool>] <change-id>"`
6. **Echo the KBD hook**: `echo '[kbd] Change complete — run /kbd-assess or /kbd-reflect as appropriate'`

### On Blocker

1. Update `progress.json`: set status → `BLOCKED`, add to `blockers` array
2. Update waypoint: set `fallback_command` to describe the blocker
3. Commit: `git add .kbd-orchestrator && git commit -m "kbd: blocked [<tool>] <change-id>"`
