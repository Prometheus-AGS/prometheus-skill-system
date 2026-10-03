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

Also refreshes `.kbd-orchestrator/current-waypoint.json` so any AI tool can
resume cleanly.

## OpenSpec lifecycle preflight

At session startup and before starting or entering a phase, resolve the installed
`kbd-process-orchestrator` skill directory as `KBD_ORCHESTRATOR_ROOT` and run:

```bash
node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" refresh --project "<project-root>" --timeout-ms 120000
```

This resolves the latest stable official OpenSpec CLI and refreshes the project's
generated OpenSpec skills/commands while preserving authored specs, changes and
custom configuration. Read the receipt: failed or pending refresh is unresolved,
not current-version proof. A cached offline version is explicitly unverified for
latest freshness. Resolve reported conflicts before dependent phase mutations.
Native KBD projects remain usable without adopting OpenSpec; the helper skips
projects without an existing OpenSpec root.

Use the same preflight in harnesses without startup hooks and before raw
`prometheus kbd` phase commands that bypass the lifecycle scripts. Invoke
OpenSpec through the managed runner, including commands copied from generated
skills, rather than an independently versioned global executable:

```bash
node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" run --project "<project-root>" -- <openspec arguments>
```

The runner preserves CLI stdout for JSON consumers. Planning model assignments
still never launches workers or changes inference providers. A dry-run or
read-only request does not authorize this mutating refresh; report its pending
preflight and defer it until a writable invocation.

## Task model dispatch

Read the phase plan's **Task model assignments** and
[task selection protocol](../kbd-plan/references/task-model-selection.md).
Resolve each entry by full phase path, change ID and backend task ID. Recheck
availability, policy and documented worker controls before dispatch; carry the
assignment reference and actual model/route into `execution.md` and worker
handoffs. For legacy plans, record an explicit task selection using that protocol.
Refresh assignments after material task, harness or capability changes. Unavailable
routes remain unresolved; record any deliberate alternative selection and its
rationale before running it. Other eligible tasks can continue. liter-llm inference
requires a tool-enabled execution mechanism; KBD retains task completion ownership.

## Final phase QA gate

Do not run tests, verification builds, artifact refinement, or adversarial review
after individual tasks or changes. Complete every planned production change first.
When the active harness provides an agent team, assign implementation to executor
roles and keep reviewer, auditor, verifier, and integration-checker roles dormant
until every production change is complete. Then run one production-path integration
gate and one cumulative artifact/adversarial review. The result is evidence and
certification state; it must not reopen the implementation counter:

```
all implementation_status values → COMPLETE in progress.json
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
  │   │   ├─ if OpenSpec: /opsx:verify → /opsx:archive
  │   │   └─ if native: move to .kbd-orchestrator/changes/archive/<date>-<id>/
  │   │   (WARNING findings: logged in the review dir, archive proceeds;
  │   │    SUGGESTION: informational)
  │   │
  │   └─ verdict BLOCK (any CRITICAL) → mark certification BLOCKED in progress.json
  │       └─ fix, then re-run only the failed final gate
  │
  └─ ANY FAIL → mark certification BLOCKED in progress.json
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

When all steps are complete, emit:

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

And after the last change in that sub-phase:

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
5. **Follow the execute protocol**: in a nested orchestrator installation, read
   `../../prompts/execute.md`; in a flat skill installation, read
   `../kbd-process-orchestrator/prompts/execute.md`. Resolve these relative to
   this skill directory, using the matching installed layout.
6. **Write `execution.md`** with selected backend + dispatch contract
7. **Record the active path** with a typed KBD command; projections refresh automatically
8. **Register planned changes and tasks** with `prometheus kbd change|task`
9. **Dispatch** to selected backend or mark phase execution-ready
10. Complete every planned production change without intermediate verification
11. Run one production-path integration gate and one cumulative final review
12. Archive changes after the final phase gates pass

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

Fire `execute:before` before selecting a backend, `execute:after` after
writing `execution.md`. **`task:before`/`task:after` are fired per task by
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
# … select backend, write execution.md …
kbd_hooks_fire execute after  "$phase" 1 1
```

Note: the `on_change_complete` legacy alias is fired automatically by
the dispatcher on the **final** `task:after` of each change (sentinel:
`KBD_HOOK_INDEX == KBD_HOOK_TOTAL`). Projects relying on
`on_change_complete` continue to work without changes.

## Stage gate & handoff

The execute gate requires the plan handoff. After writing `execution.md`
and registering canonical changes/tasks, record the handoff that reflect reads
first:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate execute || exit 2
# … select backend, write execution.md, register canonical work items …
kbd_stage_handoff_write execute "<1–3 sentences: backend chosen, dispatch contract, first pending change>" execution.md progress.json
```

Phases without a `handoffs/` directory are legacy: the gate warns and passes.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.

## Delivery cadence profiles

When a delivery-cadence profile is selected, include its path, state root, iteration scope, required build/run actions and publication interval in the authored execution dispatch contract. The harness remains continuation owner; kbd-apply remains canonical task owner. Do not edit generated waypoints.

Finish the complete independently usable increment, then BUILD and RUN its actual function. Do not run test suites, per-task verification or reviewer loops at iteration boundaries. Fix build, launch or functional failures before beginning another increment. A timer never certifies partial work. Apply the profile's human review and publication policy; keep architecture approvals separate. See the delivery-cadence skill only for cadence-enabled work.
