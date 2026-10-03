# KBD Process Orchestrator — Execute Phase

You are executing the **Execute** phase of the KBD lifecycle for the **current project**.

> **IMPORTANT**: Do NOT hard-code project names, technology stacks, or tool preferences.
> Derive project identity and constraints from context files.

## Goal

Select the best execution backend for the active phase, write a canonical KBD
execution artifact, dispatch the phase to the appropriate tool(s), and preserve
KBD as the single source of truth for execution state.

This is an orchestration step. You select, delegate, and coordinate — you do
not necessarily execute all tasks yourself.

## Model Selection

Read `plan.md` **Task model assignments** and
[task model selection](../skills/kbd-plan/references/task-model-selection.md).
Resolve each task by full phase path, change ID and backend task ID. Honor its
concrete provider/model, supported reasoning effort and documented worker route;
change-level model classes are summaries, not replacement dispatch decisions.

Recheck actual availability and user/project/budget constraints before dispatch.
Use demonstrated task suitability first, with cost and latency breaking close
ties. For legacy plans without assignments, perform that analysis and record an
explicit selection before running a task. Material task, harness or capability
changes require an updated assignment.

Prefer native execution of the selected model where supported. External models
need both liter-llm inference and an existing documented tool-enabled worker.
Missing prerequisites leave the route unresolved; record an explicit alternative
selection and rationale before using it, never silently substitute. Independent
eligible work may continue. Carry the scoped assignment and actual dispatch
identity into every handoff. See `references/model-routing.md` for policy context.

## Inputs Available to You

- `.kbd-orchestrator/current-waypoint.json` (highest priority re-entry point)
- `.kbd-orchestrator/phases/<phase>/assessment.md`
- `.kbd-orchestrator/phases/<phase>/plan.md`
- `.kbd-orchestrator/phases/<phase>/progress.json`
- `AGENTS.md` and `CLAUDE.md`
- OpenSpec changes (`openspec/changes/`) if available
- `.kbd-orchestrator/constraints.md` if present

## Backend Selection

### Tool Registry (from SKILL.md)

| Backend ID      | Tool                 | Best For                                                    |
| --------------- | -------------------- | ----------------------------------------------------------- |
| `antigravity`   | Antigravity          | Complex multi-file features, planning, browser verification |
| `roo-architect` | Roo Code (Architect) | Architecture decisions, system design                       |
| `roo-code`      | Roo Code (Code)      | Focused bounded implementation                              |
| `cursor-agent`  | Cursor Agent         | Multi-file refactoring, parallel subagent tasks             |
| `claude-code`   | Claude Code CLI      | Large architectural changes                                 |
| `codex`         | OpenAI Codex         | Parallel isolated tasks via git worktrees                   |
| `cline`         | Cline                | Terminal-first agentic workflows                            |
| `kilo-code`     | Kilo Code            | Targeted file edits                                         |
| `windsurf`      | Windsurf Cascade     | Autonomous multi-step sessions                              |
| `opencode`      | OpenCode             | Quick targeted edits and patches                            |
| `openspec`      | OpenSpec (via `/kbd-apply`) | Spec-backed changes with traceability                |
| `speckit`       | GitHub Spec Kit (via `/kbd-apply`) | `specs/<feature>/tasks.md` checklist execution |
| `hybrid`        | Multiple             | Combination: native for decomp, spec backend for QA         |
| `manual`        | Human                | Operations requiring judgment or external tools             |

> Spec backends (`openspec`, `speckit`) are always driven through `/kbd-apply`
> task-by-task — never via a bare `/opsx:apply` or `/speckit.implement`.
> `/kbd-apply detect` selects the backend automatically (openspec directory →
> `openspec`; `.specify/` or `specs/*/tasks.md` → `speckit`).

### Selection Rules

**Use `openspec` when:**

- OpenSpec directory exists at project root
- The phase needs spec-backed traceability
- Native backend would be too opaque for verification

**Use a specific tool backend when:**

- The change is well-bounded and the tool has explicit progress tracking
- You are dispatching a specific agent to a specific change (not the whole phase)
- The task matches the tool's strengths (see registry above)

**Use `hybrid` when:**

- Native tool useful for decomposition; OpenSpec for canonical task execution
- Multiple tools need to cooperate on different changes within the same phase

**Use `manual` when:**

- Human judgment is required (e.g., business decisions, external account setup)
- No AI tool can fully automate the operation

### OpenSpec Fallback Rule

If the selected non-OpenSpec backend:

- Cannot produce inspectable progress
- Cannot keep scope bounded to the phase
- Becomes blocked by missing structure

→ Record an explicit backend change to `openspec` and its rationale. Reconcile
task identities and assignments before dispatch. A backend change does not
authorize substituting a model or bypassing unresolved worker prerequisites.

## Required Output

