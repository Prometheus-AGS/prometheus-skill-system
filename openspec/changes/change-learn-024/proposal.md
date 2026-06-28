---
id: change-learn-024
title: learn-about-system + learn-harness integration test
type: test
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-016
  - change-learn-017
  - change-learn-018
---

# change-learn-024 — meta-skill integration test

## Summary

Write an integration test covering `learn-about-system` and `learn-harness`.
Asserts that `--area kbd` routes to `learn-goal` with `kbd-lifecycle-corpus.json`
as the active KB, that `--harness claude-code` produces a capability map
output, and that both corpus files pass schema validation with expected concept
IDs and skill category entries.

## Motivation

`learn-about-system` and `learn-harness` are routing skills — their primary
contract is correct dispatch and corpus selection. Integration tests here catch
routing regressions without requiring a full Feynman loop run.

## Scope

- New test: `tests/learn/integration-meta.sh`
- Validates routing, capability map output, and corpus schema
- No Feynman loop execution required in this test
