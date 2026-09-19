# change-rah-003-stage-contract-driver

**Title:** Replace the no-op run-research.sh with a stage-contract driver that enforces artifacts, hooks, checkpoint, resume, scale gate, ordering, and provenance on every exit path
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G1
**Depends on:** `change-rah-002-research-package-contract`
**Backend:** native-kbd

## Why

`run-research.sh` prints started and completed for each stage, executes nothing, fires no hook, writes nothing, and reports complete (assessment G1, verified). `post-stage.sh` writes a checkpoint nothing reads. There is no `--resume`, no `--check-tools`, no slug, no provenance sidecar, no scale gate, and no ordering guarantee between verify and resolve. Analysis D-01 option B, D-04, D-05, D-06 (driver parts).

## What Changes

- Author `references/stage-contracts.md`: for each of the ten stages, the required input artifacts, the required output artifacts with their schema or shape, and the hook fired after it. This is the table the driver enforces.
- Rewrite `scripts/run-research.sh`: derives `<slug>` via `shared/scripts/lib/slug.sh` (relocated `subject_to_slug`); creates the package dir under the output root; writes `plan.md` with task ledger, verification log, and decision log sections; executes stages through `RESEARCH_STAGE_RUNNER` when set (a command receiving `<stage> <package-dir>`), otherwise stops at each stage boundary printing the exact stage skill the harness must run, emitting a `next_stage` JSON line on stdout, and exits 3 (checkpoint mode); after every stage validates the stage's required artifacts against `stage-contracts.md`; `--resume` skips stages whose artifacts exist and validate; `--check-tools` reports search, scrape, gateway, and memory availability without running; `--scale direct|full|auto` (auto reads the planner's sub-question count, fewer than three selects direct, which runs stages 01, 02, 03, 05, 09, 10 with no subagents); stage 06 refuses to start until stage 05's artifact validates.
- Fire all four existing hooks at defined points: `pre-research.sh` at run start before stage 01 (it validates query, depth, and job id and creates the package dir); `post-stage.sh` after every stage's artifacts validate; `on-contradiction.sh` after stage 06 when `contradictions.json` contains any entry with `resolved: false`; `post-export.sh` after stage 10's manifest validates. Each hook's exit code is recorded in the plan's verification log; a failing hook marks the stage `blocked` in the checkpoint and stops the run.
- Add `scripts/write-provenance.sh` producing `<slug>.provenance.md` (date, rounds, sources consulted, accepted, rejected, verification verdict PASS, PASS WITH NOTES, or BLOCKED, plan path, stage files) and call it from a trap so every exit path, including a failed stage, leaves a sidecar with the failure named.
- Add the integration test `tests/driver-contract.sh` with a fixture stage runner that writes schema-valid artifacts: full run, resume from stage 04 with earlier artifacts present, failing stage 05 yields a BLOCKED sidecar and non-zero exit, direct-scale run executes exactly the six direct stages, a stage 06 attempt with an invalid stage 05 artifact is refused, and a hooks-fired scenario in which each of the four hooks writes a marker under `RESEARCH_HOOK_LOG` and the test asserts all four markers exist in order (pre-research, ten post-stage, on-contradiction after a fixture with one unresolved contradiction, post-export).
- Add the driver requirements to `openspec/specs/research-pipeline-execution/spec.md`.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/scripts/run-research.sh`
- `skills/research/deep-research/scripts/write-provenance.sh`
- `skills/research/deep-research/references/stage-contracts.md`
- `skills/research/deep-research/templates/research-plan.md`
- `skills/research/deep-research/hooks/post-stage.sh`
- `skills/research/deep-research/tests/driver-contract.sh`
- `skills/research/deep-research/tests/fixtures/stage-runner.sh`
- `shared/scripts/lib/slug.sh`
- `shared/scripts/content-grounding-kb.sh`
- `openspec/specs/research-pipeline-execution/spec.md`
- `skills/research/deep-research/hooks/pre-research.sh` (package id derivation; review round 1)
- `skills/research/deep-research/hooks/on-contradiction.sh` (package path from the driver)
- `skills/research/deep-research/hooks/post-export.sh` (package path from the driver)
- `skills/research/deep-research/scripts/export-package.sh` (defers post-export to the driver)
- `skills/research/deep-research/SKILL.md` (shallow depth stage set)
- `skills/learn/learn-goal/scripts/content-grounding-kb.sh` (byte-identical copy of the shared script)
- `skills/learn/learn-kb/scripts/content-grounding-kb.sh` (byte-identical copy of the shared script)

## Capabilities

- `research-pipeline-execution (driver requirements added)`

## ADDED Requirements

### Requirement: Every stage boundary is enforced
WHEN a stage completes, THEN the driver validates that stage's required artifacts before firing `post-stage.sh` and before allowing the next stage; a missing or invalid artifact stops the run with the stage and path named.

#### Scenario: Invalid artifact
- **WHEN** the fixture runner writes an invalid stage 05 artifact
- **THEN** the driver exits non-zero, does not start stage 06, and the sidecar reads `Verification: BLOCKED` with stage 05 named

### Requirement: Resume is real
WHEN `--resume` is passed and `checkpoint.json` names a last completed stage, THEN stages up to and including it are skipped only if their artifacts validate.

#### Scenario: Resume from 04
- **WHEN** a package has valid artifacts for stages 01 to 04 and a checkpoint at 04
- **THEN** the driver runs stage 05 first and the fixture runner records no calls for 01 to 04

### Requirement: A sidecar exists on every exit
The driver SHALL write `<slug>.provenance.md` on success, on failure, and on interruption.

#### Scenario: Failure
- **WHEN** any stage fails
- **THEN** the sidecar exists, its verdict is BLOCKED, and the failing stage and reason appear in it

### Requirement: Narrow questions stay narrow
WHEN the planner emits fewer than three sub-questions and scale is auto, THEN the driver runs the direct stage set and spawns no subagent stage.

#### Scenario: Direct run
- **WHEN** the fixture planner emits two sub-questions
- **THEN** stages 04, 06, 07, 08 are not invoked

### Requirement: All four hooks fire at their defined points
The driver SHALL fire `pre-research.sh` before stage 01, `post-stage.sh` after each validated stage, `on-contradiction.sh` after stage 06 when an unresolved contradiction exists, and `post-export.sh` after stage 10.

#### Scenario: Hooks fired
- **WHEN** the fixture runner leaves one unresolved contradiction and the hooks log to `RESEARCH_HOOK_LOG`
- **THEN** the log contains one pre-research marker, one post-stage marker per executed stage, one on-contradiction marker, and one post-export marker, in that order

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build. `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason, never described as passed.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1); capability is discovered, never assumed (rule 2).
- Constraint C-01 (generated artifacts): `SKILL.md` and `skill.toml` files edited in this phase are inputs only to `generate:skills-index`; `skill-system.json`, the harness adapters, and the service manifest are not edited by any change in this phase. `change-rah-011-integration-evidence-and-docs` is the named reconciliation change: it regenerates the skills index and runs `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest` at certification. No earlier change claims distribution certification.

