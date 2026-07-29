---
type: Reference
id: codex-verify-and-publish-build-feature-fix-status
title: Codex Verify-and-Publish Build Feature Fix Status
tags:
- codex-plugin
- verify-and-publish
- local-embeddings
- apple-silicon
- cargo-build
- plugin-hooks
- mcp-env
links:
- codex-plugin-verify-and-publish-phase-goals
- codex-verify-and-publish-phase-sync-status
- codex-plugin-verify-and-publish-phase-final-push-summary
sources:
- stdin
- manual:phase-codex-plugin-verify-and-publish
timestamp: 2026-07-13T11:37:41.049319+00:00
created_at: 2026-07-13T11:37:41.049319+00:00
updated_at: 2026-07-13T11:37:41.049319+00:00
revision: 0
---

## Context

- **Phase:** `phase-codex-plugin-verify-and-publish`
- **Project:** unspecified
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-13T11:26:53Z`
- **Source context:** `manual:phase-codex-plugin-verify-and-publish`

This update follows the phase goals tracked in [Codex Plugin Verify-and-Publish Phase Goals](/codex-plugin-verify-and-publish-phase-goals.md) and later sync/final-push records such as [Codex Verify-and-Publish Phase Sync Status](/codex-verify-and-publish-phase-sync-status.md) and [Codex Plugin Verify-and-Publish Phase Final Push Summary](/codex-plugin-verify-and-publish-phase-final-push-summary.md).

## Active Phase Goals

- **G-01 — GitHub Actions validation**
  - Exercise `validate:codex` in a real GitHub Actions run.
  - Confirm the CI drift/validity gate runs and passes on push or PR.
  - Addresses reflection Delta 3.
- **G-02 — MCP environment round-trip**
  - Run `codex-provision-mcp-env.sh` with keys set.
  - Install the plugin.
  - Verify `codex doctor` stops warning.
  - Verify a plugin MCP server sees its key.
  - Addresses Delta 2.
- **G-03 — Real plugin hook verification**
  - Verify real plugin hooks run cleanly under Codex with the `CLAUDE_PLUGIN_ROOT:-PLUGIN_ROOT` fix.
  - Confirm `SessionStart` executes without empty-path errors.
- **G-04 — Git subdirectory source resolution**
  - Test `codex plugin marketplace add <git-url>` against a real remote.
  - Confirm published `git-subdir` sources resolve externally.
  - First external publish target; addresses Delta 3.

## Current Finding

The build issue was traced to an incorrect release build invocation:

```bash
cargo build --release
```

That command skips `local-embeddings`. For macOS Apple Silicon, the correct build must enable the required feature set:

```bash
cargo build --release --features embedded,metal,local-embeddings
```

## Current Status

- Rebuild with `embedded`, `metal`, and `local-embeddings` features is in progress.
- Expected compile duration: approximately 5–8 minutes.
- Still waiting on the previously launched `install-binaries.sh` agent.

## Next Step

After the build and installer agent complete, start the next phase with:

```text
/kbd-new-phase <next-phase-name>
```

# Citations

1. stdin
2. manual:phase-codex-plugin-verify-and-publish