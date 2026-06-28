---
id: change-learn-010
title: "feynman-loop skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-009
  - change-learn-005
  - change-learn-006
---

# change-learn-010: feynman-loop skill

## Problem

The Feynman technique requires structured explain → identify gap → re-learn →
re-explain cycles, with depth recursion and audience escalation. There is no
skill that orchestrates this cycle end-to-end against the learner model.

## Proposal

Implement `skills/learn/feynman-loop/SKILL.md` mapping the cycle to PMPO phases
(Spec/Plan/Execute/Reflect). The skill tracks recursion depth with a floor guard,
spawns child loops for gap concepts, escalates audience horizontally (novice →
peer → skeptic), gates closure on all 3 mastery criteria, and produces a
`feynman-artifact.json` per completed cycle.

## Outcome

The core learning loop that drives the phase-learn-feynman experience.
