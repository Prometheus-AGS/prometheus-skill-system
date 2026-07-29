---
type: Reference
id: verify-and-publish-session-waiting-on-memory-server-build
title: Verify-and-Publish Session Waiting on Memory Server Build
tags:
- codex-plugin
- verify-and-publish
- surreal-memory-server
- cargo-build
- mcp-env
- plugin-hooks
- git-subdir
links:
- codex-plugin-verify-and-publish-phase-goals
- codex-verify-and-publish-build-feature-fix-status
sources:
- stdin
- manual:phase-codex-plugin-verify-and-publish
timestamp: 2026-07-15T23:46:34.792601+00:00
created_at: 2026-07-15T23:46:34.792601+00:00
updated_at: 2026-07-15T23:46:34.792601+00:00
revision: 0
---

## Context

- **Phase:** `phase-codex-plugin-verify-and-publish`
- **Project:** unspecified
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-15T23:36:36Z`
- **Source context:** `manual:phase-codex-plugin-verify-and-publish`

This session continues the work scoped in [Codex Plugin Verify-and-Publish Phase Goals](/codex-plugin-verify-and-publish-phase-goals.md), with status related to later verify-and-publish tracking records such as [Codex Verify-and-Publish Build Feature Fix Status](/codex-verify-and-publish-build-feature-fix-status.md).

## Active Goals

- **G-01 — Real GitHub Actions validation**
  - Exercise `validate:codex` in a real GitHub Actions run.
  - Confirm the CI drift/validity gate runs and passes on push or PR.
  - Addresses reflection Delta 3.
- **G-02 — MCP environment round-trip verification**
  - Run `codex-provision-mcp-env.sh` with keys set.
  - Install the plugin.
  - Verify `codex doctor` stops warning.
  - Verify a plugin MCP server sees its key.
  - Addresses Delta 2.
- **G-03 — Real Codex plugin hook execution**
  - Verify the real plugin hooks run cleanly under Codex with the `CLAUDE_PLUGIN_ROOT:-PLUGIN_ROOT` fix.
  - Confirm the verification is against real hooks, not only the probe.
  - Specifically ensure `SessionStart` executes without empty-path errors.
- **G-04 — Real remote `git-subdir` source resolution**
  - Test `codex plugin marketplace add <git-url>` against a real remote.
  - Confirm published `git-subdir` sources resolve correctly.
  - Represents the first external publish validation for Delta 3.

## Current Session Status

- Waiting for `surreal-memory-server` compilation to complete before continuing with skills installation.
- Background monitor task `bhd3oqls2` and build task `bet6qb0ct` are expected to notify on completion.
- Next action is deferred until the completion notification arrives.

# Citations

1. stdin
2. manual:phase-codex-plugin-verify-and-publish