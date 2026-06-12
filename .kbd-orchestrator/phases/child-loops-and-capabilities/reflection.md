# Reflection — child-loops-and-capabilities

Gate: sycophancy-correction analyze_reflect_phase — score 0.018 (PASS); S-08 not detected; one Low S-07 (length) note.

## Delta

1. The phase delivered child loops only, NOT dynamic capability creation (deferred by the user's scoping decision). The phase NAME is now a misnomer for what shipped.
2. Arbitrary-depth nesting surfaced an unplanned design hole: there was no "enter/descend into a child" verb. kbd-new-child after a prior kbd-new-child must add a SIBLING, not nest, so descent became an explicit op. I folded an --enter mode into kbd-child-exit to close it — unplanned scope expansion, and "enter" inside a skill named "child-exit" is an awkward home.
3. The sibling-vs-descend disambiguation rests on a subtle invariant (path[] tail == childPointer → selected-but-not-entered → sibling; pointer cleared → entered → nest). Correct and tested, but implicit knowledge in two scripts, undocumented outside comments.
4. check-child-scope.sh shipped with the SAME class of bug as Phase 3/4: a glob-relativization step turned an allowed path into "**" matching everything, silently passing out-of-scope writes. Caught by the test. THIRD path/glob-matching bug across phases.
5. The child-scope hook is advisory (warn default) + snapshot-bound — the isolation scope.json promises is not active until reload and even then only warns.
6. Orchestrator SKILL.md STILL 620 lines — now FOUR phases overdue; no change touched it. A pattern of deferral.
7. The depth machinery (path[], enter/exit, rollup, scope) is fixture-tested but NEVER driven end-to-end by a real outer agent spawning a real inner loop — the capability phase that would exercise it was deferred.

## Root Cause

1. User chose to split capabilities to isolate the risky path[] refactor (sound). The misnomer is a planning artifact (named for both workstreams before the split).
2. The plan modeled create+exit but not "create" vs "go work inside." The 2-level design hid this (only one level to be "in"); depth made it visible.
3. The selected-vs-entered distinction is genuinely subtle with no pre-existing model; invented under implementation pressure and encoded where needed, not designed/documented first.
4. Path/glob matching has bitten this work three times (symlinks, heredoc-in-subshell, glob over-broadening). ROOT: I keep writing matchers fresh per hook instead of extracting one verified shared helper.
5. Advisory + snapshot-bound is the standing hook constraint; warn-first is the deliberate rollout.
6. The extraction needs a change that touches SKILL.md; none of these implementation phases did. Will keep deferring until scheduled as its own change.
7. The capability loop (real exerciser) was the deferred workstream; only fixtures can validate the machinery this phase.

## Corrective Actions

1. The deferred capabilities work is its own future phase (capabilities-and-dynamic-skills) with a name matching its content.
2. Promote enter/descend to first-class: a dedicated /kbd-enter-child skill, or at minimum prominent docs in the orchestrator SKILL.md child-loop section.
3. Document the selected-vs-entered invariant in orchestrator SKILL.md "Nested phases" + current-waypoint.template.json so external tools follow it.
4. Extract ONE shared path-scope-match helper (canonicalize + repo-relativize + fnmatch) used by scope-guard, check-child-scope, protect-tests — replacing three separately-buggy implementations. Highest-leverage fix to stop the recurring bug.
5. Hold child-scope warn-mode until it + the scope guard have telemetry; flip together.
6. Schedule the orchestrator SKILL.md extraction as an EXPLICIT standalone change next phase (four phases overdue).
7. Treat the deferred capability phase's first kbd-capability run as the integration test for everything built here; budget for fixing what it surfaces.

## Recommended Next Phase

outer-loop-and-ux — the final approved phase: pmpo-outer-loop (Boris Cherny standing-loop runner: /loop-define, /loop-tick, /loop-report over goal + feedback sources + termination), dual-audience UX (decision-log.md, kbd-status --explain, ux_profile), scope-guard flip to ask. ALSO discharge homeless cross-phase carry-forwards: shared path-scope-match helper (CA-4), orchestrator SKILL.md extraction (CA-6, four phases overdue), enter/descend verb + invariant docs (CA-2, CA-3). Dynamic capabilities remains a separate future phase (capabilities-and-dynamic-skills), beyond the original 6-phase plan.
