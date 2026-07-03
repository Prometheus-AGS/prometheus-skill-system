---
id: change-hardening-003-mcp-client-pool-e2e
title: McpClientPool end-to-end forwarding test
phase: phase-sovereign-sync-hardening
priority: MEDIUM
effort: M
agent: codex
status: planned
scope:
  - substrate/sovereign-sync/src
  - substrate/sovereign-sync/tests
---

# change-hardening-003 — McpClientPool end-to-end forwarding test

## Context

Reflection identified that the MCP client pool is implemented but not tested end-to-end. Unit tests are not enough for this boundary because the risk is transport, upstream process lifecycle, allowed-tools filtering, and error propagation.

## Scope

- Add an integration fixture using a controlled local MCP server process or deterministic in-process transport.
- Verify `call_tool` forwarding works on the happy path.
- Verify allowed-tools filtering blocks disallowed upstream tools.
- Verify upstream error or process exit is surfaced cleanly.

## Non-Goals

- No production MCP server redesign.
- No network-dependent external MCP service.
- No broad compatibility matrix for every MCP host.

## Validation

- `cargo test` in `substrate/sovereign-sync`
- CI job from change-hardening-002 can run the new test deterministically.
