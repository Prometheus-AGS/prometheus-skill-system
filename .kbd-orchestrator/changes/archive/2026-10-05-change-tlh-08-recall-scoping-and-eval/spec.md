# change-tlh-08-recall-scoping-and-eval

**Title:** Recall admits untagged pk entries only for the current project or on a lexical match; a fixed evaluation fixture gates recall quality
**Plan PR:** 08
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** `change-tlh-04-scratch-surreal-for-envelope-test`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-08` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G3b

## Why

`pk_allowed` (`learning_recall.py:403-404`) admits every untagged pk entry, so `prior-context.md`'s "pk knowledge" block filled with other projects' legacy ingests (KnowMe, avatars, Actix) while the lesson block was relevant (assessment G3b).

## What Changes

- `pk_allowed`: an untagged entry is admitted when its pk scope is `project`, meaning it came from the current repository's own pk KB root. The off-topic entries observed are `[shared:...]` and `[global:...]` scopes, not `project`. Otherwise it is admitted only when `lexical_similarity(query, title + excerpt)` >= `PK_UNTAGGED_MIN_SIMILARITY`; tagged entries keep today's rules.
- Scope definitions for pk entries: *project* = returned by pk for the scratch repo's own KB (the test runs with cwd = a scratch repo root containing `.prometheus/project.json` and HOME = scratch, so pk resolves only scratch roots); *role* = carries a `role:<current role>` tag. *Foreign* = any shared/global-scope entry whose project tag or source names another project.
- The threshold is set from the evaluation fixture, not guessed: pick the smallest value at which the fixture's foreign-project entries are all excluded, record it as the constant with a comment naming the fixture.
- Add `shared/scripts/tests/test-recall-quality.sh`: a scratch surreal-memory (change 04 library) and a scratch pk KB seeded with project lessons, foreign-project untagged entries and global feedback; six fixed phase-goal queries split into 3 tuning and 3 held-out; the threshold is chosen on the tuning queries only, and the assertions run on the held-out ones; assert for each that >= 3 of the top 5 recalled items carry the current project or role scope and none is a foreign-project entry; print the per-query table.

## Scope

- `shared/scripts/lib/learning_recall.py`
- `shared/scripts/tests/test-recall-quality.sh`
- `shared/scripts/tests/fixtures/recall-quality/**`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- The last task regenerates generated outputs (`node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js`, which is also `build:codex`) before the gate runs `check:distribution`.
- A change with a Depends-on starts from `origin/main` after that dependency has merged; its verify.sh exits 2 (BLOCKED) when the dependency is absent from the base.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.

## Plan reference

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-08-recall-scoping-and-eval` and backend task ID).
