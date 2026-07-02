---
id: change-learn-005
title: "learner-model Rust crate"
type: design
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-001
---

# change-learn-005: learner-model Rust crate

## Problem

There is no shared runtime for reading, writing, and evolving the learner model.
Without a crate, each skill duplicates JSON parsing, FSRS scheduling, and CRDT
merge logic.

## Proposal

Implement a `learner-model` Rust crate defining `LearnerModel`, `ConceptState`,
`FSRSCard`, and `GapRecord` types. Back state with automerge, provide a
`seed_from_survey()` cold-start path, integrate `fsrs-rs` for scheduling, and
expose a JSON RPC shell interface so skills can call it without Rust tooling.

## Outcome

A single authoritative crate that all learn-* skills call to read/write learner
state, eliminating duplication and ensuring CRDT-safe writes.

## Tasks

- [x] Define `LearnerModel`, `ConceptState`, `FSRSCard`, and `GapRecord` types matching the schema from change-learn-001
- [x] Implement automerge document backend using the `CrdtEngine` trait from change-learn-004b
- [x] Implement `seed_from_survey()` — cold-start from LLM-derived mastery priors when no prior data exists
- [x] Integrate `fsrs-rs` for card scheduling (due-date calculation, `next_states()`, stability/difficulty tracking)
- [x] Expose MCP-callable JSON RPC shell interface (`learner-model-cli`) for read/write/query from skill scripts
