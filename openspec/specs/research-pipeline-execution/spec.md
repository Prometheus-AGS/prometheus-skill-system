# research-pipeline-execution Specification

## Purpose
Define the contract for a deep-research run in the Prometheus skill pack: where a research package lives, what its manifest contains, how it is validated, and what the driver and daemon guarantee about execution. Authored by phase research-agent-hardening (change-rah-002); driver, daemon, label, review, and scoring requirements are added by changes rah-003 through rah-010.

## Requirements

### Requirement: A research package lives under one output root

Every research package SHALL be a directory named `<slug>-<yyyymmdd>-<4hex>` under `${RESEARCH_OUTPUT_DIR:-~/.prometheus/research}/`. The slug SHALL be at most five lowercase hyphenated words derived from the query. No component of the pipeline SHALL read or write `~/.research-jobs/`; the daemon's status output SHALL name that legacy directory while it exists so an operator can delete it by hand.

#### Scenario: Hooks and daemon agree on the root

- **GIVEN** `RESEARCH_OUTPUT_DIR` is unset
- **WHEN** any of the four deep-research hooks or the daemon resolves the output root
- **THEN** it resolves `~/.prometheus/research/`
- **AND** no hook source contains `.research-jobs`
- **AND** every daemon source line containing `.research-jobs` also contains the word `legacy`, so the only occurrences are the helper that names the legacy root and the status notice.

#### Scenario: Legacy directory is named, not read

- **GIVEN** `~/.research-jobs/` still exists with entries
- **WHEN** `prometheus-research status <job>` or `prometheus-research --mode status` runs
- **THEN** stderr names the legacy path and its entry count and states that it is no longer read.

### Requirement: The manifest is schema-validated and complete

`manifest.json` at the package root SHALL validate against `references/schemas/research-manifest.schema.json`. Every required field SHALL be present; a value no artifact provides SHALL be emitted as `null`, `0`, `false`, or an empty collection and named on stderr as defaulted by the exporter, never omitted. `stages_completed` SHALL be an ordered array of stage ids, not a count.

#### Scenario: Export from a complete package

- **GIVEN** a package with `checkpoint.json`, `report.md`, a provenance sidecar, `sources/registry.json`, `graph.json`, and `contradictions.json`
- **WHEN** `scripts/export-package.sh <package>` runs
- **THEN** `manifest.json` validates against the schema
- **AND** `sources_count`, `claims_count`, and the three contradiction counts equal the counts in those artifacts.

#### Scenario: Export from a thin package

- **GIVEN** an empty package directory
- **WHEN** `scripts/export-package.sh <package>` runs
- **THEN** `manifest.json` still validates
- **AND** stderr lists every defaulted field.

### Requirement: The normative spec and its copies do not drift

`references/research-package-spec.md` SHALL be the only normative prose contract, and `references/schemas/research-manifest.schema.json` the only machine contract. The `manifest.json` example in `SKILL.md` SHALL be byte-for-byte the example in the spec, and the A2UI component table in `SKILL.md` SHALL list exactly the components registered in `substrate/prometheus-research/src/a2ui/registry.rs`. `scripts/check-research-package.sh` SHALL enforce both and exit non-zero on any drift.

#### Scenario: Drift check passes on a consistent tree

- **WHEN** `scripts/check-research-package.sh` runs with no arguments
- **THEN** the SKILL.md example, the spec example, and a fresh export all validate
- **AND** the SKILL.md example equals the spec example
- **AND** the A2UI table equals the registry
- **AND** the exit code is 0.

#### Scenario: Drift check fails on a broken manifest

- **GIVEN** a package whose manifest omits one required field
- **WHEN** `scripts/check-research-package.sh --package <dir>` runs
- **THEN** the failing field is named on stderr
- **AND** the exit code is non-zero.

#### Scenario: Validation without python jsonschema

- **GIVEN** python3 is present but `jsonschema` is not importable
- **WHEN** the drift check runs
- **THEN** it compares the full key set and declared types against the schema
- **AND** it reports `PASS WITH NOTES` rather than claiming a full schema validation.

### Requirement: Package-level verification labels are derived

`verification_status` SHALL be one of `verified`, `partial`, `unverified`, and `verification_verdict` SHALL be one of `PASS`, `PASS WITH NOTES`, `BLOCKED`. The manifest values SHALL equal the `report.md` frontmatter and the provenance sidecar; the package-mode drift check SHALL compare them.

#### Scenario: Sidecar and manifest agree

- **GIVEN** a package whose provenance sidecar reads `**Verification:** BLOCKED`
- **WHEN** the package-mode drift check runs
- **THEN** it fails unless `manifest.json.verification_verdict` is `BLOCKED`.

### Requirement: Every stage boundary is enforced by the driver

`scripts/run-research.sh` SHALL validate each stage's required artifacts against `references/stage-contracts.md` before recording the stage in `checkpoint.json`, before firing the stage's hooks, and before allowing the next stage. A missing or invalid artifact SHALL stop the run with the stage and reason recorded in `checkpoint.json.blocked`, and the driver SHALL exit non-zero. A run whose checkpoint status is neither `complete` nor `awaiting_stage` SHALL never exit 0.

