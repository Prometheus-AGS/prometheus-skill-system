---
id: change-learn-021
title: learn-goal + learn-survey + feynman-loop integration test
type: test
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-007
  - change-learn-008
  - change-learn-009
  - change-learn-010
---

# change-learn-021 — basic flow integration test

## Summary

Write an integration test script that exercises the happy-path flow from
`learn-goal` through `learn-survey`, `feynman-loop`, and `learn-grade` using
Tier 0 only (no UI surface required). The test uses a fixture KB and asserts
that all four primary artifact files are produced with valid structure. Also
verifies that the sycophancy-correction binary is invoked within `learn-grade`
output.

## Motivation

The core Feynman loop spans four skills. Without an integration test, regressions
in the artifact handoff contract between skills are caught only in production.

## Scope

- New test: `tests/learn/integration-basic-flow.sh`
- Fixture KB: `tests/learn/fixtures/sample-kb/`
- Artifacts asserted: `learn-goal.json`, `survey-result.json`, `feynman-artifact.json`, `grade-result.json`

## Tasks

- [x] Write `tests/learn/integration-basic-flow.sh`: set up a temp working directory, invoke `learn-goal` with a test concept, then `learn-survey`, then `feynman-loop` (Tier 0), then `learn-grade` in sequence; capture exit codes and artifact paths
- [x] Assert happy path: verify all four artifact files are produced — `learn-goal.json`, `survey-result.json`, `feynman-artifact.json`, `grade-result.json` — and that each parses as valid JSON with expected top-level keys
- [x] Create `tests/learn/fixtures/sample-kb/` with at least three markdown files covering a narrow technical concept (e.g. FSRS spacing algorithm basics) for use as a fixture KB in KB-path tests
- [x] Add KB-path test branch to `integration-basic-flow.sh`: run `learn-goal` with `--kb tests/learn/fixtures/sample-kb/` and assert that `learn-goal.json` reflects KB-sourced concepts
- [x] Add sycophancy-correction assertion: grep `grade-result.json` for a `sycophancy_check` key (or equivalent) and assert its value is not `null`; fail with a clear message if the field is absent
