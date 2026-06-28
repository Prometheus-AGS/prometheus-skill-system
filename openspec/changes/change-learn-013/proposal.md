---
id: change-learn-013
title: "learn-practice skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-009
  - change-learn-005
  - change-learn-006
---

# change-learn-013: learn-practice skill

## Problem

Explaining a concept is not sufficient for mastery. Users need deliberate practice
with varied problem types at an appropriate difficulty level, with interleaving to
prevent illusion-of-knowing effects.

## Proposal

Implement `skills/learn/learn-practice/SKILL.md` with `--type` flag supporting
`derivation`, `implementation`, and `transfer` modes. Difficulty is gated on
mastery > 0.6 for harder problems. Problem types rotate in an interleaved
schedule. Responses are graded via `learn-grade`, and results update the
learner-model.

## Outcome

A deliberate practice skill that deepens mastery beyond explanation through
varied, difficulty-gated, interleaved problem sets.
