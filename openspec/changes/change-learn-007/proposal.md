---
id: change-learn-007
title: "learn-goal skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-003
  - change-learn-004
  - change-learn-006
---

# change-learn-007: learn-goal skill

## Problem

Users starting a learning journey need to declare a goal, check its feasibility
given available resources, and assemble a grounded corpus — before any survey or
planning begins. Without a gated entry point, later skills receive under-specified
or unachievable goals.

## Proposal

Implement `skills/learn/learn-goal/SKILL.md` with a `/learn-goal` entry command.
The skill calls `content-grounding.sh` (and optionally `content-grounding-kb.sh`
via `--kb`) to assemble a corpus, then runs a feasibility gate emitting
RED/YELLOW/GREEN. The feasibility result passes through `sycophancy-correction`
before being shown to the user.

## Outcome

A gated entry point that validates learning goals and produces a verified corpus
before downstream skills run.
