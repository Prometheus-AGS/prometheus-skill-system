# change-tlh-01-ranked-memory-partition

**Title:** memory-index-partition.py ranks entries (current phase, feedback/GLOBAL, project newest-first, archives) before moving any to the learning store
**Plan PR:** 01
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-01` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G1a

## Why

`scripts/memory-index-partition.py` preserves file order (line 11) and moves the last lines first, so the live partition needed a manual priority reorder (assessment G1a). The live index is already back at 4,088 B of its 4 KB budget.

## What Changes

- Add a rank key per index bullet: 0 = a `project` entry whose slug or title matches the active phase (read `.kbd-orchestrator/current-waypoint.json` `phase`, else `position-reminder.txt` `Position:`; no phase = rank 0 empty); 1 = `feedback` entries and any entry whose title contains `GLOBAL`; 2 = other `project` entries, newer first by the `YYYYMMDD` or `YYYY-MM-DD` in the file name, undated last; 3 = `archive-*` entries. The trailing "moved to the learning store" footer line is not ranked.
- Order all bullets by priority: rank 0, then rank 1 (file order), then rank 2 newest first (undated last, file order among equals), then rank 3 (file order). Move bullets to the learning store from the END of that order (archives first, then the oldest rank-2 entries, then rank 1, never rank 0 unless it alone exceeds the budget) until the index fits `--limit`. Kept bullets are written in that priority order. The footer line is never moved and is always written last.
- `--dry-run` prints the rank of every line and which would move; output stays deterministic for identical input.
- Header docstring describes the ranking; the old "file order is preserved" rule is removed.

## Scope

- `scripts/memory-index-partition.py`
- `scripts/tests/test-memory-partition.sh`
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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-01-ranked-memory-partition` and backend task ID).
