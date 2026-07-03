---
type: Reference
id: flint-gate-agent-authz-budget-rate-limiting-progress
title: Flint Gate Agent Authz Budget Rate Limiting Progress
tags:
- flint-gate
- agent-authorization
- rate-limiting
- budget-enforcement
- mcp
- oauth-2-1
- rust
links:
- flint-gate-agent-authorization-control-plane-execution-plan
sources:
- stdin
- manual:flint-gate/agent-authz-control-plane
timestamp: 2026-07-03T14:46:44.518735+00:00
created_at: 2026-07-03T14:46:44.518735+00:00
updated_at: 2026-07-03T14:46:44.518735+00:00
revision: 0
---

## Context

- **Project:** `flint-gate`
- **Phase:** `agent-authz-control-plane`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-gate`
- **Captured:** `2026-07-03T14:41:04Z`
- **Source phase record:** `manual:flint-gate/agent-authz-control-plane`
- **Branch:** `feat/agent-authz-budget-rate-limiting`
- **Status:** `execution_ready`
- **Progress:** phase changes `0/8`; change 1 is `6/8` tasks complete

This is an execution update for the [Flint Gate Agent Authorization Control Plane Execution Plan](/flint-gate-agent-authorization-control-plane-execution-plan.md).

## Phase Objective

Evolve `flint-gate` from an auth proxy into an MCP-era agent gateway by adding an agent-authorization control plane on top of existing streaming enforcement:

- Mid-stream SSE token metering
- Session watchdog
- AG-UI/A2UI processing

Scope remains authorization-first. The phase explicitly excludes off-identity LLM-ops features:

- Semantic caching
- Multi-LLM routing
- Multimodal processing

Seed brief: `.kbd-orchestrator/evolution-briefs/ai-agent-gateway-parity.md`

## Build Goals

### 1. Budget enforcement and windowed rate limiting

Extend existing `usage_events` and lifetime `MaxTokenBudget` hook into:

- Per-key and per-team rolling-window token budgets
- Minute/hour/day budget windows
- Request-rate limits
- Threshold blocking with clear errors

Priority rationale:

- Addresses Gap G3
- Fastest win
- Highest feasibility

### 2. MCP OAuth 2.1 resource-server support

Required capabilities:

- RFC 9728 protected-resource metadata
- `WWW-Authenticate: resource_metadata` on `401`
- RFC 8414 / OIDC authorization-server discovery
- PKCE S256 verification
- RFC 8707 `resource` / audience validation
- `403 insufficient_scope` step-up
- No token passthrough to upstreams, preventing confused-deputy risk

Priority rationale:

- Addresses Gap G1
- Critical credibility gate

### 3. Embedded policy engine and per-tool-call authorization

Planned capabilities:

- Embedded native-Rust policy engine; options include Cedar core or `casbin-rs`
- No sidecar policy service
- Inline stream authorization
- Authorize each MCP tool call by:
  - Tool name
  - Parameters
  - Identity claims
- Filter unauthorized tools from `list_tools` responses following the agentgateway pattern
- Add `PreRequestHook::Authorize`
- Add stream-level tool-call gate

Priority rationale:

- Addresses Gap G2
- Critical strategic core

## Change 1 Status: Budget and Rate Limiting

Change `add-budget-rate-limiting` is in progress with 6 of 8 tasks complete.

### Completed

- Added and locked rate-limiting dependencies:
  - `governor v0.10.4`
  - `tower_governor v0.8.0`
- Implemented and independently verified tasks 2–6:
  - `BudgetWindow` and `BudgetScope` config
  - Backward-compatible serde defaults
  - `tower_governor` in-process rate layer
  - Credential key extractor
  - New `ratelimit/` module
  - Redis Lua window counters
  - Reuse of the existing Redis connection manager; no new Redis dependency
  - Pipeline enforcement blocks with `429 quota_exceeded`
  - Postgres windowed fallback when Redis is disabled

### Verification Completed

- `clippy` clean with `--all-features`
- `clippy` clean with `--no-default-features`
- Workspace compiles on both paths:
  - Redis enabled
  - Redis disabled

### In Progress

Task 7: tests are being written by a Rust-specialist agent.

Planned/active test coverage:

- Config backward compatibility
- Budget pass/block behavior
- Credential key extractor
- Window-to-interval mapping
- Live Redis/database tests gated with `#[ignore]`

### Remaining for Change 1

1. Verify returned tests pass.
2. Close task 7.
3. Run task 8:
   - `cargo check --workspace`
   - `cargo clippy --workspace`
   - `cargo test --workspace`
4. Run artifact-refiner QA gate.
5. Run `/opsx:verify`.
6. Run `/opsx:archive`.

## Design Note

Windowed budget counters currently accumulate on the streaming path only. This matches existing lifetime-budget behavior because the non-streaming branch was not metered previously. This is a known limitation, not a regression.

## Next Phase Work

After completing change 1, continue with remaining changes 2–8 and then run reflection.

# Citations

1. [1] stdin
2. [2] manual:flint-gate/agent-authz-control-plane