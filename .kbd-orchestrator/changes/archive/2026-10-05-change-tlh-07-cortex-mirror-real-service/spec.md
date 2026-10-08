# change-tlh-07-cortex-mirror-real-service

**Title:** Verify the D3 Cortex mirror against the real Cortex 2.0.3 MCP server on a scratch CORTEX_DATA_DIR
**Plan PR:** 07
**Repository:** `prometheus-skill-system`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `e421715`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-07` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G3a

## Why

`test-cortex-mirror.sh` exercises the mirror only against a stub MCP server (line 4); real Cortex 2.0.3 is installed and honours `CORTEX_DATA_DIR`, so a real-service test is possible without touching `~/.cortex` (assessment G3a, analysis §G3a).

## What Changes

- Add a `real` case to `shared/scripts/tests/test-cortex-mirror.sh`: locate the installed Cortex under the real HOME *before* switching HOME, then pass it explicitly through the existing `PROMETHEUS_CORTEX_MCP` override (`learning_write.py::cortex_command`), because the default glob under a scratch HOME finds nothing; require its `node_modules/@xenova/transformers/.cache` model directory to exist (else BLOCKED, exit 2 — never download into the plugin cache); run with `CORTEX_DATA_DIR` and HOME pointing at scratch directories; write a lesson through `learning_write.py` with the mirror enabled; then call `cortex_recall` on the real server over stdio JSON-RPC by `projectId` and assert the lesson text and its role tag (in `context`) come back.
- Keep the stub cases (absent Cortex silent; misbehaving server does not block the write).
- If the real case exposes a mirror defect (payload shape, projectId mapping), fix it in `shared/scripts/lib/learning_write.py`.
- Document in `site/docs/memory/cortex-mirror.md`, registered in `site/sidebars.js` under the memory items, that the mirror is verified against Cortex 2.0.3. The test records the Cortex version it ran against and fails if it is not 2.0.3, unless `CORTEX_EXPECTED_VERSION` overrides it.

## Scope

- `shared/scripts/tests/test-cortex-mirror.sh`
- `shared/scripts/lib/learning_write.py`
- `site/docs/memory/cortex-mirror.md`
- `site/sidebars.js`
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

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-07-cortex-mirror-real-service` and backend task ID).

## Recalled lessons (from prior-context.md)

- "For every external integration, run at least one test against the real service before claiming the work is done." Applied as an acceptance criterion.
