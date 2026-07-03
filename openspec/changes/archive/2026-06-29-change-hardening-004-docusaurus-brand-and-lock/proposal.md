---
id: change-hardening-004-docusaurus-brand-and-lock
title: Docusaurus KnowMe brand tokens and package lock
phase: phase-sovereign-sync-hardening
priority: MEDIUM
effort: S
agent: codex
status: planned
scope:
  - docs
  - website
  - package-lock.json
---

# change-hardening-004 — Docusaurus KnowMe brand tokens and package lock

## Context

Reflection identified two docs-site hardening gaps: the Docusaurus theme still uses generic purple styling, and package installation is not reproducible because package versions or lockfiles are not pinned.

## Scope

- Find the Docusaurus package in this repository.
- Apply KnowMe brand tokens: Ember `#E04E28` / `#FF6A3D`, Conviction mark usage where available, and the project font stack.
- Pin package versions and commit `package-lock.json`.
- Run the docs build or validation command.

## Non-Goals

- No marketing rewrite.
- No unrelated content migration.
- No redesign into a landing page.

## Validation

- Docusaurus package install/build succeeds.
- The committed CSS no longer reads as the generic purple default.