Write `.kbd-orchestrator/phases/<phase-name>/execution.md`:

```md
EXECUTION: <phase-name>
Project: <project-name>
Date: <ISO date>
Selected backend: <backend-id from registry>
Dispatched to: <specific tool or SELF for Antigravity>
Backend rationale: <why this backend was selected>
Backend entrypoint: <skill command, tool mode, CLI command, or manual process>
OpenSpec available: YES | NO
Source plan: .kbd-orchestrator/phases/<phase-name>/plan.md

EXECUTION SCOPE

- <change-id>: <one-line description>

DISPATCH CONTRACTS
For every task, including self-execution:

- <full phase path> / <change-id> / <backend task ID> → <harness>
  Assignment: <plan.md Task model assignments scoped entry>
  Entry: <documented worker invocation; scope, working directory, tools, skills>
  Concrete model: <provider/model and supported reasoning effort>
  Route: <native or liter-llm plus tool-enabled worker; actual availability>
  Model rationale: <task fit and dated evidence; explicit policy tradeoffs>
  Native alternative: <if different; not an automatic fallback>
  Prerequisites: <unresolved route requirements or none>
  Handoff: Return artifacts and evidence to the KBD driver; do not update task state

APPROVAL GATES

- <gate or NONE>

FALLBACK CONDITIONS

- <condition that triggers fallback to openspec>

VERIFICATION REQUIREMENTS

- <build/test command specific to this project>

PROGRESS LEDGER

- [PENDING|IN_PROGRESS|DONE|BLOCKED] <change-id> — <tool>

OUTPUTS

- <artifact or NONE>

BLOCKERS

- <blocker or NONE>

REFLECTION HANDOFF

- <what kbd-reflect should consume from this phase>

EXECUTION READY
```

Register changes/tasks and refresh canonical execution state through typed KBD
commands. Runtime-owned progress and waypoint files are projections; never edit
their counters directly. Legacy initialization follows the existing KBD adapter.

## Dispatch Protocol

### If dispatching to a non-self tool (Roo, Cursor, Cline, etc.)

Produce a **Tool Handoff Note** embedded in `execution.md` for each scoped task:

```
HANDOFF NOTE for <tool>:
1. Read .kbd-orchestrator/current-waypoint.json
2. Read the change spec: [openspec path | .kbd-orchestrator/changes/<id>/change.md]
3. Read the exact scoped assignment and use its verified model and worker route.
   Report unresolved prerequisites; do not silently substitute another model.
4. Implement only the assigned task and return artifacts, evidence and blockers.
5. The KBD driver alone owns begin-task/end-task and canonical completion updates;
   workers never edit progress.json, task markers, counters or waypoints.
6. Final integration, review and archive remain deferred until every planned
   production change is complete and the required final gates pass.
```

### If backend = `openspec` (or any spec backend) and self-executing

**Route task execution through `/kbd-apply` — never invoke bare `/opsx:apply`.**

Plan/execute boundary (F3): `/kbd-plan` *creates* the change (`/opsx:new`);
`/kbd-execute` *drives* it via `/kbd-apply`. Do not re-create changes here.

1. Confirm the active change from the waypoint (created in `kbd-plan`).
2. Hand off to `/kbd-apply`, which owns the per-task loop:
   - reads the task surface (`kbd-apply list <change>` / `progress <change>`)
   - for each not-done task: `begin-task` (fires `task:before` + plain-text
     "Starting task i of n"), implement that **one** task, `end-task` (marks
     done, syncs `progress.json` + waypoint, fires `task:after` + "Completed
     task i of n")
3. On the final task the `on_change_complete` sentinel fires automatically.
4. After every planned production change is complete, run the production-path
   integration and cumulative review gates. Once they pass, run `kbd-apply verify`
   → `kbd-apply archive` through the managed OpenSpec CLI. Do not run per-task or
   per-change QA gates while production implementation remains unfinished.

> **Why not bare `/opsx:apply`?** It is unmodified upstream OpenSpec: it fires
> no KBD hooks, writes no `progress.json`, and refreshes no waypoint. Invoking
> it directly drops the turn out of KBD entirely — the plan→execute seam this
> design repairs. `/kbd-apply` wraps the same OpenSpec CLI task-by-task instead.

## Questions the Execute Phase Must Answer

1. What backend / tool is selected for each change?
2. Why is it selected?
3. What artifact is canonical for execution progress?
4. What conditions force fallback to OpenSpec?
5. What evidence marks each change complete?
6. What data must be handed to `kbd-reflect`?

## Completion Condition

Execute phase is complete when `execution.md` exists, all changes have backend
assignments and handoff notes, every task has a scoped model selection with route
status, and canonical execution state is registered. Unresolved routes remain
explicit blockers; execution-ready artifacts are not proof of completed work.
