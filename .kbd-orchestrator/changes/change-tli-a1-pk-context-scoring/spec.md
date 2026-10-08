# change-tli-a1-pk-context-scoring

**Title:** pk context scores every snapshot entry, caps after scoring, and redistributes a failed scope's budget
**Plan PR:** A1
**Repository:** `prometheus-knowledge-rs`
**Phase:** team-aware-learning-memory-impl
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-knowledge-rs at or after `1bbaecc`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-a1` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §8 U1, §4

## Why

`pk-cli/src/main.rs::run_context` takes `ceil(max_candidates/scopes)` entries per scope in snapshot order **before** `snapshot_score` runs (lines 706-752), so most lessons are never scored, and a failed scope's share is lost (assessment A1, analysis D-5).

## What Changes

- In `run_context`, score every entry of every successfully read scope snapshot; drop the per-scope `take(remaining)`.
- Apply `max_candidates` once, to the merged scored list, after de-duplication by id and content hash.
- Order deterministically: score desc, then scope priority (project, shared, global), then entry id.
- A scope that fails (no root, no snapshot) contributes nothing and its share is not reserved: the cap is global.
- Keep the existing CLI flags and output shape (`--max-candidates`, `--max-bytes`, `--limit`, `--format`).

## Scope

- `pk-cli/src/main.rs`
- `pk-cli/tests/context_scoring.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
