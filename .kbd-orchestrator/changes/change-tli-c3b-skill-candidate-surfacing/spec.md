# change-tli-c3b-skill-candidate-surfacing

**Title:** Surface skill candidates in kbd-open and reflect
**Plan PR:** C3b
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-e1-pin-v1-11-0`, `change-tli-c1b-promotion-routing`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-c3b` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §6

## Why

Skill candidates from C3a must reach the operator at session start and at reflect.

## What Changes

- kbd-open lists skill candidates (silent when none).
- Reflect template `## Codify as Skill?` prompt (excluded from write-back); kbd-reflect reviews skill candidates.
- Fix the `pmpo-skill-creator.md` integration doc and the false 'Called by evaluate-session.sh' header in `propose-skill-update.sh`.

## Scope

- `shared/scripts/kbd-open.sh`
- `skills/process/kbd-process-orchestrator/prompts/reflect.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md`
- `skills/process/kbd-process-orchestrator/references/integrations/pmpo-skill-creator.md`
- `shared/scripts/propose-skill-update.sh`
- `shared/scripts/tests/test-skill-candidates.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
