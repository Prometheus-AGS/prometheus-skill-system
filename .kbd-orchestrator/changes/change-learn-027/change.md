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

## Tasks

- [x] Add `## Learn Domain` section to `CLAUDE.md`: introduce the domain purpose (Feynman-technique learning loop for operators), describe the four layers (Layer A: substrate crates, Layer B: ui-surface MCP, Layer C: learn-domain skills, Layer D: KB adapters), and list each crate/skill per layer
- [x] Document Layer A substrate crates in the new section: `surface-bridge` (Tier 2 Axum MCP server), `storage-provider` (FSRS card store + palace bridge), `learner-model` (FSRS scheduling binary), `content-grounding` (corpus validation)
- [x] Document Layer B (`ui-surface`) and list all Layer C skills with their entry commands and artifact contracts (input → output file names)
- [x] Document the KB adapter pattern: explain `palace_ingest`, Dify KB ID, and URL scrape paths; note that all KB data is local-first and the `source_type: operator_kb` tag is the federation boundary
- [x] Document the Tier 0/1/2 degradation contract: define each tier, state which substrate crates are required per tier, describe the fallback order (Tier 2 → Tier 1 → Tier 0), and update the `skills/` directory structure diagram to show `skills/learn/` with its skill subdirectories
