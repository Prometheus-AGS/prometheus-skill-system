---
license: MIT
name: kbd-new-child
version: '1.0.0'
description: >
  Create a child phase inside the currently-active top-level phase. Mirrors
  /kbd-new-phase but writes into phases/<parent>/children/<child>/, appends
  the new child to childPhases[], moves childPointer to it, and fires
  child:before. Use to split a parent phase into scoped sub-processes.
metadata:
  tags: [process, orchestration, automation, nested-phases]
---

# /kbd-new-child

Create a child phase owned by the currently-active top-level phase.

## What this does

1. Validates the active waypoint has a top-level phase and is not itself inside a child.
2. Validates the child name (kebab-case, no traversal, no slashes) and refuses duplicates.
3. Creates `.kbd-orchestrator/phases/<parent>/children/<child-name>/` with `goals.md` + `progress.json`.
4. Atomically appends `<child-name>` to `childPhases[]`, sets `childPointer` to the new child, updates `currentTask` and `exactNextCommand` to scope to the child.
5. Fires `child:before` exactly once for the new child.
6. Emits Progress Signals and a confirmation banner.

## When to use

When a top-level phase reveals work that's complex enough to deserve its own scope but not big enough to be a sibling phase. Each child carries its own `assessment.md` / `plan.md` / `execution.md` / `reflection.md` under `phases/<parent>/children/<child>/`.

Compare to `/kbd-new-phase`, which creates a top-level sibling.

## Progress Signals (MANDATORY)

```
Starting kbd-new-child — <parent>/<child>
Completed kbd-new-child — <parent>/<child> ready for /kbd-assess
```

## Prerequisites

- A top-level phase MUST be active (the project-level waypoint's `phase` is non-empty and `parentPhase` is null).
- The proposed child name MUST NOT already appear in `childPhases[]`.
- `current-waypoint.json` MUST be valid JSON.

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-new-child/kbd-new-child.sh" <child-name> [goal-1] [goal-2] …
```

The script does the full workflow end-to-end.

## Examples

```
/kbd-new-child auth-refactor "split user-model" "migrate sessions"
/kbd-new-child docs-sweep
/kbd-new-child perf-pass-1
```

## Hook integration

Fires `child:before` exactly once, **after** the waypoint flip, so hooks reading state see the new child as authoritative. `KBD_HOOK_INDEX` is the 1-based position of the new child in `childPhases[]` (always the last); `KBD_HOOK_TOTAL` is the new count.

See orchestrator `SKILL.md` → "Hooks" and `references/schemas/current-waypoint.template.json` for the canonical field set.
