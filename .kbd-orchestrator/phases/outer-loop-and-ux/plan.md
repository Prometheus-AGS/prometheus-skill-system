# Plan — outer-loop-and-ux (FINAL phase)

Backend: native-kbd. Order (user decision): carry-forwards FIRST, then features.
HOLD scope-guard in warn mode. Each change carries `scope:`.

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-path-scope-helper | U1 | New `shared/scripts/lib/path-scope.sh`: `pscope_relativize <root> <file>` (canonicalize cd&&pwd-P + repo-relativize, macOS-safe) and `pscope_match <rel> <globs-json>` (python fnmatch). Refactor scope-guard.sh, check-child-scope.sh, protect-tests.sh onto it — behavior unchanged. All 3 existing tests stay green. New test-path-scope.sh. |
| 2 | change-002-skill-extract | U2 | Extract the "Hooks" section (~100 lines) from orchestrator SKILL.md to `references/hooks.md`, leaving a short pointer; gets SKILL.md under 500 lines. Keep Progress Signals + lifecycle in SKILL.md. validate:strict warning clears. |
| 3 | change-003-nesting-docs | U3 | Document enter/descend + the selected-vs-entered invariant in SKILL.md "Nested phases" + a note in current-waypoint.template.json. Add `/kbd-child-exit --enter` to Quick Start. (Doc-only; no code.) |
| 4 | change-004-outer-loop | U4 | New `skills/process/pmpo-outer-loop/`: SKILL.md + loop-definition.schema.json; /loop-define writes `.kbd-orchestrator/loops/<name>/loop.json` (goal+criteria, feedback_sources, termination incl max_no_progress_ticks, escalation_points, cadence); /loop-tick runs ONE evolver cycle with feedback diff + escalate-via-elicit; /loop-report renders journal.md. Wraps evolver; no daemon. Declares Progress Signals. |
| 5 | change-005-dual-ux | U5 | `references/templates/decision-log.template.md` (TL;DR/Why/Alternatives/Learn-more); kbd-status SKILL.md gains `--explain` (expand decision-log + "what's next and why"); `ux_profile: beginner\|advanced` in project.template.json (sets default verbosity, never gates info). |

Out: scope-guard ask-flip (HELD per user — documented as future step in
change-003 or reflection). Dynamic capabilities (separate future phase).

Completion per change: change.md tasks checked, tests green, commit. Phase end:
validate:strict (SKILL.md warning MUST be gone), validate:signals, build, FULL
shell sweep + new path-scope test. Reflection gated. This CLOSES the 6-phase plan.
