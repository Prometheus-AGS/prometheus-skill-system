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
