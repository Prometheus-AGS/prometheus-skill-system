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

## Tasks

- [x] Create `skills/learn/` directory and write `skills/learn/README.md` with domain overview, skill list table (name, purpose, entry command), and a skill dependency diagram (text-based or Mermaid) showing the learn loop flow
- [x] Add `learn` domain entry to `marketplace/marketplace.json` (domain name, description, skill count placeholder, tags: `["learning", "feynman", "spaced-repetition", "knowledge-base"]`)
- [x] Add `learn` to the skill category list in `.claude-plugin/plugin.json` so the plugin marketplace surfaces learn-domain skills
- [x] Update `scripts/validate-skills.js` (or the npm validate script config) to include `skills/learn/` in the `validate:strict` sweep — confirm `npm run validate:strict` exits 0 on the empty domain directory
- [x] Add `learn` row to the skills table in the top-level `README.md` (columns: domain, description, example skills, install path)
