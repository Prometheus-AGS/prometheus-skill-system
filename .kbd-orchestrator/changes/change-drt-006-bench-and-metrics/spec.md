# change-drt-006-bench-and-metrics

**Title:** Make "on par with Onyx" falsifiable: adopt the RACE criteria, build FACT and verified-claim ratio
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G5
**Depends on:** `change-drt-002-thread-scheduler-and-budgets`, `change-drt-003-director-worker-agents`, `change-drt-004-deterministic-merge`, `change-drt-005-multipass-report`

> Benchmarking the *sequential* pipeline would measure the thing this phase exists to replace. The full threaded path must exist before a parity number means anything (adversarial round-1 finding 2).
**Backend:** native-kbd

## Why

Analyze **D-10**, and the verdict changed under adversarial review. G5 originally
had no candidate and was defaulting to build. Research found
`Ayanami0730/deep_research_bench` — **Apache-2.0**, 827 stars, the reference
implementation of the very benchmark Onyx's standing is quoted against. It ships
`data/criteria_data/criteria.jsonl`, the task set in
`data/prompt_data/query.jsonl`, the criteria prompts, and the RACE scorer.

Adopting the published criteria is the whole point: a home-grown rubric produces
a number comparable to nothing. Scoring against the same criteria is what turns
"on par with Onyx" from a slogan into a claim that can be checked — including
checked and found false.

Onyx's own standing (RACE ~54, "reported as #1 at points in 2026") remains
**unverified** (assessment V4) and must be labelled as such until measured here.

## What Changes

- `tests/bench/`: adopts the criteria, the English criteria prompt, and a 10-task
  subset of the query set, **pinned at commit `469cce54`** so scores are
  reproducible against a fixed rubric. Attribution recorded per Apache-2.0.
- The Python scorer is **not** ported wholesale; the harness is bash driving the
  pack's existing gateway (`judge` routes to `k3`, distinct from the producer, so
  judge never equals producer).
- Builds what the benchmark does not provide: **FACT** metrics (effective
  citations, citation accuracy) computed from `citations.json` plus claim labels,
  and the **verified-claim ratio** — the metric Onyx structurally cannot report,
  and the pack's differentiator.
- `tests/bench/BENCH-RESULTS.md` records per-run RACE overall, effective
  citations, citation accuracy, and verified-claim ratio, with every number
  labelled `verified` or `unverified` per the phase vocabulary.

## Scope

- `skills/research/deep-research/tests/bench/run-bench.sh`
- `skills/research/deep-research/tests/bench/score-race.sh`
- `skills/research/deep-research/tests/bench/score-fact.py`
- `skills/research/deep-research/tests/bench/criteria/` (adopted, pinned, attributed)
- `skills/research/deep-research/tests/bench/queries/` (10-task subset)
- `skills/research/deep-research/tests/bench/BENCH-RESULTS.md`
- `skills/research/deep-research/tests/bench/ATTRIBUTION.md`

## Capabilities

- `research-pipeline-execution (benchmark added)`

## ADDED Requirements

### Requirement: Scores are reproducible against a pinned rubric
The adopted criteria and queries SHALL be pinned to a named upstream commit, and the pin SHALL be recorded beside the results.

#### Scenario: Pin recorded
- **WHEN** a bench run completes
- **THEN** `BENCH-RESULTS.md` names the upstream commit `469cce54`, the judge model, and the date

### Requirement: The judge is not the producer
The RACE judge SHALL be a model distinct from the one that produced the report under test.

#### Scenario: Distinct judge
- **WHEN** a bench run dispatches the judge
- **THEN** the judge model differs from the producer, and a run where they match is recorded BLOCKED rather than scored

### Requirement: Four numbers, every one labelled
Each run SHALL report RACE overall, effective citations, citation accuracy, and verified-claim ratio, each carrying a `verified` or `unverified` label.

#### Scenario: Onyx comparison is honest
- **WHEN** results are compared against Onyx's published figure
- **THEN** Onyx's number is labelled `unverified` unless Onyx was run locally on the same subset, and the comparison says which

### Requirement: A regression is visible
WHEN a bench run follows an earlier one, THEN a movement in any of the four numbers SHALL be recorded rather than overwritten.

#### Scenario: Movement recorded
- **WHEN** a second run scores differently
- **THEN** both runs appear in `BENCH-RESULTS.md` with their dates and the delta stated

## Constraints

- Implementation-first, integration-only evidence: no unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred`; provenance is `PASS | PASS WITH NOTES | BLOCKED`. A gate that could not run is recorded BLOCKED with the reason.
- Scripts stay bash 3.2 compatible (C-05); test under `/bin/bash`.
- Every script keeps `set -euo pipefail` and non-zero exit on failure.
- The pack never depends on the Companion or any extension.
- **Apache-2.0 attribution is mandatory** for the adopted criteria, prompts, and queries; `ATTRIBUTION.md` is part of the deliverable, not optional.
- A bench run costs real gateway tokens; the 10-task subset is the budget ceiling for this phase.

## Open Questions

- Whether 10 tasks is statistically meaningful for RACE, or only indicative (default: treat as indicative and say so in `BENCH-RESULTS.md`; the full 100 is a later decision).
- Whether to run Onyx locally for a true head-to-head. Defaults to no this phase: it needs an Onyx deployment and its own API keys, and the published figure with an `unverified` label is honest without it.
