---
id: change-learn-028
title: v1.4.0 release bump + changelog
type: release
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-021
  - change-learn-022
  - change-learn-023
  - change-learn-024
  - change-learn-025
  - change-learn-026
  - change-learn-027
---

# change-learn-028 — v1.4.0 release

## Summary

Bump the package version to `1.4.0` across `package.json` and
`.claude-plugin/plugin.json`, update `marketplace/marketplace.json` to `1.4.0`
with learn domain tags, write the `CHANGELOG.md` v1.4.0 entry covering the
learn domain, KB adapter, meta-learning skills, and substrate crates, and
run final validation and build.

## Motivation

All phase-learn-feynman changes are gated by integration tests (021–024) and
infrastructure wiring (025–027). This release change is the merge commit that
ships the domain publicly.

## Scope

- `package.json` — version `1.4.0`
- `.claude-plugin/plugin.json` — version `1.4.0`
- `marketplace/marketplace.json` — version `1.4.0` + learn tags
- `CHANGELOG.md` — new v1.4.0 section

## Tasks

- [x] Bump version to `1.4.0` in `package.json` and `.claude-plugin/plugin.json` (update the `"version"` field in both files; confirm they match)
- [x] Update `marketplace/marketplace.json`: set version to `1.4.0` and add learn domain tags (`"learning"`, `"feynman"`, `"spaced-repetition"`, `"knowledge-base"`, `"meta-learning"`) to the top-level tags array and the learn domain entry
- [x] Write `CHANGELOG.md` v1.4.0 entry with sections: `### Learn Domain` (list all learn-domain skills with one-line descriptions), `### KB Adapter` (local, Dify, URL scrape), `### Meta-Learning Skills` (learn-about-system, learn-harness), `### Substrate Crates` (surface-bridge, storage-provider, learner-model)
- [x] Verify `npm run validate:strict` passes for all learn skills: run the command, confirm exit 0, and fix any validation errors before marking this task complete
- [x] Run `npm run build` to rebuild `.claude-plugin/` symlinks and confirm no broken links; then do a final `git status` check to confirm all expected files are staged and no unintended files are included
