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

## Tasks

- [x] Write `docs/guide/10-learn-skills.md` domain overview section: explain the Feynman technique loop, the three surface tiers (Tier 0 text / Tier 1 AskUserQuestion / Tier 2 A2UI), the substrate layer (learner-model, storage-provider, surface-bridge, content-grounding), and the full skill list with one-line descriptions
- [x] Add per-skill sections to `docs/guide/10-learn-skills.md`: for each learn-domain skill document purpose, entry command with flag examples, primary inputs and outputs (artifact filenames and key JSON fields), and cross-harness behaviour notes
- [x] Add KB adapter guide subsection: explain the three adapter types (local files, Dify KB, URL scrape), show `learn-kb add` command examples for each, document how KB entries influence `learn-grade`, and note privacy considerations (no data leaves the local palace store without operator action)
- [x] Add meta-learning guide subsection: describe the KBD adoption path (learn-about-system → learn-goal → survey → feynman → grade → retain → practice → certify), explain the self-teaching loop concept, and provide a worked example of using the skill pack to learn the KBD lifecycle itself
- [x] Update `docs/guide/00-index.md`: add an entry for chapter 10 (`10-learn-skills.md`) in the table of contents with a one-line description
