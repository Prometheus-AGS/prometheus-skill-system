---
id: change-learn-027
title: CLAUDE.md update for learn domain
type: documentation
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-020
---

# change-learn-027 — CLAUDE.md learn domain section

## Summary

Add a `## Learn Domain` section to the canonical `CLAUDE.md` documenting the
four-layer architecture (Layer A substrate, Layer B ui-surface, Layer C skills,
Layer D KB adapters), the KB adapter pattern, the Tier 0/1/2 degradation
contract, and update the `skills/` directory structure diagram to include the
`learn/` subdirectory.

## Motivation

`CLAUDE.md` is the canonical engineering reference for all sibling repositories.
Without a learn domain section, contributors working in the prometheus stack
cannot understand the substrate architecture or degradation contract from the
primary reference document.

## Scope

- Updated file: `CLAUDE.md`
- New section: `## Learn Domain`
- Updated section: skills directory structure diagram
