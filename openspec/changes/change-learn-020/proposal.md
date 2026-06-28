---
id: change-learn-020
title: skills/learn domain directory + validation + docs scaffolding
type: infrastructure
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-006
---

# change-learn-020 — skills/learn domain scaffolding

## Summary

Create the `skills/learn/` domain directory and wire it into every infrastructure
touch-point: README, marketplace manifest, plugin.json skill category list,
validation scope, and the docs table of contents. This change is the prerequisite
gate for all learn-domain skills landing in subsequent changes.

## Motivation

Skills cannot be installed, validated, or distributed until their domain
directory exists and is registered in all manifests. Doing this as a dedicated
change keeps the diff small and reviewable before any skill content lands.

## Scope

- New directory: `skills/learn/` with `README.md`
- `marketplace/marketplace.json` — add `learn` domain entry
- `.claude-plugin/plugin.json` — add `learn` skill category
- `npm run validate:strict` scope — include `skills/learn/`
- Top-level `README.md` skills table — add `learn` row
