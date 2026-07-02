---
id: change-learn-017
title: learn-about-system skill (adoption entry point)
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-016
  - change-learn-007
  - change-learn-006
---

# change-learn-017 — learn-about-system skill

## Summary

Add a `learn-about-system` skill that serves as the zero-friction entry point
for operators discovering the skill pack and KBD lifecycle. When invoked with
no arguments it asks what the operator wants to learn and routes accordingly.
The `--area` flag accepts `kbd`, `skills`, or `harness` and skips elicitation.
The skill demonstrates the self-teaching loop: the skill pack teaches itself
using its own learning infrastructure.

## Motivation

New adopters have no clear starting point. This skill is the "front door" — it
surfaces the right corpus and routes to `learn-goal` or `learn-survey` without
requiring the operator to know which skill to invoke first.

## Scope

- New skill directory: `skills/learn/learn-about-system/`
- Three `--area` routing paths: `kbd`, `skills`, `harness`
- Interactive discovery mode (no args) using `AskUserQuestion`
- Documents self-teaching loop pattern

## Tasks

- [x] Write `skills/learn/learn-about-system/SKILL.md` with frontmatter, overview, `--area` flag documentation (`kbd` | `skills` | `harness`), and interactive mode description
- [x] Implement interactive discovery mode (no args): use `AskUserQuestion` to elicit the operator's interest area, then branch to the appropriate `--area` path
- [x] Implement `--area kbd` path: load `kbd-lifecycle-corpus.json` as the active KB, invoke `learn-goal` with that corpus, and emit a session start message explaining the self-teaching loop
- [x] Implement `--area skills` path: survey skill domains by reading `skills/` directory structure, surface the five most relevant skills for the operator's stated goal using a brief elicitation exchange
- [x] Document the self-teaching loop pattern in `skills/learn/learn-about-system/references/self-teaching-loop.md` (how the skill pack teaches itself using its own learning infrastructure, entry → goal → survey → feynman → grade → retain cycle)
