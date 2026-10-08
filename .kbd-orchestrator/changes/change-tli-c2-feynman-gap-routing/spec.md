# change-tli-c2-feynman-gap-routing

**Title:** Knowledge-gap detection on prompts with /learn-goal suggestion and gap resolution
**Plan PR:** C2
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b7-cross-agent-awareness-beta`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-c2` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §4

## Why

When no context covers a problem, the user should be offered the Feynman/learn path (approved plan PR 9).

## What Changes

- `shared/scripts/karpathy-hook-dispatch.sh` prompt branch: one `pk context --format json` call; flag a gap when no result or top score < `PROMETHEUS_GAP_MIN_SCORE`, the prompt is problem-shaped, and < 2 prompt keywords appear in CLAUDE.md/AGENTS.md; emit one `[prometheus-gap] … /learn-goal <topic>` line once per session per topic; append to `~/.prometheus/knowledge-gaps/gaps.jsonl` with the resolved role when inside a subagent; no pk → unchanged behaviour.
- learning_recall writes `## Knowledge gaps` when all sources are empty; kbd-assess/analyze list them.
- learn-goal, feynman-loop, learn-kb SKILL.md closing step ingests the final explanation into pk and marks the gap resolved.

## Scope

- `shared/scripts/karpathy-hook-dispatch.sh`
- `shared/scripts/lib/learning_recall.py`
- `skills/learn/learn-goal/SKILL.md`
- `skills/learn/feynman-loop/SKILL.md`
- `skills/learn/learn-kb/SKILL.md`
- `shared/scripts/tests/test-prompt-gap.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
