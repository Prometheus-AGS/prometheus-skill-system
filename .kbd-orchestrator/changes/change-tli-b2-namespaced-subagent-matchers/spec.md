# change-tli-b2-namespaced-subagent-matchers

**Title:** Per-role anchored SubagentStop matchers with a Codex team-role guard; executor learning preserved
**Plan PR:** B2
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b1-identity-resolver`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b2` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §1

## Why

Bare matchers `assessor|analyst|planner|executor|reflector` in `shared/harnesses/hook-contract.json` (events 5–9) fire iterative-evolver hooks for any team role with those names (analysis D-4, both review rounds).

## What Changes

- Gate step 0 (precondition, committed as evidence before strings change): capture one real Claude SubagentStop payload for an installed `iterative-evolver` agent in a scratch HOME and record its `agent_type`.
- The five existing groups keep their args and gain `harnesses: ["claude-code"]`; matchers become `^iterative-evolver:<role>$` (or the measured form from step 0).
- Add five Codex-only copies (`harnesses: ["codex"]`, matcher `^<role>$`, new unique hook ids `subagent-<role>-…-codex`); every hook target is wrapped by new `shared/scripts/team-role-guard.sh <real-target> <args…>`, which exits 0 silently when `agent_identity.py --is-team-role <agent_type>` succeeds and otherwise execs the target.
- The executor groups keep their Karpathy learning hook unchanged. `team-role-guard.sh` also supports `--only-team-roles` (inverted guard) for B3's team-role learning group; B2 does not add that group.
- Kimi is untouched (no SubagentStop surface). Regenerate adapters and dist (C-01, C-04).

## Scope

- `shared/harnesses/hook-contract.json`
- `shared/scripts/team-role-guard.sh`
- `scripts/tests/hook-dispatch.test.mjs`
- `scripts/tests/subagent-matchers.test.mjs`
- `.kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/b2-claude-iterative-evolver-payload.json`
- `hooks/hooks.json`
- `hooks/codex-hooks.json`
- `shared/harnesses/generated/*`
- `shared/scripts/generated/hook-dispatch-v1.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
