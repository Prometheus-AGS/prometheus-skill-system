# Assessment — outer-loop-and-ux

Phase 6 (FINAL) of the approved framework-evolution plan. User decisions:
carry-forwards FIRST then features; HOLD scope-guard in warn mode (do not flip
to ask — hooks still unverified live).

## Ground truth (verified)

| Fact | Location | Implication |
|------|----------|-------------|
| 3 path-matchers share identical _canon + REL-relativize + fnmatch | scope-guard.sh:60-89, check-child-scope.sh:72-96, protect-tests.sh:48-66 | extract ONE shared/scripts/lib/path-scope.sh; refactor all three onto it |
| The recurring bug class (symlinks, glob over-broadening) lived in these copies | reflections phases 3/4/5 | one verified helper kills the bug class |
| SKILL.md is 619 lines; "Hooks" (393-491) + "Multi-Tool Coordination" (120-188) are large extractable blocks | SKILL.md | extract Hooks → references/hooks.md to clear the 500-line warn |
| enter/descend is buried in kbd-child-exit --enter; "Nested phases" section exists (356) | SKILL.md:356 | document enter/descend + selected-vs-entered invariant there + in waypoint template |
| evolver has /evolve + max_iterations guard | iterative-evolver SKILL.md | outer-loop wraps it: one /evolve cycle per tick |
| decision-log.md referenced by kbd-analyze but no template exists | research-pipeline.md, kbd-analyze | create the template |
| no ux_profile, no kbd-status --explain | project.template.json, kbd-status SKILL.md | add both |

## Gaps this phase closes

| ID | Gap | Source |
|----|-----|--------|
| U1 | 3 separately-buggy path-matchers (recurring bug class). | CF CA-4 (phase 5) |
| U2 | Orchestrator SKILL.md 619 lines (>500). | CF (4 phases overdue) |
| U3 | enter/descend verb + selected-vs-entered invariant undocumented. | CF CA-2/CA-3 (phase 5) |
| U4 | No standing outer-loop runner (Boris Cherny "write loops not prompts"). | Plan 6.1 |
| U5 | No dual-audience UX (decision-log, kbd-status --explain, ux_profile). | Plan 6.2 |

## Constraints

- HOLD warn mode: do NOT flip scope-guard to ask (user decision — hooks
  unverified live). Document the flip as a future step instead.
- The path-scope refactor must keep all 3 hooks' tests green (immutable-tests
  spirit): refactor onto the helper without changing observable behavior.
- SKILL.md extraction must keep validate:strict + validate:signals green (the
  Progress Signals section stays in SKILL.md, not extracted).
- outer-loop wraps the EXISTING evolver (no new loop engine); cadence delegated
  to platform primitives (manual/background/cron), no daemon.

## Verdict

GO. Debt paydown is mechanical (extract + refactor onto a tested helper,
extract a doc section). The outer-loop runner composes the evolver; UX is
additive artifacts + a status flag. This phase completes the planned 6-phase
scope; dynamic capabilities remain an explicit separate future phase.
