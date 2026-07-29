---
type: Reference
id: codex-plugin-verify-and-publish-phase-goals
title: Codex Plugin Verify-and-Publish Phase Goals
tags:
- codex-plugin
- verify-and-publish
- github-actions
- mcp-env
- plugin-hooks
- git-subdir
- skill-pack
links:
- codex-plugin-verify-and-publish-executor-completion
sources:
- stdin
timestamp: 2026-07-13T01:32:07.892180+00:00
created_at: 2026-07-13T01:32:07.892180+00:00
updated_at: 2026-07-13T01:32:07.892180+00:00
revision: 0
---

## Context

- **Phase:** `phase-codex-plugin-verify-and-publish`
- **Project:** unspecified
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-13T01:30:29Z`
- **Source context:** `manual:phase-codex-plugin-verify-and-publish`

This phase precedes or complements the status tracked in [Codex Plugin Verify-and-Publish Executor Completion](/codex-plugin-verify-and-publish-executor-completion.md).

## Goals

- **G-01 — Real GitHub Actions validation**
  - Exercise `validate:codex` in an actual GitHub Actions run.
  - Confirm the CI drift/validity gate runs and passes on push or PR.
  - Addresses reflection Delta 3.

- **G-02 — MCP environment round-trip verification**
  - Run `codex-provision-mcp-env.sh` with keys set.
  - Install the plugin.
  - Verify `codex doctor` stops warning.
  - Verify a plugin MCP server can see its key.
  - Addresses Delta 2.

- **G-03 — Real Codex plugin hook execution**
  - Verify the real plugin hooks run cleanly under Codex, not only the probe.
  - Confirm the `CLAUDE_PLUGIN_ROOT:-PLUGIN_ROOT` fix prevents empty-path errors.
  - Specifically verify `SessionStart` executes without empty-path failures.

- **G-04 — Remote `git-subdir` source resolution**
  - Test `git-subdir` source resolution against a real remote.
  - Run `codex plugin marketplace add <git-url>` and verify published `git-subdir` sources resolve.
  - Intended as the first external publish verification.
  - Addresses Delta 3.

## Session Status

Three parallel agents were running at capture time:

- `surreal-memory-server` rebuild
- `liter-llm` rebuild
- Full skill pack installation

A follow-up report was expected after those agents completed.

# Citations

1. [1] stdin