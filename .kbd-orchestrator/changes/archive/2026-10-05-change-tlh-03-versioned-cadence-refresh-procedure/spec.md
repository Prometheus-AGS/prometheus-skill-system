# change-tlh-03-versioned-cadence-refresh-procedure

**Title:** Ship the skill-pack refresh procedure in delivery-cadence: profile inputs, state.json parity, fail-loud, submodule update
**Plan PR:** 03
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-03` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G1c

## Why

`.prometheus/cadence/procedures/refresh-skill-pack.sh` is git-ignored (`.gitignore:94`) and carries this machine's only copy of four recorded fixes: deploy-worktree freezing, state.json parity, submodule update after fast-forward, and the surreal-memory kickstart (assessment G1c).

## What Changes

- Add `skills/process/delivery-cadence/scripts/refresh-skill-pack.sh` (under the skill's existing `scripts/` directory, which the validators and distribution already carry; bash 3.2, `set -euo pipefail`) with inputs from flags or env: `--deploy <worktree>`, `--state <state.json>`, `--mode full|verify|auto`, `--services <launchd labels, comma-separated>`.
- `--mode auto` reads the iteration number from `state.json` directly (never the cadence CLI, which holds the lock inside a checkpoint); odd = full, even = verify. A missing, unreadable or non-numeric iteration exits 2 with a message and never defaults.
- Full mode: fast-forward the deploy worktree to `origin/main` (refuse a dirty or diverged worktree, exit 1), `git submodule update --init --recursive`, run `scripts/update-skill-pack.sh --force` and `scripts/install-binaries.sh` from the deploy worktree, `launchctl kickstart -k gui/$UID/<label>` for each service, then print a JSON summary (source commit, pk/worker/surreal-memory versions, health, plugin generation). Verify mode prints the same summary without changing anything.
- For tests only, and only when `REFRESH_TEST_MODE=1`, `REFRESH_UPDATE_CMD`, `REFRESH_INSTALL_CMD` and `REFRESH_KICKSTART_CMD` override the update, install and kickstart commands (defaults: the real scripts and `launchctl`).
- `references/profile.md` documents the procedure and how a profile points a checkpoint at it. The local `.prometheus/cadence/procedures/refresh-skill-pack.sh` becomes a shim that `exec`s `$HOME/.claude/skills/delivery-cadence/scripts/refresh-skill-pack.sh`, the installed flat path. The shim is committed as an example under `examples/`; the local file stays untracked.

## Scope

- `skills/process/delivery-cadence/scripts/refresh-skill-pack.sh`
- `skills/process/delivery-cadence/references/profile.md`
- `skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh`
- `skills/process/delivery-cadence/SKILL.md`
- `shared/scripts/tests/test-cadence-refresh-procedure.sh`
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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-03-versioned-cadence-refresh-procedure` and backend task ID).

## Recalled lessons (from prior-context.md)

- "`delivery-cadence` freezes every source named in `ready`; do **not** point those sources at the actively edited main checkout." and "run `git submodule update` before invoking `update-skill-pack.sh`".
- "Do not invoke a CLI or subprocess that takes a lock from inside code that already holds the same lock. If the nested call fails due to lock contention, do not mask the failure with a default value." Applied: no default N.
