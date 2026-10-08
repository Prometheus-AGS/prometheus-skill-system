# change-tli-d2-mini-hook-timeouts-seconds

**Title:** Mini: hook timeouts in seconds, budget test rewritten, one unit in lib/kbd/hooks.mjs
**Plan PR:** D2
**Repository:** `prometheus-skills-mini`
**Phase:** team-aware-learning-memory-impl
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skills-mini at or after `56cbf12`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-d2` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skills-mini worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §5 hook safety (mirror of skill-pack PR #124)

## Why

Mini `hooks/hooks.json` uses 1000/5000/15000 (milliseconds) while Claude reads seconds; `lib/karpathy/hooks-budget.test.mjs:51` assumes ms and would silently stop guarding after conversion; `lib/kbd/hooks.mjs:78` already defaults to 15 seconds (assessment round 2).

## What Changes

- Convert `hooks/hooks.json` timeouts to seconds with a startup floor: 1000 ms → 5 s, 5000 ms → 10 s, 15000 ms → 15 s (a bare 1 s would kill a node hook on startup; ordering of budgets preserved).
- Rewrite `lib/karpathy/hooks-budget.test.mjs` with a seconds threshold (fast-budget hooks ≤ 5 s, the same set as before) so it still guards.
- Make `lib/kbd/hooks.mjs` and `hooks/hooks.test.mjs` assert 1 ≤ timeout ≤ 600 seconds.
- No Codex hooks (mini has none).

## Scope

- `hooks/hooks.json`
- `hooks/hooks.test.mjs`
- `lib/karpathy/hooks-budget.test.mjs`
- `lib/kbd/hooks.mjs`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
