---
id: change-learn-006
title: "ui-surface skill"
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-002
---

# change-learn-006: ui-surface skill

## Problem

Learn skills need to surface questions, prompts, and feedback to the user, but
the available UI mechanism varies by harness tier. Without a shared abstraction,
each skill reimplements tier detection and rendering.

## Proposal

Implement `skills/learn/ui-surface/SKILL.md` with three rendering tiers driven
by `detect-surface-tier.sh` from `change-learn-002`. Tier 0 uses markdown and
checklists. Tier 1 uses `AskUserQuestion` on Claude Code and a file-pair
convention on other harnesses. Tier 2 is stubbed (Axum server not yet shipped).
The degradation rule prevents any skill from blocking on an unavailable tier.

## Outcome

A shared ui-surface skill that all other learn-* skills invoke, ensuring
consistent rendering and graceful degradation across all harnesses.

## Tasks

- [x] Write `skills/learn/ui-surface/SKILL.md` documenting the three-tier rendering model and invocation contract
- [x] Implement Tier 0: markdown/checklist rendering (works on all harnesses, no tool dependency)
- [x] Implement Tier 1: `AskUserQuestion` for Claude Code and `.ui-question` / `.ui-answer` file-pair convention for other harnesses
- [x] Add Tier 2 stub with a clear comment that it requires the surface-bridge Axum server (not yet shipped)
- [x] Document the degradation rule: never block on `preferred_tier`; always fall back to the next available tier
