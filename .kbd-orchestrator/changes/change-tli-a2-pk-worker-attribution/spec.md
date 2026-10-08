# change-tli-a2-pk-worker-attribution

**Title:** Learning worker carries project/team/role, never emits a null agent_id, and commits the prompt snapshot
**Plan PR:** A2
**Repository:** `prometheus-knowledge-rs`
**Phase:** team-aware-learning-memory-impl
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-knowledge-rs at or after `1bbaecc`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-a2` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §6, §8 U2

## Why

`pk-learning-worker/src/main.rs` writes via `MarkdownStore::upsert` but never calls `pk_store::commit_prompt_snapshot` (so session entries are invisible to `pk context`), sends `"agent_id": null` from `enqueue_memory` (lines 727-734) and from `normalize_payload`'s `add_task_step` branch (line 1315, also `user_id: null`). This is the source of the 100% null-agent_id records (assessment A2, round-2 review).

## What Changes

- Add `project_id`, `team_id`, `role_id` (all `#[serde(default)]`) to `LearningJob`.
- `enqueue_memory` derives `user_id` = job `project_id` (falling back to `project_scope()`), and `agent_id` per the design table: `<team>/<role>` when both are set, `@project` otherwise. Never null.
- `normalize_payload` `add_task_step`: fill `user_id` and `agent_id` the same way.
- After a successful `store.upsert` in `process_job`, call `pk_store::commit_prompt_snapshot` for the touched KB.
- Bump the crate/workspace version to 1.10.0 is NOT done here (A5a tags after all A changes merge).

## Scope

- `pk-learning-worker/src/main.rs`
- `pk-learning-worker/tests/attribution.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
