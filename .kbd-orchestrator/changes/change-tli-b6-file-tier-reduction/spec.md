# change-tli-b6-file-tier-reduction

**Title:** Reduce the file-memory tier: MEMORY.md index ≤ 4 KB, per-role local memory, Codex names and memory controls
**Plan PR:** B6
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b5-subagentstart-delivery-alpha`, `change-tli-b3b-agent-team-memory-envelope`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b6` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §5 file tier, §10

## Why

Every Claude subagent loads the ~14 KB auto-memory index and every Codex agent the ~10.8 KB user summary, untargeted (probe behaviour 5; design §5/§10).

## What Changes

- `scripts/memory-index-partition.py`: split an auto-memory `MEMORY.md` into a ≤ 4 KB project index and role-addressed lessons written through learning_write (dry-run by default; `--apply` writes).
- `skills/process/agent-team-creator/runtime/src/export-files.mts`: Claude agent exports may set `memory: local` with a pack-generated per-role MEMORY.md (opt-in); Codex exports use names `role_id.replace('-', '_')` (`^[a-z0-9_]+$`) and set `generate_memories=false` for subagent threads.
- `docs/guide/memory-tiers.md` documents the tiers and the partition tool.

## Scope

- `scripts/memory-index-partition.py`
- `skills/process/agent-team-creator/runtime/src/export-files.mts`
- `skills/process/agent-team-creator/runtime/test-src/export.integration.mts`
- `skills/process/agent-team-creator/scripts/*.mjs`
- `skills/process/agent-team-creator/tests/*.mjs`
- `docs/guide/memory-tiers.md`
- `scripts/tests/test-memory-partition.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
