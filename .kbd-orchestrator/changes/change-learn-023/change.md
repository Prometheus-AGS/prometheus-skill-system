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

## Tasks

- [x] Write `tests/learn/integration-kb.sh`: invoke `learn-kb add --local tests/learn/fixtures/sample-kb/` in a clean temp directory, capture exit code and stdout, assert exit 0
- [x] Assert corpus entries: read the output `grounding-corpus.json` (or the palace store index) and verify at least one entry has `source_type: "operator_kb"` and a non-empty `content` field
- [x] Extend `tests/learn/fixtures/sample-kb/` if needed: ensure the fixture contains at least one markdown file with a clearly identifiable concept term that can be detected in downstream grade output
- [x] Add grade integration assertion: run `learn-grade` with the KB corpus path and a mock `feynman-artifact.json`; assert that `grade-result.json` transfer problems reference at least one concept term from the fixture KB
- [x] Add negative test: invoke `learn-kb add --local /nonexistent/path` and assert exit code is non-zero with a human-readable error message (no stack trace leakage)
