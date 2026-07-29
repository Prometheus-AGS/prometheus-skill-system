---
type: Reference
id: karpathy-mcp-gate-live-ingestion-verification
title: Karpathy MCP Gate Live Ingestion Verification
tags:
- mcp-ingestion
- pk-cherry
- repair-gate
- wiki-ingestion
- live-store-visibility
links:
- mcp-ingestion-required-for-running-pk-cherry-store-visibility
sources:
- repair-gate:mcp
timestamp: 2026-07-17T16:11:54.345325+00:00
created_at: 2026-07-17T16:11:54.345325+00:00
updated_at: 2026-07-17T16:11:54.345325+00:00
revision: 0
---

## Context

A knowledge entry tagged `karpathy-mcp-gate-2026-07-17-02` was ingested through the running PK MCP server to prove that the live MCP path can write and immediately read wiki entries.

## Verification

- **Path tested:** running PK MCP server ingestion path.
- **Purpose:** confirm live MCP-backed writes are immediately visible to the active knowledge store.
- **Result:** MCP ingestion path was used as the repair gate for write/read visibility.

## Operational Significance

This confirms the operational rule documented in [MCP Ingestion Required for Running pk-cherry Store Visibility](/mcp-ingestion-required-for-running-pk-cherry-store-visibility.md): when validating ingestion against a running `pk-cherry` store, use MCP-backed ingestion rather than assuming external CLI or filesystem writes will hot-reload into the active process.

# Citations

1. [1] repair-gate:mcp