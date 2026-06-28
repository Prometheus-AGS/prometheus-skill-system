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