## Open Questions

- Checkpoint mode emits both a human line and a `next_stage` JSON line; whether harness adapters consume the JSON line is decided when the first adapter is written.

## Unresolved review findings

Adversarial review (diff mode, judge k3 via rest-gateway, `cross_model_check: verified-distinct`) ran two rounds against this change's scoped diff. Round 1: 1 CRITICAL, 5 WARNING, 1 SUGGESTION. Round 2 (after the round-1 fixes): 3 CRITICAL, 2 WARNING, 1 SUGGESTION. Under the two-round cap the round-2 findings are carried here with their disposition; every fix was proven by a scenario in `tests/driver-contract.sh` (61 assertions, bash 5 and bash 3.2) before archive, not re-vetted by the judge.

Round 1, all fixed and covered: shallow depth could never complete (stage set now `01 02 03 04 05 09 10`, `shallow-depth` scenario); post-export was recorded with a fabricated exit 0 (the driver now fires it itself after post-stage 10 with the real exit code, exporter defers via `RESEARCH_POST_EXPORT_BY_CALLER=1`); post-stage.sh could overwrite a driver checkpoint when jq was absent (guarded); the full-run verdict assertion could not tell PASS from PASS WITH NOTES (exact match); interim sidecar failure was silent (now blocks); ledger lines before plan.md existed were dropped (buffered in `.ledger-pending.md` and folded in, asserted); pre-research fell back to the job id as a directory name (derives a conforming id from the query, rejects without the library).

Round 2, all fixed and covered:
- CRITICAL: a recovered run kept a stale `.blocked` and the sidecar printed `Blocked: stage NN` beside a PASS verdict. Completing any stage now clears `.blocked`; the sidecar prints the blocked line only when status is `blocked`. Asserted in `resume-from-04` and `hook-failure`.
- CRITICAL: a hook failure at stage 10 wedged the run because stage 10 stayed in `stages_completed`. Stage 10 rolls back on post-stage or post-export failure; the same rollback now applies to every stage's post-stage and to on-contradiction, so a failed hook never leaves its stage counted. Asserted in `hook-failure` (stage 03 and stage 10, with recovery on resume and manifest and sidecar verdicts agreeing after recovery).
- CRITICAL: `completed_at` was defaulted without being named. Routed through the exporter's `get()`; the thin-export probe lists it.
- WARNING: interim sidecar failure only warned and still exited 3. It now records a block and exits 1.
- WARNING: no scenario exercised a failing hook. `hook-failure` added, using `RESEARCH_HOOK_DIR` to substitute a stub hook directory.
- SUGGESTION: pre-research duplicated the package-id regex. It now calls `package_id_is_valid` from the shared library, with the inline pattern kept only as the documented no-library fallback.
