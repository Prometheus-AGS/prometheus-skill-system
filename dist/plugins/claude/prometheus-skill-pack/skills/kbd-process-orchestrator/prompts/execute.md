# KBD Process Orchestrator — Execute Phase

You are executing the **Execute** phase of the KBD lifecycle for the **current project**.

> **IMPORTANT**: Do NOT hard-code project names, technology stacks, or tool preferences.
> Derive project identity and constraints from context files.

## Goal

Select the best execution backend for the active phase, write a canonical KBD
execution artifact, dispatch the phase to the appropriate tool(s), and preserve
KBD as the single source of truth for execution state.

You select, delegate, and coordinate the work through completion; you do not
necessarily execute every task yourself. Dispatch is the beginning of Execute.
Keep the parent Execute stage active until every assigned change/task and its
required QA, review, verification, and archival work has completed.

## Model Selection

**This phase is model-tiered.** Each change routes independently based on its `Model class` annotation from `plan.md`. If a change lacks the annotation, score it using the rules in `references/model-routing.md` before dispatching.

For each change, resolve the concrete model:

1. Read `Model class` from `plan.md` (`small | medium | frontier`).
2. If absent, score by complexity rules in `references/model-routing.md`:
   - **Low** (small): task count ≤ 3, no new abstractions, single layer touched
   - **Medium** (medium): task count 4–8, one module boundary crossed, no design markers
   - **High** (frontier): task count > 8, cross-domain, new abstractions, `TODO:` / `DECISION:` markers
3. Resolve the concrete model: `project.json → model_policy.registry.<class>.<active_environment>`.
4. Annotate the dispatch contract (see Required Output) with `Model class`, `Concrete model`, and `Model rationale`.

The execute phase prompt itself can run on a small model — it is mechanical orchestration. The cost reduction comes from routing each *dispatched* change to the cheapest viable model.

See `references/model-routing.md` for the full routing contract.

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
> `/kbd-apply detect` selects the backend automatically (openspec dir+CLI →
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

→ Fall back to `openspec` and document why.

## Required Output

At dispatch, write `.kbd-orchestrator/phases/<phase-name>/execution.md`:
This is a dispatch contract, not evidence that Execute has completed.

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
For each change assigned to a non-self tool:

- <change-id> → <tool>
  Entry: <exact prompt or command to give the tool>
  Model class: <small | medium | frontier>
  Concrete model: <resolved from model_policy.registry.<class>.<active_environment>>
  Model rationale: <one line — why this class for this change>
  Progress view: .kbd-orchestrator/phases/<phase>/progress.json (generated)
  Handoff: Report through kbd-apply task boundaries and typed KBD transitions

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

PLANNED REFLECTION INPUTS

- <what kbd-reflect should consume after execution completes>

DISPATCH READY — EXECUTE REMAINS ACTIVE
```

Write a separate phase-root `execute-dispatch.json` receipt containing the
actual dispatch time, assigned changes, and pending work. It must not carry
`completedAt` or `nextStage: reflect` and must not occupy
`handoffs/execute.handoff.json`. Do not emit `execute:after` at dispatch.

Register missing changes/tasks through typed `prometheus kbd change register`
and `task register` commands and record the active stage through `stage enter`
/ `stage transition`. Let the runtime create or refresh missing progress and
waypoint projections; never initialize them by copying a compatibility schema.

## Dispatch Protocol

### If dispatching to a non-self tool (Roo, Cursor, Cline, etc.)

Produce a **Tool Handoff Note** embedded in `execution.md` under each change:

```
HANDOFF NOTE for <tool>:
1. Read .kbd-orchestrator/current-waypoint.json
2. Read the change spec: [openspec path | .kbd-orchestrator/changes/<id>/change.md]
3. Before each task: run kbd-apply begin-task with its real ID and plan totals.
4. After implementing that task: run kbd-apply end-task; the driver records
   canonical transitions and regenerates progress/waypoint projections.
5. Record implementation completion through a typed KBD change transition.
   Evidence/certification/publication remain independent; record their actual
   outcomes through typed commands. Use kbd-apply verify then archive after
   the applicable gates pass, including for OpenSpec changes.
6. On blocker: use the typed KBD blocker command and retain the pending task.
   Review and commit only the intended artifacts/state under project policy.
   Never increment counters or edit generated progress/waypoint fields.
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
4. Run the artifact-refiner QA gate and independent adversarial review, then
   `kbd-apply verify` → `kbd-apply archive` (which call `openspec validate` /
   `openspec archive`). Keep Execute active while other phase work remains.

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

The dispatch contract and registered work only establish readiness. Execute
is complete only after all changes/tasks assigned to this phase’s execution
scope are complete, required QA and independent review are satisfied by real
receipts or an explicitly permitted signed waiver, and required driver
verification/archive operations have succeeded for every applicable change.
Include phase-owned reconciliation work; one completed change cannot complete
the parent stage. `pending_review`, blockers, or missing evidence keep Execute
active even if implementation N/N is complete.

At that boundary, use a typed `prometheus kbd stage transition` to record
Execute completion, fire `execute:after`, inspect its actual outcome under
project hook policy, and write `handoffs/execute.handoff.json` only once the
required outcomes are satisfied. Follow the completion checklist and handoff
contract in `skills/kbd-execute/SKILL.md`. That handoff names completed scope
and real QA/review/verification/archive evidence for Reflect. Never infer past
hook success or write a completion handoff merely because dispatch is ready.
