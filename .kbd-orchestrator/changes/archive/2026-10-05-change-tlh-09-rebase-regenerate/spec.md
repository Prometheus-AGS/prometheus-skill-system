# change-tlh-09-rebase-regenerate

**Title:** scripts/rebase-regenerate.sh resolves generated-only rebase/merge conflicts by regenerating, then validates
**Plan PR:** 09
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-09` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G3c

## Why

Six PRs last phase needed rebases, mostly in generated files (hook bundles, harness manifests, `dist/**`), each resolved by hand-running the generators (assessment G3c, previous reflection Delta 2).

## What Changes

- `scripts/generated-paths.mjs` prints the authoritative generated-path set derived from the generators' declared outputs: the `skill-system.json` target matrix used by `generate-skill-system-distribution.js` (which is also what `build:codex` runs), and the output list of `generate-harness-adapters.js`, exported by each generator as a function or `--list-outputs` flag. `check:distribution` and the new script share it.
- `scripts/rebase-regenerate.sh` (bash 3.2): requires an in-progress rebase or merge; lists conflicted paths; exits 1 naming them if any is not in the generated set; otherwise resolves each to either side (content discarded), runs `node scripts/generate-harness-adapters.js` and `node scripts/generate-skill-system-distribution.js` (the same script `build:codex` runs), then `npm run check:distribution`, `npm run validate:harness-adapters`, `npm run validate:codex`; on success stages the generated paths and prints the `git rebase --continue` / `git commit` command (it never continues itself).
- `docs/CONTRIBUTING.md` documents it; an optional `.gitattributes` merge-driver registration is described but not installed.

## Scope

- `scripts/generated-paths.mjs`
- `scripts/rebase-regenerate.sh`
- `scripts/generate-skill-system-distribution.js`
- `scripts/generate-harness-adapters.js`
- `scripts/tests/test-rebase-regenerate.sh`
- `docs/CONTRIBUTING.md`

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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-09-rebase-regenerate` and backend task ID).

## Recalled lessons (from prior-context.md)

- "For each generated format, maintain exactly one authoritative emitter and one authoritative validator." Applied: regenerate with the repository emitters, then run the paired validators.
