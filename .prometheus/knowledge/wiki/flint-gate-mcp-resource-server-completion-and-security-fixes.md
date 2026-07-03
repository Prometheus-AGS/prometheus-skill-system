---
type: Reference
id: flint-gate-mcp-resource-server-completion-and-security-fixes
title: Flint Gate MCP Resource Server Completion and Security Fixes
tags:
- flint-gate
- mcp
- oauth-2-1
- agent-authorization
- jwks
- confused-deputy
- rust
links:
- flint-gate-agent-authorization-control-plane-execution-plan
- flint-gate-agent-authz-budget-rate-limiting-progress
sources:
- stdin
- manual:flint-gate/agent-authz-control-plane
timestamp: 2026-07-03T17:21:02.587439+00:00
created_at: 2026-07-03T17:21:02.587439+00:00
updated_at: 2026-07-03T17:21:02.587439+00:00
revision: 0
---

## Context

- **Project:** `flint-gate`
- **Phase:** `agent-authz-control-plane`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-gate`
- **Captured:** `2026-07-03T16:57:23Z`
- **Status:** `execution_ready`
- **Progress:** phase changes `2/8`
- **Completed change:** `add-mcp-resource-server` (`G1`), committed as `708191f`
- **Delta spec promoted to:** `openspec/specs/mcp-authorization`

This update advances the [Flint Gate Agent Authorization Control Plane Execution Plan](/flint-gate-agent-authorization-control-plane-execution-plan.md). Change 1 covered budget enforcement and windowed rate limiting progress; see [Flint Gate Agent Authz Budget Rate Limiting Progress](/flint-gate-agent-authz-budget-rate-limiting-progress.md).

## Phase Objective

Evolve `flint-gate` from an auth proxy into an MCP-era agent gateway by adding an agent-authorization control plane on top of existing streaming enforcement:

- Mid-stream SSE token metering
- Session watchdog
- AG-UI/A2UI processing

Scope remains authorization-first. The phase deliberately excludes off-identity LLM-ops features:

- Semantic caching
- Multi-LLM routing
- Multimodal processing

Seed brief: `.kbd-orchestrator/evolution-briefs/ai-agent-gateway-parity.md`

## Build Goals

### 1. Budget enforcement and windowed rate limiting

Extend existing `usage_events` and lifetime `MaxTokenBudget` support into:

- Per-key and per-team token budgets
- Rolling-window token budgets for minute/hour/day windows
- Request-rate limits
- Threshold blocking with clear errors

Status: prior phase work/change 1.

### 2. MCP OAuth 2.1 resource-server support

Implemented in `add-mcp-resource-server`.

Capabilities shipped:

- RFC 9728 protected-resource metadata
- `WWW-Authenticate: resource_metadata` discovery on `401`
- `insufficient_scope` step-up on `403`
- RFC 8414/OIDC authorization-server discovery
- RFC 8707 `resource`/audience validation
- PKCE S256 helper
- No-token-passthrough guard to prevent confused-deputy behavior
- Shared `JwksCache`, refactored so both `jwt_verify` and `mcp` use the same cache implementation

### 3. Embedded policy engine and per-tool-call authorization

Next change: `/kbd-apply add-policy-engine`.

Planned scope:

- Embedded native-Rust policy engine, likely Cedar core or `casbin-rs`
- No sidecar dependency
- Inline stream authorization for each MCP tool call
- Authorization by:
  - Tool name
  - Tool parameters
  - Identity claims
- Filter unauthorized tools from `list_tools` responses
- New `PreRequestHook::Authorize`
- Stream-level tool-call gate
- `ArcSwap` hot reload
- `authz_policies` table
- Write-time policy validation
- Security-reviewer pass required due to security-sensitive scope

## Completed Change: `add-mcp-resource-server`

`add-mcp-resource-server` completed all 9/9 tasks, was verified, archived, and committed as `708191f`.

### Key Security Outcomes

The security review found **1 CRITICAL**, **2 HIGH**, and **3 MEDIUM** issues. All 6 were fixed and re-verified.

#### CRITICAL: confused-deputy audience bypass

Issue C1: `audience: None` silently skipped RFC 8707 enforcement. A token minted for a different resource behind the same authorization server could be accepted.

Fix:

- Audience is now required.
- Issuer is now required.
- Validation fails closed when audience/issuer are absent.

#### JWKS SSRF hardening

Fixes:

- JWKS fetches require `https`.
- Redirects are disabled.
- A dedicated HTTP client is used for JWKS retrieval.

#### JWT key and algorithm hardening

Fixes:

- Key selection is asymmetric-only.
- Algorithm allowlist added.
- Symmetric keys are rejected.

#### JWKS refresh control

Fixes:

- Single-flight JWKS refresh added.
- Refresh rate floor added to prevent excessive refresh behavior.

## Verification Notes

Verification results:

- Workspace tests: **191 passed**, **0 failed**, **3 ignored**
- `clippy`: clean for both feature sets
- `openspec validate`: strict validation passed
- QA gate: PASS

An independent re-verification caught 3 failures missed by the agent report. Root cause was a malformed test-fixture RSA modulus producing:

```text
Base64 error: Invalid input length
```

A probe confirmed the security logic was correct; the fixture was then fixed.

## Deferred Follow-Up

`OctetKeyPair` / Ed25519 / OKP keys are asymmetric but currently grouped with the symmetric-reject branch. This fails closed and is safe, but authorization servers issuing Ed25519 tokens would be rejected. Deferred as non-blocking.

## Current Position

```text
Position: agent-authz-control-plane
status: execution_ready
progress: changes 2/8
remaining: 6 changes
```

Next execution step: `/kbd-apply add-policy-engine`.

# Citations

1. stdin
2. manual:flint-gate/agent-authz-control-plane