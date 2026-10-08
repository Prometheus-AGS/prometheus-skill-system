# change-tlh-02-codex-memories-install-and-doctor

**Title:** The installer sets Codex `[memories] generate_memories = false` and archives memory_summary.md; the doctor checks it
**Plan PR:** 02
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-02` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G1b

## Why

Codex memory generation is disabled only in the operator's hand-edited `~/.codex/config.toml`; a fresh machine regrows `memory_summary.md`, which Codex injects into every thread (assessment G1b). The agent-team exporter already sets the option per generated Codex agent (`adapters-local.mts:26`) but not for the main thread.

## What Changes

- New `shared/scripts/codex-memories-config.sh` (bash 3.2): idempotently inserts or updates `generate_memories = false` under a `[memories]` table in `${CODEX_HOME:-$HOME/.codex}/config.toml` with a line-level edit that preserves every other line and comment; writes a timestamped backup first; re-parses the result with `python3 -c 'import tomllib'` and restores the backup on a parse failure; archives an existing `memories/memory_summary.md` to `memories-archive/memory_summary-<UTC>.md`; leaves `MEMORY.md`, `raw_memories.md` and every other file in place. `--check` reports state as JSON without writing.
- Both installers apply it: `scripts/install-system.js` (the `prometheus setup` / `install.sh` path, which runs its own Codex step and does not call install-skills-flat) spawns the script after its Codex plugin step, and `install_to_codex()` in `scripts/install-skills-flat.sh` calls it through a guarded helper `scripts/lib/install-codex-memories.sh`. Every failure prints a warning and the install completes (installer-must-not-abort). Absent Codex (no `~/.codex`) is silent. `CODEX_MEMORIES_SCRIPT` overrides the script path, for fault injection in tests only.
- `prometheus doctor` gains an optional check `codex.memories`: Green when `generate_memories` is `false` and no `memory_summary.md` exists; Yellow otherwise, with the exact repair command; skipped (Green, note) when Codex is not installed. It never counts toward failed checks.
- Docs: `docs/guide/06-memory-and-learning.md` and `site/docs/operations/doctors-and-mac-certification.md` describe the setting, the archive and the check.

## Scope

- `shared/scripts/codex-memories-config.sh`
- `scripts/install-skills-flat.sh`
- `scripts/lib/install-codex-memories.sh`
- `scripts/install-system.js`
- `shared/scripts/tests/test-codex-memories-config.sh`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs`
- `tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs`
- `docs/guide/06-memory-and-learning.md`
- `site/docs/operations/doctors-and-mac-certification.md`
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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-02-codex-memories-install-and-doctor` and backend task ID).

## Recalled lessons (from prior-context.md)

- "Install Codex memory settings by hand only" was knowledge gap 2 in `prior-context.md`; this change closes it.
