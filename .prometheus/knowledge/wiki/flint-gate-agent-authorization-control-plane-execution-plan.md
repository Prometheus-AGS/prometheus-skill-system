---
type: Reference
id: flint-gate-agent-authorization-control-plane-execution-plan
title: Flint Gate Agent Authorization Control Plane Execution Plan
description: "Phase `agent-authz-control-plane` for project `flint-gate` is ready for execution."
tags:
- flint-gate
- agent-authorization
- mcp
- oauth-2-1
- policy-engine
- rate-limiting
- kbd-execution
sources:
- stdin
- manual:flint-gate/agent-authz-control-plane
timestamp: 2026-07-03T13:35:10.020302+00:00
created_at: 2026-07-03T13:35:10.020302+00:00
updated_at: 2026-07-03T13:35:10.020302+00:00
revision: 0
---

## Context

Phase `agent-authz-control-plane` for project `flint-gate` is ready for execution.

- KBD root: `/Users/gqadonis/Projects/prometheus/flint-gate`
- Captured: `2026-07-03T13:23:01Z`
- Backend: `openspec`
- Criteria profile: `effort-impact`
- Current state: `execution_ready`
- Progress: `0/8` changes complete

## Objective

Evolve `flint-gate` from an auth proxy into an MCP-era agent gateway by adding an agent-authorization control plane on top of existing streaming enforcement capabilities:

- Mid-stream SSE token metering
- Session watchdog
- AG-UI/A2UI processing

Scope is authorization-first. The phase explicitly excludes off-identity LLM-ops features such as:

- Semantic caching
- Multi-LLM routing
- Multimodal processing

Seed brief: `.kbd-orchestrator/evolution-briefs/ai-agent-gateway-parity.md`

## Build Goals

### 1. Budget enforcement and windowed rate limiting

Extend existing `usage_events` and lifetime `MaxTokenBudget` support into:

- Per-key and per-team token budgets
- Rolling-window budgets for minute/hour/day periods
- Request-rate limits
- Threshold blocking with clear errors

Rationale:

- Addresses Gap G3
- Fastest high-impact win
- Highest feasibility among planned changes

### 2. MCP OAuth 2.1 resource-server support

Implement MCP resource-server behavior required for gateway credibility:

- RFC 9728 protected-resource metadata
- `WWW-Authenticate: resource_metadata` on `401`
- RFC 8414 / OIDC authorization-server discovery
- PKCE S256 verification
- RFC 8707 `resource` / audience validation
- `403 insufficient_scope` step-up behavior
- Confused-deputy prevention by ensuring tokens are not passed through to upstreams

Rationale:

- Addresses Gap G1
- Critical credibility gate

### 3. Embedded policy engine and per-tool-call authorization

Evaluate and embed a native Rust policy engine inline in the stream:

- Candidate engines: Cedar core or `casbin-rs`
- No sidecar dependency
- Authorize each MCP tool call by:
  - Tool name
  - Parameters
  - Identity claims
- Filter unauthorized tools from `list_tools` responses, following the agentgateway pattern
- Add `PreRequestHook::Authorize`
- Add stream-level tool-call gate

Rationale:

- Addresses Gap G2
- Strategic authorization core

## Dispatch Contract

`execution.md` has been written and the phase state has moved to `execution_ready`.

The waypoint was refreshed, `execute:after` fired, and handoff was recorded.

Execution must be driven through KBD-aware commands rather than bare OpenSpec apply commands:

```text
/kbd-apply <change-id>
```

Do not use bare:

```text
/opsx:apply
```

Reason: bare `/opsx:apply` lacks KBD awareness and bypasses the orchestration seam this phase depends on.

## Change Dispatch Order

Dependency-respecting execution order:

1. `add-budget-rate-limiting`
2. `mcp-resource-server`
3. `policy-engine`
4. `per-tool-authz`
5. `authz-audit-trail`
6. `hitl-approval`
7. `guardrail-hook`
8. `web-config-ui`

Next action:

```text
/kbd-apply add-budget-rate-limiting
```

## Per-Change Gate

Each change is executed one task per turn with per-task KBD hooks and synchronization of:

- `progress.json`
- Waypoint state

Required gate sequence for each completed change:

1. All tasks done
2. Run `/refine-validate`
   - Artifact-refiner QA against `constraints.md`
3. If validation passes:
   - Run `/opsx:verify`
   - Run `/opsx:archive`
4. If validation fails:
   - Mark change `BLOCKED`
   - Run `/refine-code`

## Security Review Requirements

The following authorization-sensitive changes require an additional `security-reviewer` pass:

- `mcp-resource-server`
- `policy-engine`
- `per-tool-authz`
- `hitl-approval`

## Engineering Quality Bar

Every change must assert:

```text
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

Additional requirement:

- At least 80% coverage on new code

## Execution Notes

`/kbd-execute` only writes the execution contract. It does not implement code.

The phase contains substantial implementation work:

- 8 changes
- Approximately 56 tasks
- Rust backend work
- Cedar or Casbin policy-engine integration
- MCP OAuth 2.1 resource-server implementation
- Streaming per-tool authorization
- HITL approval flow
- React SPA work

Remaining work:

```text
implement all 8 changes (0/8 done) → reflect
```

# Citations

1. stdin
2. manual:flint-gate/agent-authz-control-plane