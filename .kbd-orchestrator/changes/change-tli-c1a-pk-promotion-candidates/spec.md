# change-tli-c1a-pk-promotion-candidates

**Title:** pk promotion detector and `pk candidates list|accept|reject`
**Plan PR:** C1a
**Repository:** `prometheus-knowledge-rs`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a5a-skillpack-pin-v1-10-0`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-knowledge-rs at or after `1bbaecc`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-c1a` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §3

## Why

Promotion to user/global is auto-proposed and human-confirmed (approved policy); nothing proposes or applies candidates today.

## What Changes

- `pk-learning-worker/src/promotion.rs` after `run_once`: fingerprint lessons into `~/.prometheus/learning-index/lessons.jsonl`; propose a candidate with evidence when a lesson recurs (Jaccard ≥ 0.6) across ≥ 2 projects, is tagged global without a marker, or names a dependency/CLI with no repo-relative paths; candidates under `~/.prometheus/promotion-candidates/pending/` written atomically.
- `pk candidates list|accept|reject --kind promotion|skill` in pk-cli: accept upserts into the shared KB, commits the snapshot, queues an SM op for `@user:<hash>`/`@global`, moves the file to `accepted/`.

## Scope

- `pk-learning-worker/src/promotion.rs`
- `pk-learning-worker/src/main.rs`
- `pk-cli/src/main.rs`
- `pk-learning-worker/tests/promotion.rs`
- `pk-cli/tests/candidates.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
