# /kbd-execute

Select the execution backend and dispatch the current phase change list.

## Instructions

1. Read `.kbd-orchestrator/project.json` → `activePhase` (fall back to legacy
   `active_phase`), `preferred_execution_agents`, `workspace`, and
   `focus_project_path`.
2. Read `.kbd-orchestrator/phases/<activePhase>/plan.md`.
3. Read `.kbd-orchestrator/phases/<activePhase>/progress.json`.
4. Read the full execute prompt: `.agent/skills/kbd-process-orchestrator/prompts/execute.md`.
5. For each PENDING change in progress.json:

   a. Select the executing agent (from change's `recommended_agent` or project preferences).
   b. Write a HANDOFF NOTE with: focus_project_path, workspace roles, and the change spec.
   c. Initialize progress.json entry with status=IN_PROGRESS, started_by=<this_tool>.
   d. Commit `.kbd-orchestrator/` state to git.
6. For this tool (self-executing):

   a. Implement the change in `focus_project_path`.
   b. Run bdd-testing (`/bdd-testing` for feature file + steps).
   c. Run artifact-refiner QA gate (`/refine-validate <change-id>`).
   d. When code/integration is complete, set
      `implementation_status=COMPLETE` and update
      `completion.implementation` even if evidence tasks remain. Update legacy
      `changes_completed/changes_total` as exact implementation aliases.
      Evidence/certification/publication statuses advance independently.
      Prefer the atomic helper:
      `scripts/kbd-validate-progress.sh --mark-implementation-complete <progress.json> <change-id>`.
   e. Commit: `git add -A && git commit -m "feat: <change-id>"`.
7. After all changes: update waypoint to `kbd-reflect`.

If a phase name is provided as an argument (`$ARGUMENTS`), use it instead of `activePhase`.

## Before Starting

As the very first action, emit the begin-of-step status banner:

```bash
bash "${CLAUDE_PLUGIN_ROOT:-$(git -C "$(pwd)" rev-parse --show-toplevel 2>/dev/null)/.claude-plugin}/shared/scripts/kbd-phase-status.sh" execute begin 2>&1 || true
```

Then proceed with the execute instructions above.

## On Completion

After all changes are dispatched/completed and `current-waypoint.json` is refreshed, emit the end-of-step status banner:

```bash
bash "${CLAUDE_PLUGIN_ROOT:-$(git -C "$(pwd)" rev-parse --show-toplevel 2>/dev/null)/.claude-plugin}/shared/scripts/kbd-phase-status.sh" execute end 2>&1 || true
```

When dispatching to another tool, output:

```
HANDOFF TO: <tool-name>
CHANGE: <change-id>
FOCUS PROJECT (read/write): <focus_project_path>
REFERENCE (READ ONLY): <reference paths from workspace.folders>
CHANGE SPEC: <path to change.md or OpenSpec change>
PROGRESS FILE: .kbd-orchestrator/phases/<phase>/progress.json
WAYPOINT: .kbd-orchestrator/current-waypoint.json
```
