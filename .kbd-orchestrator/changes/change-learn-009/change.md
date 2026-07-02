---
id: change-learn-009
title: "learn-grade skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-003
  - change-learn-004b
  - change-learn-005
  - change-learn-006
---

# change-learn-009: learn-grade skill

## Problem

Multiple skills (feynman-loop, learn-practice, learn-retain, learn-certify) need
to grade user responses against a grounded corpus. Without a shared grader, each
skill implements its own assessment logic, leading to inconsistent standards and
sycophantic evaluations.

## Proposal

Implement `skills/learn/learn-grade/SKILL.md`. The grader retrieves the concept
corpus via semantic search, checks the response for completeness, accuracy, and
misconceptions, runs the output through `sycophancy-correction` (S-02 pattern),
and generates novel transfer problems from the corpus — not from the user's
explanation — to probe genuine understanding.

## Outcome

A reusable, anti-sycophantic grading skill that all assessment-dependent skills
invoke as a sub-step.

## Tasks

- [x] Write `skills/learn/learn-grade/SKILL.md` with invocation contract (concept, explanation, corpus reference) and output schema
- [x] Implement corpus retrieval for the target concept via semantic search (surreal-memory palace or content-grounding)
- [x] Implement grader checks: completeness (all key ideas present), accuracy (no factual errors), misconceptions (known-wrong-model patterns flagged)
- [x] Run grader output through `sycophancy-correction` S-02 check before returning grade to caller
- [x] Generate novel transfer problems from corpus sources — explicitly not derived from the user's explanation — to test transfer
