---
id: change-learn-012
title: "learn-retain skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-005
  - change-learn-009
  - change-learn-006
---

# change-learn-012: learn-retain skill

## Problem

Knowledge acquired through feynman-loop and learn-practice decays without spaced
repetition. There is no skill that reads the FSRS due queue and surfaces review
prompts at the optimal moment.

## Proposal

Implement `skills/learn/learn-retain/SKILL.md`. The skill reads the FSRS due
queue from the learner-model crate, surfaces review prompts via ui-surface,
grades the user's response via `learn-grade` at a ≥ 0.6 retention threshold,
and updates the `FSRSCard` via `fsrs-rs next_states()`.

## Outcome

A spaced-repetition review skill that keeps acquired knowledge above the
retention threshold using the FSRS scheduler embedded in the learner-model crate.

## Tasks

- [x] Write `skills/learn/learn-retain/SKILL.md` with invocation contract, FSRS queue read protocol, and session exit criteria
- [x] Read FSRS due queue from learner-model via JSON RPC CLI (`learner-model-cli due-queue --today`)
- [x] Surface review prompts for each due card via ui-surface (Tier 1 preferred, Tier 0 fallback)
- [x] Grade each review response via `learn-grade` with retention threshold ≥ 0.6 as passing bar
- [x] Update `FSRSCard` in learner-model via `fsrs-rs next_states()` for correct and incorrect outcomes
