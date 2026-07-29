---
type: Reference
id: global-skill-repair-learning-gate
title: Global Skill Repair Learning Gate
tags:
- skill-repair
- learning-gate
- installation
- knowledge-base
- feynman-learning
links:
- deep-research-feynman-integration
sources:
- installation-repair-gate
timestamp: 2026-07-17T14:17:13.740637+00:00
created_at: 2026-07-17T14:17:13.740637+00:00
updated_at: 2026-07-17T14:17:13.740637+00:00
revision: 0
---

## Purpose

Defines a global learning gate for skill installation/repair work. The gate records repair outcomes as reusable knowledge instead of treating installation failures as one-off fixes.

## Tag

- `2026-07-17-karpathy-learning-gate`

## Operating Principle

- Skill repair should include a learning checkpoint after the fix is identified.
- The checkpoint should capture:
  - the failing installation or repair condition,
  - the root cause,
  - the applied fix,
  - the reusable prevention rule or diagnostic heuristic.
- This aligns with the explain→grade→gap→recurse quality-gate pattern described in [Deep Research + Feynman Learning Integration](/deep-research-feynman-integration.md): a repair is not complete until the system can explain the failure mode and preserve the lesson.

## Knowledge-Base Implication

Installation and repair incidents should be converted into concise wiki entries or update existing entries when they reveal durable engineering knowledge about skill packaging, installation flows, platform assumptions, or CLI behavior.

# Citations

1. [1] installation-repair-gate