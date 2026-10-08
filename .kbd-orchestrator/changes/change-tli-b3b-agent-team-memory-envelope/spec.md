# change-tli-b3b-agent-team-memory-envelope

**Title:** agent-team runtime memory writer adopts the learning envelope, agent_id table and a required project id
**Plan PR:** B3b
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b3-envelope-write-library`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b3b` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §2, §6 (analysis D-2)

## Why

`skills/process/agent-team-creator/runtime/src/memory.mts` already writes team memories with its own `{kind:'agent-team-memory'}` envelope and nullable `user_id` (lines 56-66); recall would miss them and two incompatible write paths would exist (assessment review C1, analysis D-2).

## What Changes

- Record content becomes the learning envelope (`kind` mapped from the entry scope); categories follow the B3 encoding.
- `agent_id` follows the design table; `user_id` comes from the caller-supplied `scopeMapping.userId`/`kbd.projectId`; memory.mts fails closed (throws) when absent. The CLI entries `memory-queue`/`memory-publish` resolve it once via `project-id.sh --json` located through `CLAUDE_PLUGIN_ROOT`/`PLUGIN_ROOT` (fallback: repo-relative for source runs).
- Outbox, digest ids and idempotency unchanged; the `mapped-http` provider unchanged and not recalled.
- Rebuild the runtime's committed `scripts/*.mjs` output.

## Scope

- `skills/process/agent-team-creator/runtime/src/memory.mts`
- `skills/process/agent-team-creator/runtime/src/cli.mts`
- `skills/process/agent-team-creator/runtime/test-src/memory-envelope.integration.mts`
- `skills/process/agent-team-creator/scripts/*.mjs`
- `skills/process/agent-team-creator/tests/*.mjs`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
