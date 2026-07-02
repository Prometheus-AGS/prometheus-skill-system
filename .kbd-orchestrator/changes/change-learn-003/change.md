---
id: change-learn-003
title: "content-grounding service + public corpus assembly"
type: design
status: DONE
phase: phase-learn-feynman
depends_on: []
---

# change-learn-003: content-grounding service + public corpus assembly

## Problem

Skills that teach need reliable, high-quality source material. Without a grounding
service, every skill invents its own ad-hoc retrieval strategy, leading to
inconsistent quality and missing misconception coverage.

## Proposal

Implement `content-grounding.sh` — a reusable shell service that assembles a
teaching corpus for a given subject by walking a prioritized source chain
(primary literature → textbooks → reference implementations → surveys →
secondary → LLM fill). Include an `--include-misconceptions` flag for
known-wrong-model retrieval.

## Outcome

A script callable by any learn-* skill to get a grounded corpus without
duplicating retrieval logic.

## Tasks

- [x] Write `shared/scripts/content-grounding.sh` with `--subject`, `--level`, `--budget-sources`, and `--budget-minutes` flags
- [x] Implement source priority chain: primary literature > textbooks > reference implementations > surveys > secondary > LLM fill
- [x] Add `--include-misconceptions` flag that retrieves known-wrong-model sources alongside correct ones
- [x] Define `grounding-corpus.schema.json` specifying output envelope (sources[], misconceptions[], metadata)
- [x] Test script with a sample subject and verify priority chain ordering and schema conformance
