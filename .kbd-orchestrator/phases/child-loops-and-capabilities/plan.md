# Plan — child-loops-and-capabilities

Backend: native-kbd. Ordered: the path[] foundation + resolver first (everything
nests on it), then depth-enabled spawn, then exit/rollup, then the scope hook.
Each change carries `scope:`. Capability creation is OUT (deferred per user).

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-waypoint-path | C1 | Waypoint v3: `kbd_node_dir <path...>` + `kbd_current_node_dir` in waypoint.sh (resolve on-disk dir from a path array); `path[]` emitted by waypoint_load, synthesized from `[phase]` / `[phase, childPointer]` when absent; keep parentPhase/childPointer derived. `kbd_node_chain` renders N-level breadcrumb. Tests: depth-1/2/3 resolution + v2 synthesis. |
| 2 | change-002-depth-child | C1, C2 | kbd-new-child.sh: drop the depth-1 die; resolve parent node from `path[]`; build nested `children/<...>/children/<name>` dir; write `path[]` (push child), `handoff-in.md` (why/inputs/success criteria), `scope.json` (allowedWritePaths/deniedPaths/inheritsConstraints), optional per-child `constraints.md`. kbd-next-child path[]-aware. `maxChildDepth` (default 4) in project.template.json. Tests: grandchild create, depth cap. |
| 3 | change-003-child-exit-rollup | C3 | New `KBD/skills/kbd-child-exit/` (SKILL.md + .sh): require child reflection; write `handoff-out.md` (deliverables/status/recommendations); `KBD/shared/lib/rollup.sh` recomputes a `children{}` aggregate block up the ancestor chain into each parent progress.json; pop `path[]`, restore parent cursor; fire child:after. progress.schema.json gains optional `children`. Tests: exit writes handoff, rolls up counts, pops path. |
| 4 | change-004-child-scope-hook | C4 | `KBD/shared/lib/check-child-scope.sh` (PreToolUse Write\|Edit\|MultiEdit): when waypoint path[] is inside a child, block/notice writes outside the child's scope.json allowedWritePaths (advisory; reuse is_descendant canonicalization). `PROMETHEUS_CHILD_SCOPE_ENFORCE=off\|warn\|ask` default warn. Wire into root hooks.json. Tests: in-scope pass, out-of-scope warn/ask, no-child pass. |
| 5 | change-005-apply-position-depth | C1 | Extend kbd-apply `_phase_dir` and position.sh to walk `path[]` for arbitrary depth (not just 1 child level). Existing depth-1 child tests stay green; add a depth-2 apply+position fixture. |

Completion per change: change.md tasks checked, tests green, commit. Phase end:
`npm run validate:strict`, `npm run build`, `validate:signals`, FULL shell-test
sweep — especially the pre-existing test-kbd-child-phase.sh and
test-kbd-apply-child.sh (must stay green: depth-1 unchanged). Reflection gated.
Carry-forward: this phase does not edit orchestrator SKILL.md, so the overdue
extraction is NOT addressed here — flag it again in reflection.
