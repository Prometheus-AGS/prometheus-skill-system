---
id: change-learn-001
title: "Spike: learner-model schema + CRDT conflict semantics"
type: design
status: DONE
phase: phase-learn-feynman
depends_on: []
---

# change-learn-001: Spike — learner-model schema + CRDT conflict semantics

## Problem

The learner-model crate has no agreed schema. Without a canonical JSON shape and a
documented merge strategy, multiple agents writing concurrently will produce
divergent or corrupted state.

## Proposal

Produce a design spike that fully specifies the `learner_model_seed` JSON schema
and the field-level CRDT merge semantics for every field in that schema. Include
worked conflict examples and the PFA update rule for observations ≥ 5.

## Outcome

A `learner-model-schema.json` and a `conflict-semantics.md` that the
`change-learn-005` crate implementation can follow without ambiguity.
