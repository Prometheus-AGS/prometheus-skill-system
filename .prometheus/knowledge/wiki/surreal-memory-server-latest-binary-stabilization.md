---
type: Reference
id: surreal-memory-server-latest-binary-stabilization
title: surreal-memory-server Latest Binary Stabilization
description: "During phase `phase-okf-llm-wiki-adoption`, `surreal-memory-server` was verified and stabilized so it no longer blocks remaining repository rebuilds. The phase goals align with [OKF LLM Wiki Adoption Phase Completion Status](/okf-llm-wiki-adoption-phase-completion-status.md): OKF"
tags:
- surreal-memory-server
- surrealdb
- mcp
- launchd
- okf
- build-stabilization
links:
- okf-llm-wiki-adoption-phase-completion-status
sources:
- stdin
timestamp: 2026-07-03T13:42:47.345543+00:00
created_at: 2026-07-03T13:42:47.345543+00:00
updated_at: 2026-07-03T13:42:47.345543+00:00
revision: 0
---

## Context

During phase `phase-okf-llm-wiki-adoption`, `surreal-memory-server` was verified and stabilized so it no longer blocks remaining repository rebuilds. The phase goals align with [OKF LLM Wiki Adoption Phase Completion Status](/okf-llm-wiki-adoption-phase-completion-status.md): OKF v0.1 wiki conformance, root `index.md`/`log.md`, body-based cross-links, citations, first-class LLM Wiki operations, and permissive linting.

## Stabilization Outcome

`surreal-memory-server` is running the latest expected binary and is stable.

- **Binary source:** freshly built from submodule commit `b2ed891` from Jul 3 08:27.
- **Deployment paths:**
  - `/usr/local/bin`
  - `~/.local/bin`
- **Signing:** deployed binaries were ad-hoc re-signed.
- **Health check:** `/health` returned `10/10` HTTP `200` responses under a rapid burst.
- **MCP SSE endpoint:** `/mcp/sse` is live and serving MCP sessions.
- **launchd state:**
  - `runs=1`
  - `state=running`
  - stable single start; run count is not climbing.
  - configured with `RunAtLoad` and `KeepAlive`.
- **Dependency:** SurrealDB is up on port `:28000`.

## Root Cause

Earlier `/health` flakiness was not caused by a missing route. The route exists at:

```text
src/api/mod.rs:103
```

The observed instability was caused by CPU starvation from concurrent builds. Once build pressure cleared, the service became stable.

## Verification Notes

The running binary is definitively the latest `b2ed891` code. The build artifact and deployed copy differ only by signature bytes.

## Remaining Follow-up

With `surreal-memory-server` unblocked, remaining rebuilds should be completed sequentially on the now-idle machine:

1. `liter-llm` v1.9.2
2. `pk`
3. `sycophancy`

After rebuilds, commit the following on a branch:

- `tools/liter-llm` fast-forward pointer
- hooks fixes
- opencode fixes
- installer fixes

# Citations

1. stdin