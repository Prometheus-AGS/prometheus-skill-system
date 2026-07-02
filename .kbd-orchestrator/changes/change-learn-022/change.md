---
id: change-learn-022
title: learn-retain + learn-practice + learn-certify integration test
type: test
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-021
  - change-learn-012
  - change-learn-013
  - change-learn-014
---

# change-learn-022 — full loop integration test

## Summary

Write an integration test that runs the complete post-Feynman pipeline:
two `feynman-loop` iterations feed into `learn-retain` (FSRS card update),
then `learn-practice` (retrieval practice session), then `learn-certify`
with `--checkpoint` (verifiable credential emission). Tests assert FSRS card
mutation, practice-result artifact, and checkpoint VC structure including
`integrity_warning` on anomalous trajectories.

## Motivation

The retain → practice → certify pipeline is the long-tail path that most users
hit after the first session. Integration coverage here prevents silent regressions
in FSRS scheduling and VC emission.

## Scope

- New test: `tests/learn/integration-full-loop.sh`
- Builds on fixture KB and artifacts from change-learn-021
- Assertions cover FSRS card state, practice-result.json, and JSON-LD VC

## Tasks

- [x] Write `tests/learn/integration-full-loop.sh`: reuse the fixture KB from change-learn-021, run `feynman-loop` twice for two distinct concepts, then invoke `learn-retain` with both `feynman-artifact.json` outputs and capture the FSRS card store path
- [x] Assert FSRS card update: after `learn-retain` completes, read the FSRS card store JSON and verify that both concepts have a card entry with non-null `due`, `stability`, and `difficulty` fields
- [x] Invoke `learn-practice` with the updated card store and assert that `practice-result.json` is produced with at least one question-answer pair and a `score` field
- [x] Invoke `learn-certify --checkpoint` and assert that a checkpoint VC is emitted as valid JSON-LD: check `@context`, `type`, `credentialSubject.concepts`, and `proof` top-level keys
- [x] Add anomalous-trajectory branch: artificially set an implausibly high score in `practice-result.json`, re-run `learn-certify --checkpoint`, and assert that the VC contains `integrity_warning: true`
