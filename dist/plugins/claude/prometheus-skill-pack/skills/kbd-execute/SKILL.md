---
license: MIT
name: kbd-execute
version: '1.0.0'
description: >
  Select an execution backend for the active KBD phase, write canonical phase
  execution state, dispatch the phase to the appropriate tool or OpenSpec, and
  maintain KBD as the source of truth. Supports multi-tool handoff via
  progress.json protocol. Runs integration and review gates once at phase completion.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-execute

Run the **Execute** phase of the KBD lifecycle.

## What this does

Reads `.kbd-orchestrator/phases/<phase-name>/plan.md`, selects the best
execution backend (tool or OpenSpec), writes `execution.md`, and dispatches the
phase while keeping KBD as the source of truth.

Dispatch begins Execute; it does not complete it. Write the backend contract
in `execution.md` and a dispatch receipt at
`.kbd-orchestrator/phases/<phase>/execute-dispatch.json` with the actual
dispatch time, assigned changes, and pending work. This receipt is not a stage
handoff: do not give it `completedAt` or `nextStage: reflect`, and do not place
it at `handoffs/execute.handoff.json`.

Keep the parent Execute stage active while delegated or local work runs. Use
`kbd-apply` task boundaries and typed KBD mutations; the runtime refreshes
progress and waypoint projections. Resume from actual canonical work state,
not the existence of a dispatch artifact.

## Final phase QA gate

Do not run tests, verification builds, artifact refinement, or adversarial review
after individual tasks or changes. Complete every planned production change first.
When the active harness provides an agent team, assign implementation to executor
roles and keep reviewer, auditor, verifier, and integration-checker roles dormant
until every production change is complete. Then run one production-path integration
gate and one cumulative artifact/adversarial review. The result is evidence and
certification state; it must not reopen the implementation counter:

```
all phase implementation complete via typed KBD transitions (projected in progress.json)
  │
  ├─ /refine-validate "<change-id>"
  │   ├─ reads constraints from .kbd-orchestrator/constraints.md
  │   ├─ validates all produced artifacts
  │   └─ writes .refiner/artifacts/<change-id>/refinement_log.md
  │
  ├─ ALL PASS → /adversarial-review --mode diff "<change-id>"
  │   ├─ cross-model fresh-context judge (cheap checklist first, judgment second)
  │   ├─ writes .kbd-orchestrator/phases/<phase>/review/<change-id>/findings.json
  │   │
  │   ├─ verdict PASS → proceed to archive
  │   │   ├─ if OpenSpec: kbd-apply verify → kbd-apply archive
  │   │   └─ if native: kbd-apply verify → kbd-apply archive
  │   │   (WARNING findings: logged in the review dir, archive proceeds;
  │   │    SUGGESTION: informational)
  │   │
  │   └─ verdict BLOCK (any CRITICAL) → record certification BLOCKED through typed KBD commands
  │       └─ fix, then re-run only the failed final gate
  │
  └─ ANY FAIL → record certification BLOCKED through typed KBD commands
      └─ fix the phase-level finding, then re-run only the failed final gate
```

See `references/integrations/artifact-refiner.md` for the QA invocation
contract and constraint wiring, and
`references/integrations/adversarial-review.md` for the adversarial-review
contract (packet assembly, judge dispatch, fallback chain).

### Local review coverage

File-count and documentation-only skips do not exist. QA and adversarial review
cover the cumulative phase diff after implementation is complete.
`--skip-qa` and `--skip-adversarial-review` may let development continue, but
record `pending_review`; final local certification still requires a completed
receipt or an SSH-signed waiver. The two flags remain independent.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists) to get the current phase, step N of T, and next command. If that file is absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting kbd-execute — <phase-name> (step N of T)
```

Only after the execute completion checklist below is satisfied, emit:

```
Completed kbd-execute — <phase-name> (step N of T)
```

**How to get N and T (MANDATORY — never estimate):**
- Read `.kbd-orchestrator/phases/<phase>/progress.json` →
  `completion.implementation.completed` = N and `.total` = T; fall back to
  legacy `changes_completed` / `changes_total` only when canonical fields are absent.
- If `progress.json` is absent, read `current-waypoint.json` →
  `implementationCompleted` / `implementationTotal`, then legacy aliases.

When executing a named sub-phase within a multi-phase plan, emit the phase-level signal BEFORE the first change signal — even when the orchestrator is not present:

```
Starting phase <N> out of <total>: <sub-phase-name>
```

After all changes in that sub-phase satisfy the execute completion checklist:

```
Completed phase <N> out of <total>: <sub-phase-name>
```

Additionally, emit before and after each individual change (read canonical
`completion.implementation` from `progress.json`; legacy counters are fallback
aliases only — never guess and never use evidence-task completion):

```
Starting change <N> of <total>: <change-id>
Completed change <N> of <total>: <change-id>
```

When the code/integration contract for a change is complete, update the ledger
atomically with:

```bash
scripts/kbd-validate-progress.sh --mark-implementation-complete \
  .kbd-orchestrator/phases/<phase>/progress.json <change-id>
