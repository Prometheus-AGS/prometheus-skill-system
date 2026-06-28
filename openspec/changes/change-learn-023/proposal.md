---
id: change-learn-023
title: learn-kb integration test
type: test
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-015
  - change-learn-021
---

# change-learn-023 — learn-kb integration test

## Summary

Write an integration test covering the `learn-kb add` subcommand with a local
file fixture, asserting that ingested entries appear in `grounding-corpus.json`
with `source_type: operator_kb`. A second assertion runs `learn-grade` with the
KB corpus and verifies that KB-specific concepts appear in the transfer problem
set, confirming end-to-end KB influence on grading.

## Motivation

The KB adapter pipeline has three distinct stages (scrape/read → normalise →
ingest). Without integration coverage a silent failure in any stage produces
an empty corpus without a visible error.

## Scope

- New test: `tests/learn/integration-kb.sh`
- Fixture: `tests/learn/fixtures/sample-kb/` (reused from change-learn-021)
- Assertions: grounding-corpus entries, grade transfer problem content
