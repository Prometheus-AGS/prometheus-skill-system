# change-tlh-06-ledger-reconciliation

**Title:** kbd-apply mark-done syncs the ledger; kbd-apply reconcile detects and repairs task/ledger drift; cadence and reflect run it
**Plan PR:** 06
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** `change-tlh-03-versioned-cadence-refresh-procedure`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-06` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G2c

## Why

`kbd-apply.sh:681-683`: `mark-done` flips the backend flag only, so last phase's ledger showed 13 of 25 changes while every `tasks.json` said done, and nothing warned (assessment G2c).

## What Changes

- `mark-done <change> <id>` calls `sync_progress` after `b_mark_done` (same as `end-task`, without firing hooks) and prints that hooks were not fired.
- New `reconcile [<phase>] [--repair] [--json]`: for every change in the phase plan, compare the backend's done flags (native-kbd `tasks.json`, openspec `tasks.md`) with the canonical runtime ledger; print one line per drifted task and exit 1 on drift, 0 when clean. `--repair` replays each drifted task through the `begin-task`/`end-task` path and re-checks.
- Inputs `reconcile` reads: the phase's change list from the canonical runtime (`prometheus kbd --path <root> status --json`, the active phase's changes), each change's backend task file, and the ledger projection `.kbd-orchestrator/phases/<phase>/progress.json` (`changes[].tasks`). It never writes the ledger except through `--repair`'s begin-task/end-task path.
- `/kbd-reflect` SKILL.md step 4 runs `kbd-apply reconcile` before reading progress.json and stops on drift with the repair command.
- The delivery-cadence refresh procedure (change 03) gains `--kbd-root <repo>` and `--reconcile-phase <phase|auto>` (auto = `phase` from `<kbd-root>/.kbd-orchestrator/current-waypoint.json`). The committed shim example passes `--kbd-root` and `--reconcile-phase auto`, so every cadence iteration launched through the shim runs the check. The procedure that runs `kbd-apply reconcile` read-only in verify and full modes and reports drift in its JSON summary (it reads files; it never calls the cadence CLI).

## Scope

- `skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh`
- `skills/process/kbd-process-orchestrator/skills/kbd-apply/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md`
- `skills/process/delivery-cadence/scripts/refresh-skill-pack.sh`
- `shared/scripts/tests/test-kbd-apply-reconcile.sh`
- `shared/scripts/tests/test-cadence-refresh-procedure.sh`
- `skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh`
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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-06-ledger-reconciliation` and backend task ID).

## Recalled lessons (from prior-context.md)

- "`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger." Applied: task closure in this change uses `begin-task`/`end-task`.
- "Avoid reentrant lock-taking CLIs and default fallbacks." Applies to the reconcile check invoked inside the cadence checkpoint: it reads files, never the cadence CLI.