```

This transition does not mark evidence, certification, or publication complete.
Never postpone it merely because those independent dimensions are pending.

Use the canonical phase name from the argument or `current-waypoint.json`. Phase and change totals must come from `progress.json` or the plan — never guessed. Emit to plain response text — no tool call needed.

## How to invoke

1. **Discover project identity** — read `.kbd-orchestrator/project.json` or infer
2. **Confirm the active phase** — from argument or waypoint
3. **Load waypoint** — `.kbd-orchestrator/current-waypoint.json` first when it exists
4. **Load assessment and plan** for the phase
5. **Follow the execute protocol** in `../prompts/execute.md`
6. **Write `execution.md`** with selected backend + dispatch contract
7. **Record the active path** with a typed KBD command; projections refresh automatically
8. **Register planned changes and tasks** with `prometheus kbd change|task`
9. **Dispatch** and write `execute-dispatch.json`; keep Execute active
10. Complete every planned production change without intermediate verification
11. Run one production-path integration gate and one cumulative final review
12. Verify and archive changes through `kbd-apply` after the final phase gates pass
13. **Complete Execute only at the phase boundary** — use the checklist below; dispatch is not completion

## Backend Types

| Backend       | When to use                                                |
| ------------- | ---------------------------------------------------------- |
| `openspec`    | OpenSpec available; spec-backed traceability required      |
| `native-tool` | Tool has explicit planning, inspectable progress           |
| `hybrid`      | Native tool for decomposition, OpenSpec for spec execution |
| `manual`      | Human-only operation; no automation possible               |

## Examples

```
/kbd-execute                             # uses active waypoint phase
/kbd-execute phase-2-sales-module        # explicit phase name
/kbd-execute phase-2-sales-module roo   # dispatch to Roo Code specifically
/kbd-execute --skip-qa                   # skip artifact-refiner QA gate
/kbd-execute --skip-adversarial-review   # skip cross-model adversarial review gate
```

## Hook integration

Fire `execute:before` when entering Execute, before selecting a backend.
Fire `execute:after` only at the completed Execute boundary described below,
never after merely writing `execution.md` or dispatching work. **`task:before`/`task:after` are fired per task by
`/kbd-apply`** — the KBD-owned apply driver — not by `/kbd-execute` and **not**
by bare `/opsx:apply`. `/kbd-execute` writes the dispatch contract; `/kbd-apply`
walks the tasks, firing the per-task hooks and emitting the plain-text position
signal on each boundary. See the `kbd-apply` SKILL for the per-task contract.

> **Corrected (2026-06-03):** earlier versions of this file claimed bare
> `/opsx:apply` fired the per-task KBD hooks. It does **not** — `/opsx:apply` is
> unmodified upstream OpenSpec with no KBD awareness (no hooks, no
> `progress.json`, no waypoint). Driving it directly is the seam that broke
> plan→execute. Always route task execution through `/kbd-apply`.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

kbd_hooks_fire execute before "$phase" 1 1
# … write execution.md and execute-dispatch.json; drive the assigned work …
# No execute:after or execute completion handoff at dispatch.
```

Note: the `on_change_complete` legacy alias is fired automatically by
the dispatcher on the **final** `task:after` of each change (sentinel:
`KBD_HOOK_INDEX == KBD_HOOK_TOTAL`). Projects relying on
`on_change_complete` continue to work without changes.

## Execute completion checklist

Before completing Execute, inspect the active phase’s canonical state and
actual evidence. All of the following must hold:

1. Every change and task assigned to this phase’s execution scope is complete;
   none remains pending, in progress, or blocked. Completion of one delegated
   change does not complete the parent stage.
2. The cumulative phase satisfies the single final integration, QA and independent
   review gates, with real receipts or an explicitly permitted signed waiver. A
   skip flag or `pending_review` is not a passing result. Per-change QA is forbidden.
3. Required `kbd-apply verify` and `kbd-apply archive` operations have
   succeeded for every applicable change, including any reconciliation work
   assigned to this phase. Record actual outcomes and evidence locations.

If anything remains, report it and keep Execute active. Implementation N/N
alone does not satisfy this checklist. Do not infer success from a dispatch
receipt, a missing tool, or a previous completion claim.

Once all conditions hold, record the completed execute stage with a typed
`prometheus kbd stage transition`, fire `execute:after`, inspect its actual
outcome under project hook policy, and then write the completion handoff.
Required hook failures must be resolved before handing off to Reflect.
Preserve actual earlier receipts; never manufacture a successful past hook.

## Stage gate & handoff

At dispatch, require the plan handoff and write only dispatch artifacts:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate execute || exit 2
# … enter Execute with a typed command, write execution.md and execute-dispatch.json …
# Keep Execute active; do not write its completion handoff here.
```

Only after the completion checklist and required hook outcomes above are
satisfied, invoke:

```sh
kbd_stage_handoff_write execute "<completed phase scope; QA/review and verify/archive evidence>" execution.md progress.json
```

This completion handoff is what Reflect reads first. Its `completedAt` and
`nextStage` describe a completed Execute boundary, never dispatch readiness.

A missing `handoffs/` directory does not bypass required predecessors.
A missing required handoff fails with remediation: complete the predecessor
stage, or record an explicit skip with its reason under project policy.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.
