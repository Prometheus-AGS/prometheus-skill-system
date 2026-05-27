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

After each change reaches `DONE` in `progress.json`, invoke artifact-refiner
as a quality gate before archiving:

```
change status → DONE in progress.json
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
  └─ ANY FAIL → mark change BLOCKED in progress.json
      └─ /refine-code "<change-id>" for iterative refinement
```

See `references/integrations/artifact-refiner.md` for the full invocation
contract and constraint wiring.

### When to skip QA

- Change has fewer than 3 files modified
- Change is documentation-only
- User passes `--skip-qa` flag

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting kbd-execute — <phase-name or argument>
```

When all steps are complete, emit:

```
Completed kbd-execute — <phase-name or argument>
```

When executing a named sub-phase within a multi-phase plan, emit the phase-level signal BEFORE the first change signal — even when the orchestrator is not present:

```
Starting phase <N> out of <total>: <sub-phase-name>
```

And after the last change in that sub-phase:

```
Completed phase <N> out of <total>: <sub-phase-name>
```

Additionally, emit before and after each individual change (read `changes_total` and `changes_completed` from `progress.json` for accurate counts — never guess):

```
Starting change <N> of <total>: <change-id>
Completed change <N> of <total>: <change-id>
```

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
writing `execution.md`. **`task:before`/`task:after` are fired per
OpenSpec task by `/opsx:apply`**, not by `/kbd-execute` itself —
`/kbd-execute` writes the dispatch contract, the apply skill walks
tasks. See orchestrator `SKILL.md` → "Hooks" for per-task wiring.

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
