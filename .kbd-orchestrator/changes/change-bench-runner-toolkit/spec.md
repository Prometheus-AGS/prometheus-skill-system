# change-bench-runner-toolkit

**Title:** Bench runner toolkit — stage runner via gateway, evidence labeller, multi-task orchestrator, bench runbook
**Repository:** `prometheus-skill-pack`
**Phase:** bench-runner-automation (child of deep-research-onyx-parity)
**Goal:** G1–G4 (this change delivers all four); G5–G6 (close drt-006 task 4 + parent reflection update) are follow-on activities executed by the orchestrator
**Backend:** native-kbd

## Why

The parent's G5 (RACE/falsifiable measurement) is PARTIAL — only 1 of 10 bench packages built (task-51, RACE 64.0), with verified_claim_ratio=0.0 because the produced claims lack `verified`/`unverified` labels. Nine tasks remain; the agent-driven pipeline scales poorly (10 stages × bespoke-research-per-domain × ~25 min agent time = 2-3 hours minimum, with high context cost). The pipeline was *designed* for autonomous execution: `run-research.sh:17` defines `RESEARCH_STAGE_RUNNER` as the hook that drives each stage via an external command (which can be a gateway call). That path is unused. This change implements the runner, closes the labels gap with a deterministic labeller, and bundles a multi-task orchestrator + bench runbook so the remaining 9 packages are produced + scored automatically.

## What Changes

- New `skills/research/deep-research/scripts/run-stage.sh` — the `RESEARCH_STAGE_RUNNER` implementation. Bash 3.2. Reads the stage SKILL.md from `skills/research/deep-research/skills/stage-NN-name/`, builds a prompt (SKILL.md + package state context), POSTs to `http://localhost:4000/v1/chat/completions` (or falls back to `http://localhost:8181/v1` per the gateway candidates list in `~/.prometheus/kbd/models.toml`), parses the response, and writes the artifact per the stage's contract. Retries with backoff on 5xx/timeout. Per-stage timeouts.
- New `skills/research/deep-research/scripts/label-claims.py` — the evidence labeller. Reads `sources/registry.json`, adds `verified`/`unverified`/`partial` to each claim based on (a) source credibility score from `sources/credibility.json` (>=70 → verified, 40-69 → partial, <40 → unverified), (b) presence of the verbatim quote in at least one chunk text, (c) source tier (primary-statistical > secondary). Idempotent.
- New `tools/bench-automation/run-bench-suite.sh` — the multi-task orchestrator. Drives all 10 queries from `subset-10.jsonl` (51, 58, 66, 71, 75, 79, 81, 83, 85, 87) end-to-end via run-research.sh + run-stage.sh + label-claims.py, with retry/backoff on gateway rate-limits, watchdog per task (~40 min ceiling), sequential with bounded concurrency if the gateway holds. Writes each package to `/Users/gqadonis/.prometheus/research/bench-g5/<task-id>/<slug>-<id>/` per run-research.sh's natural layout.
- New `tools/bench-automation/bench-runbook.sh` — calls `run-bench.sh --packages <bench-g5>` (after creating the symlink `<bench-g5>/task-<id>` → the package's run-research slug-dir to fix run-bench's flat-layout assumption), runs `score-race.sh` + `score-fact.py` per task, and appends a per-task entry to `tests/bench/BENCH-RESULTS.md` (RACE + effective_citations + citation_accuracy + verified_claim_ratio).
- New `tools/bench-automation/README.md` — one-page operator doc: how to run one task, how to run the suite, where the outputs land, how to retry on failure.

## Scope

- `skills/research/deep-research/scripts/run-stage.sh` (new)
- `skills/research/deep-research/scripts/label-claims.py` (new)
- `tools/bench-automation/run-bench-suite.sh` (new)
- `tools/bench-automation/bench-runbook.sh` (new)
- `tools/bench-automation/README.md` (new)

No edits to existing scripts (run-research.sh, the stage SKILL.md files, run-bench.sh) — the runner invokes them as-is.

## Capabilities

- `research-pipeline-execution (autonomous)` — adds automatic stage execution
- `research-quality-assurance (claim labeling)` — closes the drt-006 verified-claim-ratio gap
- `bench-suite-orchestration` — new capability

## ADDED Requirements

### Requirement: A single command drives all 10 bench tasks end-to-end
The orchestrator SHALL take no arguments (or `--tasks 51,58,...`) and produce a complete bench-g5 directory with 10 valid packages, each scored via score-race.sh + score-fact.py.

#### Scenario: Full 10-task suite runs to completion
- **WHEN** `run-bench-suite.sh` is invoked
- **THEN** within wall-clock and gateway-quota bounds, all 10 packages are produced with `manifest.json` + `index.md` + `report.md`, and `BENCH-RESULTS.md` carries a row per task with the four numbers

