---
id: change-learn-008
title: "learn-survey skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-001
  - change-learn-005
  - change-learn-006
  - change-learn-007
---

# change-learn-008: learn-survey skill

## Problem

Before planning a learning path, the system needs to assess the user's current
knowledge state. Without a diagnostic survey grounded in the teaching corpus,
the learner model starts with uninformed priors.

## Proposal

Implement `skills/learn/learn-survey/SKILL.md`. The skill generates diagnostic
items (conceptual, procedural, misconception probes) from the corpus assembled
by `learn-goal`, renders them via ui-surface (Tier 1 preferred), produces a
`survey-result.json` capturing `recursion_floor` and `mastery_priors`, and
writes the initial learner model seed via `change-learn-005`.

## Outcome

A diagnostic that calibrates the learner model before any curriculum or Feynman
loop begins.

## Tasks

- [x] Write `skills/learn/learn-survey/SKILL.md` with invocation contract, input (corpus from learn-goal), and output spec
- [x] Generate diagnostic items from corpus: conceptual questions, procedural tasks, and misconception probe prompts
- [x] Render items via ui-surface at Tier 1 (preferred); degrade to Tier 0 if unavailable
- [x] Produce `survey-result.json` with `recursion_floor` (minimum Feynman depth), `mastery_priors` per concept, and confidence intervals
- [x] Write `learner_model_seed` to the learner-model crate via the JSON RPC shell interface from change-learn-005
