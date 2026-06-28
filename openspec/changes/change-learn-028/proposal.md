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