### Requirement: Stages execute without agent invocation
The runner SHALL call each stage's gateway model directly (no checkpoint-mode agent loop), so the pipeline does not require the orchestrating model to advance stage-by-stage.

#### Scenario: Stage 01 of task-66 completes in one runner call
- **WHEN** the runner is invoked with `<stage>=01 <pkg>=<task-66-pkg>`
- **THEN** the package has a valid `plan.md` with `- ` bullets under `## Sub-questions`

### Requirement: Claims carry checkable labels
After `label-claims.py`, every claim in `sources/registry.json` carries a `verified` | `unverified` | `partial` label keyed to the credibility scorer + chunk-text presence + source tier.

#### Scenario: Verified-claim-ratio is no longer zero across all 10 tasks
- **WHEN** `score-fact.py --package <pkg>` runs for every bench-g5 task
- **THEN** the `verified_claim_ratio` field is > 0 for each (and reflects actual labeled evidence, not a fabricated label)

### Requirement: Gateway failures don't poison the suite
The runner SHALL retry transient failures (5xx, timeout, network) with exponential backoff (max 3 attempts) and continue to the next task on permanent failure, recording a SKIPPED state with the last error.

#### Scenario: Gateway rate-limits one task; the suite still completes
- **WHEN** task 66 hits a rate-limit error
- **THEN** task 66 is recorded as SKIPPED with the error, and tasks 67-87 still execute

## Constraints

- Implementation-first, integration-only evidence (no mocks; real gateway calls).
- Local-only validation; no hosted CI.
- bash 3.2 compatible (C-05); set -euo pipefail.
- All scripts use the KBD-wired gateway candidates order (liter-llm :4000 first, openai-proxy :8181 second) per `~/.prometheus/kbd/models.toml` [gateway].
- KBD_PRODUCER_MODEL = glm-5.3 throughout.
- No secrets in tracked files (C-02).
- Generator outputs (dist/plugins/**) stay in sync (C-01); no source edits, no dist changes.

## Open Questions

- Per-task time budget: how aggressive can the orchestrator be before the gateway rejects? Empirically the gateway holds at moderate concurrency; default sequential with one-task-at-a-time and retry/backoff is the safe start; concurrency can be added later if the gateway permits.
- Should label-claims.py be invoked as a separate stage or fold into stage 04? Fold into 04 is cleaner (claims carry labels before the downstream stages read them); implement as a hook at the end of stage 04.
- Does the orchestrator overwrite or skip a task if a package already exists at the output path? Skip if `manifest.json` + `report.md` both present; this makes the suite idempotent (resume from failure).

## Tasks

1. Implement run-stage.sh: env RESOURCE_GATEWAY probe + gateway candidates; per-stage prompt assembly from stage SKILL.md + package state; chat-completions POST with retry/backoff (3 attempts, exponential 1s/4s/16s, per-attempt timeout 600s); response parsing with content extraction; artifact write per stage contract; stage-aware timeouts (01/05/06/07/09 = 600s, others = 300s).
2. Implement label-claims.py: idempotent Python; reads registry.json + credibility.json + chunk texts; adds `verified|partial|unverified` per claim based on: credibility_score thresholds (≥70 verified, 40-69 partial, <40 unverified), tier (primary-statistical > secondary-analysis-of-primary > primary-research-institute > primary-government > primary-policy-research > secondary-analysis), quote-presence check (must appear in at least one chunk text); updates registry.json in place; emits a label_summary.json with counts.
3. Implement run-bench-suite.sh: reads subset-10.jsonl; for each task id: ensure output root exists, invoke run-research.sh with the query + depth deep, after each stage's resume call run-stage.sh for the next stage (until 10 done or timeout), invoke label-claims.py after stage 04, invoke detect-contradictions.sh (stage 06), invoke build-graph.sh (stage 07), write citations.json (stage 08), resume to stage 09 (runner generates report.md per stage SKILL contract), stage 10 (export via run-research.sh); retry/backoff per task on transient failures; SKIP + log on permanent; watchdog per task 40 min.
4. Implement bench-runbook.sh: symlink `<bench-g5>/task-<id>` → the package's run-research slug-dir; invoke `KBD_PRODUCER_MODEL=glm-5.3 bash tests/bench/run-bench.sh --packages <bench-g5>`; per scored task, append a row to `tests/bench/BENCH-RESULTS.md` with task id, RACE, effective_citations, citation_accuracy, verified_claim_ratio, label; emit a `bench-summary.json` with the 10-task array.
5. Implement README.md: how to run one task end-to-end (commands), how to run the suite (`run-bench-suite.sh`), output layout (`<bench-g5>/<task-id>/<slug>-<id>/`), retry semantics, where the BENCH-RESULTS row appears.
6. Integration smoke: run task-58 end-to-end through the toolkit (validate the runner + labeller + orchestrator produce a package that scores); check the verified_claim_ratio is > 0 for task-58 specifically.
