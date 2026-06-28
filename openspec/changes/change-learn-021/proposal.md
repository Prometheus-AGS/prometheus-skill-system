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
