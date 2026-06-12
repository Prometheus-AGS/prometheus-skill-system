---
id: change-001-waypoint-path
title: Waypoint v3 path[] + kbd_node_dir resolver
phase: child-loops-and-capabilities
gaps: [C1]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/shared/lib/waypoint.sh
  - skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-waypoint-path.sh
---

# change-001 — Waypoint v3 path[] + node resolver

## Context

The waypoint represents position with scalar parentPhase + childPointer, which
caps nesting at 2 levels. An additive `path[]` array makes position a true
chain. Per user decision: additive + lazy — synthesize path[] from v2 fields on
read, keep parentPhase/childPointer as derived for one release.

## Scope

In:

- `waypoint.sh`:
  - `kbd_node_dir <p0> [p1] [p2] ...` — echo the on-disk node dir:
    `.kbd-orchestrator/phases/<p0>/children/<p1>/children/<p2>/...`.
  - `kbd_current_node_dir [waypoint-path]` — read path[] (or synthesize from
    phase/childPointer) and echo the current node dir.
  - `waypoint_load` emits `path=` (comma-joined); synthesize `[phase]` or
    `[phase, childPointer]` when `.path` is absent.
  - `kbd_node_chain <p0> [p1] ...` — render the N-level breadcrumb using the
    existing chain_separator.
- `current-waypoint.template.json`: document `path[]` (schemaVersion bump note;
  additive — every field still has a default; readers must not depend on
  __schemaVersion).
- New `test-waypoint-path.sh`: kbd_node_dir for depth 1/2/3; synthesis from a v2
  waypoint (no path[]); chain rendering; derived-field consistency.

Out: spawning grandchildren (change-002), exit (change-003).

## Tasks

- [x] 1. Add kbd_node_dir/kbd_current_node_dir/kbd_node_chain to waypoint.sh
- [x] 2. waypoint_load emits path[]; synthesize from v2 when absent
- [x] 3. Document path[] in the template; write test; run green

## Verification

Test green; a v2 waypoint (no path[]) synthesizes [phase] or [phase,pointer];
kbd_node_dir builds correct nested paths to depth 3.
