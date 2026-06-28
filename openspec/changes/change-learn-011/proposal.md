---
id: change-learn-011
title: "learn-plan skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-008
  - change-learn-005
  - change-learn-006
---

# change-learn-011: learn-plan skill

## Problem

After a diagnostic survey, the user needs a sequenced curriculum that respects
concept prerequisites and adapts to their current mastery. Without automated
planning, users must manually sequence their learning, risking prerequisite
violations and inefficient ordering.

## Proposal

Implement `skills/learn/learn-plan/SKILL.md`. The skill queries the concept DAG
from surreal-memory, produces a `curriculum.json` with prerequisite-gated ordered
phases, supports a `--replan` mode triggered when mastery diverges more than 0.2
from the plan, and renders the DAG via ui-surface (Tier 0 list, Tier 2 mindmap
when available).

## Outcome

An adaptive curriculum planner that sequences learning phases based on live
mastery data and concept prerequisites.
