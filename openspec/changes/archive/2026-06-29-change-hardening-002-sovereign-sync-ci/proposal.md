---
id: change-hardening-002-sovereign-sync-ci
title: Sovereign-sync Rust CI
phase: phase-sovereign-sync-hardening
priority: HIGH
effort: S
agent: codex
status: planned
scope:
  - .github/workflows
  - substrate/storage-provider
  - substrate/sovereign-sync
---

# change-hardening-002 — Sovereign-sync Rust CI

## Context

The previous phase produced real Rust substrate crates, but reflection identified no CI job protecting the sovereign-sync path. The immediate hardening need is a repeatable GitHub Actions workflow that catches formatting, lint, and regression failures.

## Scope

- Add or extend a GitHub Actions workflow for the sovereign-sync substrate crates.
- Run format, clippy, and tests on stable Rust.
- Include `substrate/storage-provider`, `substrate/sovereign-sync`, and required dependency closure.
- Ensure generated build artifacts are not staged or committed.

## Non-Goals

- No release packaging.
- No deployment workflow.
- No credentials or secret provisioning.

## Validation

- Workflow YAML parses.
- Equivalent local commands run successfully or documented if a dependency blocks them.
