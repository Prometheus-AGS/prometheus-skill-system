# change-tlh-04-scratch-surreal-for-envelope-test

**Title:** Shared scratch surreal-memory test library; memory-envelope.integration runs against it, never the live :23001
**Plan PR:** 04
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-04` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G2a

## Why

`runtime/test-src/memory-envelope.integration.mts:13` defaults to the live `http://127.0.0.1:23001` service; it flaked twice at load ~250 and writes test records into the operator's store (assessment G2a).

## What Changes

- Extract the scratch-server start/stop steps of `shared/scripts/tests/test-kbd-memory-loop.sh` into `shared/scripts/tests/lib/scratch-surreal.sh` (bash 3.2): `scratch_surreal_start` picks a free port, creates a temp data dir, starts `surreal-memory-server` (binary from `TLI_SM_BIN` or PATH, >= 1.10.0, with the local embedding executor), waits for `/health`, exports `SURREAL_MEMORY_URL`; `scratch_surreal_stop` kills it and removes the dir; a missing binary or old version calls `blocked` (exit 2).
- `test-kbd-memory-loop.sh` and `test-subagent-delivery.sh` source the library instead of their inline copies (behaviour unchanged).
- New `skills/process/agent-team-creator/tests/run-memory-envelope.sh` starts a scratch server via the library, runs the compiled `memory-envelope.integration.mjs`, stops the server.
- `memory-envelope.integration.mts` drops the `:23001` default: without `SURREAL_MEMORY_URL` it exits 2 (BLOCKED); rebuild `tests/*.mjs` with `npm run build:tests`.

## Scope

- `shared/scripts/tests/lib/scratch-surreal.sh`
- `shared/scripts/tests/test-kbd-memory-loop.sh`
- `shared/scripts/tests/test-subagent-delivery.sh`
- `skills/process/agent-team-creator/runtime/test-src/memory-envelope.integration.mts`
- `skills/process/agent-team-creator/tests/memory-envelope.integration.mjs`
- `skills/process/agent-team-creator/tests/run-memory-envelope.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- The last task regenerates generated outputs (`node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js`, which is also `build:codex`) before the gate runs `check:distribution`.
- A change with a Depends-on starts from `origin/main` after that dependency has merged; its verify.sh exits 2 (BLOCKED) when the dependency is absent from the base.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.

## Plan reference

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-04-scratch-surreal-for-envelope-test` and backend task ID).
