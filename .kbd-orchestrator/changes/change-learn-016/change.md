---
id: change-learn-016
title: Meta-grounding corpus for KBD lifecycle + skill pack
type: content artifact
status: DONE
phase: phase-learn-feynman
depends_on: []
---

# change-learn-016 — Meta-grounding corpus

## Summary

Produce two JSON corpora that ground the Feynman loop in Prometheus-specific
concepts: `kbd-lifecycle-corpus.json` covering KBD phases, OpenSpec, hooks,
waypoints, and progress signaling; and `skill-pack-corpus.json` covering skill
categories, distribution formats, validation rules, and imported submodules.
Both files include misconception entries so `learn-grade` can detect wrong
mental models.

## Motivation

Without first-party corpora the learning loop can only surface general
programming concepts. Operators adopting this skill pack need structured
grounding material so the Feynman loop can teach the pack itself.

## Scope

- New files under `docs/learn/meta-corpus/`
- No code changes; pure content artifact
- Validated against `grounding-corpus.schema.json`

## Tasks

- [x] Write `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` with concept entries for: assess, analyze, plan, execute, reflect, evolve, OpenSpec, hooks (PreToolUse/PostToolUse/Stop), waypoints (position-reminder.txt, current-waypoint.json, progress.json), and progress signaling format
- [x] Add misconception entries to `kbd-lifecycle-corpus.json` for common wrong mental models (e.g. treating reflect as a success summary, skipping progress signals, confusing phase vs. change)
- [x] Write `docs/learn/meta-corpus/skill-pack-corpus.json` with concept entries for: skill domains, SKILL.md frontmatter schema, dual-format (agentskills.io + Claude Code plugin), imported submodules, validate:strict vs. validate:skill, and install-skills-flat.sh platform targets
- [x] Add misconception entries to `skill-pack-corpus.json` for common wrong mental models (e.g. editing `.claude-plugin/` directly, confusing `name` field with directory name, using backslashes in paths)
- [x] Validate both corpora against `grounding-corpus.schema.json` (run `npm run validate:strict` or equivalent schema check; fix any schema violations)
