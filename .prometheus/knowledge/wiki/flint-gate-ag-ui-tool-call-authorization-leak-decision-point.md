---
type: Reference
id: flint-gate-ag-ui-tool-call-authorization-leak-decision-point
title: Flint Gate AG-UI Tool-Call Authorization Leak Decision Point
tags:
- flint-gate
- agent-authorization
- ag-ui
- tool-call-authorization
- streaming-enforcement
- mcp
- security-review
links:
- flint-gate-agent-authorization-control-plane-execution-plan
- flint-gate-agent-authz-budget-rate-limiting-progress
- flint-gate-mcp-resource-server-completion-and-security-fixes
sources:
- manual:flint-gate/agent-authz-control-plane
timestamp: 2026-07-03T21:09:57.062439+00:00
created_at: 2026-07-03T21:09:57.062439+00:00
updated_at: 2026-07-03T21:09:57.062439+00:00
revision: 0
---

## Context

- **Project:** `flint-gate`
- **Phase:** `agent-authz-control-plane`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-gate`
- **Captured:** `2026-07-03T20:45:06Z`
- **Source phase record:** `manual:flint-gate/agent-authz-control-plane`
- **Status:** `execution_ready`
- **Progress:** phase changes `3/8`; change 4 is `4/6` tasks complete
- **Security review:** found `C1` CRITICAL plus `H1`/`H2` HIGH; awaiting direction on `H1`

This update continues the [Flint Gate Agent Authorization Control Plane Execution Plan](/flint-gate-agent-authorization-control-plane-execution-plan.md), after prior progress on [Flint Gate Agent Authz Budget Rate Limiting Progress](/flint-gate-agent-authz-budget-rate-limiting-progress.md) and [Flint Gate MCP Resource Server Completion and Security Fixes](/flint-gate-mcp-resource-server-completion-and-security-fixes.md).

## Phase Objective

Turn `flint-gate` from an auth proxy into an MCP-era agent gateway by adding an agent-authorization control plane on top of existing streaming enforcement:

- Mid-stream SSE token metering
- Session watchdog
- AG-UI/A2UI processing

The scope is authorization-first and explicitly excludes off-identity LLM-ops features:

- Semantic caching
- Multi-LLM routing
- Multimodal processing

Seed brief: `.kbd-orchestrator/evolution-briefs/ai-agent-gateway-parity.md`  
Criteria profile: `effort-impact`

## Impact-Weighted Build Goals

1. **Budget enforcement and windowed rate limiting**
   - Extend existing `usage_events` and lifetime `MaxTokenBudget` hook.
   - Add per-key and per-team rolling-window token budgets for minute/hour/day periods.
   - Add request-rate limits.
   - Block threshold violations with clear errors.
   - Gap: `G3`; fastest high-feasibility win.

2. **MCP OAuth 2.1 resource-server support**
   - RFC 9728 protected-resource metadata.
   - `WWW-Authenticate: resource_metadata` on `401`.
   - RFC 8414/OIDC authorization-server discovery.
   - PKCE S256 verification.
   - RFC 8707 `resource` / audience validation.
   - `403 insufficient_scope` step-up.
   - No token passthrough to upstreams to prevent confused-deputy behavior.
   - Gap: `G1`; critical credibility gate.

3. **Embedded policy engine and per-tool-call authorization**
   - Evaluate embedded native-Rust policy engine: Cedar core or `casbin-rs`; no sidecar.
   - Authorize each MCP tool call by tool name, parameters, and identity claims.
   - Filter unauthorized tools from `list_tools` responses, matching the agentgateway pattern.
   - Add `PreRequestHook::Authorize` and stream-level tool-call gate.
   - Gap: `G2`; critical strategic core.

## H1 Security Finding: AG-UI Args Leak Before Fine Authorization

AG-UI, the CopilotKit streaming protocol proxied by `flint-gate`, represents tool calls over SSE in three phases:

| Event | Payload characteristics | Authorization status |
|---|---|---|
| `TOOL_CALL_START` | Tool name only; no arguments yet | Current coarse authorization can run |
| `TOOL_CALL_ARGS` | One or more incremental JSON argument deltas | Currently forwarded as they arrive |
| `TOOL_CALL_END` | Signals complete argument JSON | Current fine authorization can run |

Current behavior performs two checks:

1. **Coarse check at `TOOL_CALL_START`**
   - Uses tool name only.
   - Answers: may this principal ever call this tool?

2. **Fine check at `TOOL_CALL_END`**
   - Uses complete arguments.
   - Answers: may this principal call this tool with these arguments?

The H1 gap is that `TOOL_CALL_ARGS` deltas are forwarded to the client before the `TOOL_CALL_END` fine check runs. If the fine check later denies the call, forbidden arguments have already been delivered. If a malicious upstream never sends `TOOL_CALL_END`, the fine check never runs.

## Product Impact

`flint-gate`'s differentiator in this phase is **mid-stream enforcement**: enforcing during the stream rather than only at connection time. H1 undermines that claim for argument-level tool authorization because the arg-level decision occurs after delivery.

## Core Tradeoff

To make argument-level policy a true pre-delivery control, `flint-gate` must buffer the whole tool call until complete arguments can be authorized.

- **Security benefit:** forbidden arguments are not delivered before the fine-grained policy decision.
- **Cost:** tool-call arguments no longer stream token-by-token to the client.
- **Scope of UX impact:** normal text streaming is unaffected; only streamed tool-call args incur buffering latency.

## Decision Needed

Direction is needed on whether to treat H1 as a blocking design change or defer it as a documented limitation.

Clarifying questions raised for decision:

- Is argument-level authorization required now, or is coarse by-name tool gating sufficient for the near term?
- Who consumes partial streamed tool-call args, and does the client act on them incrementally?
- Should H1 be split into a follow-up while landing `C1` DoS remediation, `H2` `list_tools` fail-open remediation, and `L2` now?
- How concerning is the buffering latency cost for tool-call args specifically, given normal text streaming remains unaffected?

## Open Security Items

- `C1` CRITICAL: DoS issue identified by security review.
- `H1` HIGH: AG-UI tool-call args leak before fine authorization; awaiting product/security direction.
- `H2` HIGH: `list_tools` fail-open issue identified by security review.
- `L2`: lower-priority item available to land with the current batch if H1 is split out.

# Citations

1. manual:flint-gate/agent-authz-control-plane