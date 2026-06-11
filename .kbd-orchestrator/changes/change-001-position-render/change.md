---
id: change-001-position-render
title: Waypoint position renderer library
phase: position-and-handoff-guarantee
gaps: [F1, F2]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/lib/waypoint-render.sh
  - shared/scripts/tests/test-position-render.sh
---

# change-001 — Waypoint position renderer library

## Context

No reusable renderer exists that turns `.kbd-orchestrator/current-waypoint.json`
plus the active phase's `progress.json` into a user-facing position block. The
prompt hook (change-002) and stop gate (change-003) both need exactly that, and
it must be one implementation so the injected header and the enforced footer can
never disagree.

## Scope

In:

- New `shared/scripts/lib/waypoint-render.sh` — sourceable library (no side
  effects on source) exposing `waypoint_render`:
  - Walks up from `$PWD` to locate `.kbd-orchestrator/` (same walk-up loop as
    pipeline-enforce.sh); prints nothing and returns 0 when absent.
  - Reads waypoint keys camelCase-first with snake_case fallback
    (`exactNextCommand` → `exact_next_command`, etc.).
  - Renders between sentinels:

    ```
    <!-- prometheus-position -->
    Position: <phase>[ › <childPointer>][ › <change>][ › task i/n] | status: <status>
    Progress: changes <done>/<total>[ | stage: <stage>]
    Last: <currentTask or last_completed>
    Next: <exactNextCommand>
    <!-- /prometheus-position -->
    ```

  - `PROMETHEUS_POSITION_VERBOSITY=dense` (default) | `explain` (adds a `Why:`
    line from waypoint `next_action` when present).
  - Pure bash + jq; 0 network; safe under `set -u`.
- New `shared/scripts/tests/test-position-render.sh` — fixture waypoint +
  progress.json in mktemp dir: asserts sentinel presence, progress fraction,
  Next line, camelCase-over-snake_case precedence, silent exit 0 with no
  `.kbd-orchestrator`, explain-mode Why line.

Out: hook wiring (change-002/003), position.json (change-005).

## Tasks

- [x] 1. Write `shared/scripts/lib/waypoint-render.sh` with `waypoint_render`
- [x] 2. Write `shared/scripts/tests/test-position-render.sh`
- [x] 3. Run the test green; `chmod +x` both files

## Verification

`bash shared/scripts/tests/test-position-render.sh` exits 0.
