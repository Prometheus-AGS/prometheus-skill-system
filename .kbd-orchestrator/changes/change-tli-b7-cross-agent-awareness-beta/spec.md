# change-tli-b7-cross-agent-awareness-beta

**Title:** Path-overlap routing, team digest and lead view — beta gate
**Plan PR:** B7
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b5-subagentstart-delivery-alpha`, `change-tli-b6-file-tier-reduction`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b7` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §7, §10

## Why

An agent's lessons must reach the roles that own the paths it touched, and every role should see what changed, without broadcasting full text (design §7).

## What Changes

- `shared/scripts/lib/learning_route.py`: compare lesson `paths` with team `owns` globs; add `role:<r>` audience for matching roles other than the author, at most 3, else `lead`.
- learning_write appends a ≤ 160-char digest line (author, paths, contentHash) to `~/.prometheus/team-digest/<projectId>/<team>.jsonl` (rotate at 1,000 lines) and mirrors it under `<team>/@team`.
- learning_recall includes the last 50 digest lines (counted against budget) and gives the lead/main thread `T/@lead`.
- `shared/scripts/tests/test-team-awareness.sh` = beta gate.

## Scope

- `shared/scripts/lib/learning_route.py`
- `shared/scripts/lib/learning_write.py`
- `shared/scripts/lib/learning_recall.py`
- `shared/scripts/tests/test-team-awareness.sh`
- `scripts/report-learning-delivery.py`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
