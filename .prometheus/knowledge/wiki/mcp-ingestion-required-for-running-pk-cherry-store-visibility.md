---
type: Reference
id: mcp-ingestion-required-for-running-pk-cherry-store-visibility
title: MCP Ingestion Required for Running pk-cherry Store Visibility
tags:
- mcp-ingestion
- pk-cherry
- wiki-ingestion
- hot-reload
- installation-repair
sources:
- installation-repair-mcp-gate
timestamp: 2026-07-17T16:03:40.755453+00:00
created_at: 2026-07-17T16:03:40.755453+00:00
updated_at: 2026-07-17T16:03:40.755453+00:00
revision: 0
---

## Summary

Verified the repair path for wiki ingestion when using a running `pk-cherry` store: MCP-backed ingestion is required for immediate visibility across the active store process.

## Finding

- **Delta:** MCP-backed wiki ingestion was verified.
- **Root cause:** Separate CLI writes are not hot-reloaded into an already-running `pk-cherry` store.
- **Impact:** Content written by an external CLI process may not be visible to the currently running store until the store is refreshed.

## Corrective Actions

- Prefer **MCP ingest** for wiki writes that must be visible to the active `pk-cherry` process.
- If using separate CLI writes, perform a **restart or reload** before expecting cross-process visibility.

## Operational Rule

For installation or repair workflows that validate wiki ingestion through a live store, do not assume filesystem-level or CLI-level writes are immediately reflected in the running store. Use the MCP ingestion path or explicitly refresh the store lifecycle.

# Citations

1. installation-repair-mcp-gate