# Assessment — child-loops-and-capabilities

Phase 5 of the approved framework-evolution plan. SCOPE DECISION (user): child
loops ONLY this phase; dynamic capability creation deferred to a later phase.
MIGRATION DECISION (user): additive + lazy — keep parentPhase/childPointer as
derived fields for one release.

## Ground truth (verified by reading the scripts)

| Fact | Location | Implication |
|------|----------|-------------|
| Depth-1 restriction is one explicit die | kbd-new-child.sh:~36 `[[ -z "$this_parent" ]] \|\| die` | removing it + nesting the dir path is the core enabler |
| Child dir = phases/<parent>/children/<name> | kbd-new-child.sh:~58 | grandchild = phases/<p>/children/<c>/children/<g>; build from path[] |
| Waypoint write uses parent/ptr string concat for currentTask/exactNextCommand | kbd-new-child.sh:~92 | path[]-aware version concatenates the full chain |
| waypoint_load emits parentPhase/childPhases/childPointer | waypoint.sh:48-51 | add path[] emission; synthesize when absent |
| waypoint_chain already renders parent › phase › ptr | waypoint.sh:68-85 | reuse for path[] rendering; extend to N levels |
| is_descendant already canonicalizes (cd && pwd -P) | waypoint.sh:100-113 | the canonicalization rule from Phase 3 already lives here — reuse for scope hook |
| kbd-apply _phase_dir is child-aware (1 level) | kbd-apply.sh:176-190 | extend to walk path[] for arbitrary depth |
| position.sh handles 1 child level | position.sh | extend cursor/tree to walk path[] |
| Existing child tests: test-kbd-child-phase.sh, test-kbd-apply-child.sh | shared/lib/tests/ | must stay green (depth-1 still works) |

## Gaps this phase closes

| ID | Gap | From plan |
|----|-----|-----------|
| C1 | Only 2 nesting levels; no path[] canonical position; no kbd_node_dir resolver. | Phase 5.1 |
| C2 | kbd-new-child hard-blocks nesting inside a child; no per-child scope.json or handoff-in.md. | Phase 5.2 |
| C3 | No graceful child exit: no handoff-out, no progress rollup up the ancestor chain, no path pop. | Phase 5.2 |
| C4 | No scoped-permission enforcement for a child loop (an inner loop can edit anything). | Phase 5.2 |

## Constraints

- ADDITIVE: waypoint v3 adds path[] but synthesizes it from v2 on read and
  keeps parentPhase/childPointer as derived (deepest-frame) for one release —
  existing scripts/hooks keep working unchanged.
- Script migration to kbd_node_dir is INCREMENTAL behind tests, not a hard
  cutover (user decision). This phase migrates the child-loop + apply + position
  paths; other scripts keep their existing resolution until a later cleanup.
- check-child-scope.sh is ADVISORY (hook-level, not OS sandbox) — documented as
  such; reuses the Phase 3 canonicalization rule (already in is_descendant).
- All existing depth-1 child tests must stay green.

## Verdict

GO. The depth enabler is small (remove one die, build the dir from path[]). The
risk is the path[] refactor's blast radius — contained by the additive/lazy
decision and by migrating only child/apply/position resolution this phase, each
behind tests, with depth-2 and depth-3 fixtures.
