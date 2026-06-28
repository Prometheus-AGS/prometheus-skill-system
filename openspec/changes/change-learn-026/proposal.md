---
id: change-learn-026
title: docs/guide update for learn domain
type: documentation
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-020
---

# change-learn-026 — docs/guide learn domain chapter

## Summary

Write `docs/guide/10-learn-skills.md` as the authoritative operator guide for
the learn domain. Covers domain overview, per-skill documentation (purpose,
entry command, inputs, outputs, cross-harness behaviour), KB adapter guide
(ingest, use in `learn-goal`, privacy notes), and meta-learning guide
(KBD adoption path, self-teaching loop). Update `docs/guide/00-index.md` to
link the new chapter.

## Motivation

The `docs/guide/` directory is the primary operator reference. Without a learn
domain chapter, operators must read individual `SKILL.md` files to understand
how the skills compose into a learning session.

## Scope

- New file: `docs/guide/10-learn-skills.md`
- Updated file: `docs/guide/00-index.md`
