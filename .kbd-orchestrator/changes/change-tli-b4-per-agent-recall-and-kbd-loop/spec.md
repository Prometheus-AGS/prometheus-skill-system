# change-tli-b4-per-agent-recall-and-kbd-loop

**Title:** learning_recall and the KBD memory loop: lessons written at reflect come back at the next stage
**Plan PR:** B4
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b3-envelope-write-library`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b4` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §4, §6

## Why

kbd-memory-recall reads lifecycle metadata, not lessons; reflect write-back extracts sections the template never emits; the installed `reflect:after` hook path `${KBD_ORCHESTRATOR_ROOT}/../../../shared/scripts/memory-writeback.sh` resolves outside the install so write-back never runs (assessment B/E).

## What Changes

- `shared/scripts/lib/learning_recall.py`: for `(P,T,R)` run the design §4 equality-filtered queries in priority order (`T/R`, `T/@lead` if lead, `T/@team` last 50, `@project`, `@user:<hash>` top 3, `@global` top 3) via REST `/api/v1/search` with `categories` where useful; score semantic 0.5 + recency 0.3 (30-day half-life) + importance 0.2; dedupe on `h:`; stop at the budget; fallback pk `context --tag` then file tier; append `{agentType, bytesByChannel, entriesByScope}` to `~/.prometheus/learning-index/delivery.jsonl`; also read legacy `agent-team-memory` records.
- Rewrite `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh` on learning_recall (lead view for main-thread stages) writing `prior-context.md` sections: lessons, pk knowledge, previous reflection, knowledge gaps.
- Reflect template (`skills/process/kbd-process-orchestrator/prompts/reflect.md`) leads with Delta / Root Cause / Corrective Actions, keeps Lessons Learned with optional `[GLOBAL]`/`[USER]` prefixes, renames to Next Phase Seed, adds Codify as Skill?; `shared/scripts/memory-writeback.sh` extracts those (legacy Next Phase Focus fallback) and writes through learning_write with per-phase hash dedupe.
- `shared/scripts/kbd-stage-writeback.sh` as builtin `assess|analyze|plan:after` hooks writing the stage handoff summary at visibility `lead`.
- Fix the orchestrator `reflect:after` hook to resolve memory-writeback.sh through `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}` with the repo-relative path as fallback.
- kbd-* stage SKILL.md files: step 1 reads `prior-context.md` and cites applicable lessons; reflect states which recalled lessons recurred.

## Scope

- `shared/scripts/lib/learning_recall.py`
- `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh`
- `skills/process/kbd-process-orchestrator/prompts/reflect.md`
- `shared/scripts/memory-writeback.sh`
- `shared/scripts/kbd-stage-writeback.sh`
- `skills/process/kbd-process-orchestrator/hooks/hooks.json`
- `skills/process/kbd-process-orchestrator/shared/lib/hooks.sh`
- `skills/process/kbd-process-orchestrator/skills/kbd-*/SKILL.md`
- `shared/scripts/tests/test-kbd-memory-loop.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
