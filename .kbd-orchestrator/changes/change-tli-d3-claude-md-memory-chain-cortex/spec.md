# change-tli-d3-claude-md-memory-chain-cortex

**Title:** CLAUDE.md memory chain describes per-role delivery; optional Cortex mirror
**Plan PR:** D3
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b6-file-tier-reduction`, `change-tli-b7-cross-agent-awareness-beta`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-d3` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §5 file tier, §6 Cortex mirror

## Why

CLAUDE.md tells every agent to read the full memory index; the design delivers per role. Cortex is an optional third tier (analysis W4).

## What Changes

- CLAUDE.md and AGENTS.md memory sections: lessons arrive via SubagentStart for each role; the index is project-scope only; write through learning_write.
- learning_write mirrors to Cortex (`cortex_remember` CLI/MCP bridge when present) with project id, `team/role` tag and global flag; absent → silent exit 0.
- `docs/guide/memory-tiers.md` gains the Cortex section.

## Scope

- `CLAUDE.md`
- `AGENTS.md`
- `shared/scripts/lib/learning_write.py`
- `docs/guide/memory-tiers.md`
- `shared/scripts/tests/test-cortex-mirror.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
