---
id: change-002-depth-child
title: Arbitrary-depth kbd-new-child + scope.json + handoff-in
phase: child-loops-and-capabilities
gaps: [C1, C2]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-new-child/kbd-new-child.sh
  - skills/process/kbd-process-orchestrator/skills/kbd-next-child/kbd-next-child.sh
  - skills/process/kbd-process-orchestrator/references/schemas/project.template.json
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-grandchild.sh
---

# change-002 — Arbitrary-depth child spawn

## Context

kbd-new-child hard-blocks nesting inside a child (the depth-1 die). With
path[] (change-001), a child can itself have children. Spawn must also write the
child's context-isolation contract (scope.json) and parent→child handoff.

## Scope

In:

- `kbd-new-child.sh`:
  - Remove the `[[ -z "$this_parent" ]] || die` depth-1 restriction.
  - Resolve the parent node from path[] (kbd_current_node_dir); build the child
    dir under it (`.../children/<name>`).
  - Push `<name>` onto path[] in the waypoint; update currentTask/exactNextCommand
    using the full chain (kbd_node_chain).
  - Write into the child dir: `handoff-in.md` (why spawned, input paths, success
    criteria, expected deliverables), `scope.json`
    ({allowedWritePaths, deniedPaths, inheritsConstraints:true}), optional
    `constraints.md` placeholder.
  - Enforce `maxChildDepth` (project.json, default 4) as a sanity rail.
- `kbd-next-child.sh`: resolve current parent from path[] (works at any depth).
- `project.template.json`: add `maxChildDepth` (default 4).
- New `test-kbd-grandchild.sh`: create child then grandchild; assert nested dir,
  path[] has 3 entries, scope.json + handoff-in.md written, depth cap blocks.

Out: exit/rollup (change-003).

## Tasks

- [ ] 1. Drop depth-1 die; path[]-aware parent resolution + nested dir
- [ ] 2. Write scope.json + handoff-in.md + maxChildDepth rail
- [ ] 3. kbd-next-child path[]-aware; project.template.json
- [ ] 4. Write test; run green (existing depth-1 child tests stay green)

## Verification

Test green; a grandchild nests correctly with path[] length 3; existing
test-kbd-child-phase.sh still green.