#### Scenario: Invalid stage 05 artifact

- **GIVEN** the stage runner writes an invalid `sources/credibility.json`
- **WHEN** the driver validates stage 05
- **THEN** it exits non-zero, `checkpoint.json.status` is `blocked` with `blocked.stage` `05` and a reason containing `artifact contract`
- **AND** stage 06 is never invoked
- **AND** no `manifest.json` is exported.

#### Scenario: Stage 06 refuses to start without a valid stage 05

- **GIVEN** stage 05 is recorded complete but `sources/credibility.json` no longer validates and cannot be regenerated
- **WHEN** the driver resumes
- **THEN** stage 06 is never invoked and the block names stage 05.

### Requirement: Resume skips only stages whose artifacts still validate

WHEN `--resume` is given, THEN a stage listed in `checkpoint.json.stages_completed` SHALL be skipped only if its artifacts validate now; a listed stage whose artifact is missing or corrupt SHALL be re-run and a note recorded. `pre-research.sh` SHALL not fire again on resume.

#### Scenario: Resume from stage 04

- **GIVEN** a run that blocked at stage 05 with stages 01 to 04 complete
- **WHEN** the driver is re-invoked with `--resume <package>` and a working runner
- **THEN** the runner is invoked for stages 05 to 09 only
- **AND** the final checkpoint lists ten completed stages.

### Requirement: A provenance sidecar exists on every exit

The driver SHALL write `<slug>.provenance.md` on success, on a block, and on interruption, via `scripts/write-provenance.sh` from `checkpoint.json` and the package artifacts. The verdict SHALL be `BLOCKED` when the run blocked or ended incomplete, `PASS WITH NOTES` when the run completed at direct scale, carried notes, had a failing hook, or completed a full run without an adversarial review, and `PASS` otherwise. A run awaiting a stage in checkpoint mode SHALL write an interim sidecar that is not `BLOCKED`.

#### Scenario: Failure leaves a BLOCKED sidecar

- **GIVEN** the stage runner exits non-zero at stage 05
- **WHEN** the driver exits
- **THEN** the sidecar exists, its verdict is `BLOCKED`, and the failing stage is named on its `Blocked` line.

### Requirement: Narrow questions stay narrow

WHEN `--scale auto` (the default) and stage 01's `plan.md` lists fewer than three `## Sub-questions` bullets, THEN the driver SHALL run the direct stage set `01 02 03 05 09 10`, record `scale: direct` in `checkpoint.json` and `manifest.json`, and write the decision to the plan's `## Decision log`. Three or more bullets SHALL select `full`.

#### Scenario: Direct run

- **GIVEN** the planner emits two sub-questions
- **WHEN** the run completes
- **THEN** the runner was invoked for stages 01, 02, 03, 05, 09 only
- **AND** `manifest.json.scale` is `direct`
- **AND** the sidecar verdict is `PASS WITH NOTES`.

### Requirement: All four hooks fire at defined points

The driver SHALL fire `pre-research.sh` once before stage 01, `post-stage.sh` after each validated stage including stage 10, `on-contradiction.sh` after stage 06 when `contradictions.json` has an entry with `resolved: false`, and `post-export.sh` after stage 10's export. Each firing SHALL be recorded in `checkpoint.json.hook_log` with its exit code and appended to `RESEARCH_HOOK_LOG` when that variable names a file. A hook exiting non-zero SHALL block the stage.

#### Scenario: Hook markers in order

- **GIVEN** a full run whose stage 06 leaves one unresolved contradiction and `RESEARCH_HOOK_LOG` names a file
- **WHEN** the run completes
- **THEN** the file lists, in order: `pre-research`, `post-stage 01` through `post-stage 06`, `on-contradiction 06`, `post-stage 07` through `post-stage 10`, `post-export`
- **AND** a run with no unresolved contradiction lists no `on-contradiction` marker.

### Requirement: Checkpoint mode hands the next stage to the harness

WHEN `RESEARCH_STAGE_RUNNER` is unset, THEN the driver SHALL validate what exists, stop at the first incomplete stage, print the stage skill to run, emit one JSON line `{"next_stage", "skill", "package_dir"}` on stdout, set `checkpoint.json.status` to `awaiting_stage`, and exit 3. Re-invoking with `--resume` after the harness has run that stage SHALL advance to the next incomplete stage.

#### Scenario: First stop and advance

- **GIVEN** no runner is set
- **WHEN** the driver starts a new run
- **THEN** it exits 3 with `next_stage` `01`
- **AND** after `plan.md` is written and the driver is resumed, it exits 3 with `next_stage` `02` and `stages_completed` is `["01"]`.

### Requirement: The driver reports tool availability without running

`run-research.sh --check-tools` SHALL report the presence or absence of the search providers, python3, jq, the model gateway, surreal-memory, a stage runner, and the output root, and SHALL exit non-zero only when jq is missing.

#### Scenario: No search key

- **GIVEN** neither `TAVILY_API_KEY` nor `FIRECRAWL_API_KEY` is set
- **WHEN** `--check-tools` runs
- **THEN** the search line reads `NONE` with a note that stage 02 will be blocked, and the exit code is 0 because jq is present.
