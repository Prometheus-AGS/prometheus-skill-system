---
license: MIT
name: kbd-execute
version: '1.0.0'
description: >
  Select an execution backend for the active KBD phase, write canonical phase
  execution state, dispatch the phase to the appropriate tool or OpenSpec, and
  maintain KBD as the source of truth. Supports multi-tool handoff via
  progress.json protocol. Integrates artifact-refiner QA per completed change.
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

## Per-Change QA Gate (artifact-refiner)

After each change reaches `implementation_status: COMPLETE` in `progress.json`,
invoke artifact-refiner as a quality gate before archiving. The QA result is
evidence/certification state; it must not reopen the implementation counter:

```
implementation_status → COMPLETE in progress.json
  │
  ├─ /refine-validate "<change-id>"
  │   ├─ reads constraints from .kbd-orchestrator/constraints.md
  │   ├─ validates all produced artifacts
  │   └─ writes .refiner/artifacts/<change-id>/refinement_log.md
  │
  ├─ ALL PASS → proceed to archive
  │   ├─ if OpenSpec: /opsx:verify → /opsx:archive
  │   └─ if native: move to .kbd-orchestrator/changes/archive/<date>-<id>/
  │
  └─ ANY FAIL → mark certification BLOCKED in progress.json
      └─ /refine-code "<change-id>" for iterative refinement
```

See `references/integrations/artifact-refiner.md` for the full invocation
contract and constraint wiring.

### When to skip QA

- Change has fewer than 3 files modified
- Change is documentation-only
- User passes `--skip-qa` flag

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
5. **Follow the execute protocol** in `../prompts/execute.md`
6. **Write `execution.md`** with selected backend + dispatch contract
7. **Refresh waypoint** files
8. **Initialize `progress.json`** for the phase if it doesn't exist
9. **Dispatch** to selected backend or mark phase execution-ready
10. **Per completed change**: run artifact-refiner QA gate (see above)
11. **Archive** changes that pass QA

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
and initializing `progress.json`, record the handoff that reflect reads
first:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate execute || exit 2
# … select backend, write execution.md, init progress.json …
kbd_stage_handoff_write execute "<1–3 sentences: backend chosen, dispatch contract, first pending change>" execution.md progress.json
```

Phases without a `handoffs/` directory are legacy: the gate warns and passes.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.
