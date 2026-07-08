---
id: change-prb-004-http-sse-server
title: Implement Axum HTTP server with AG-UI SSE streaming and surface-bridge emit
phase: phase-prometheus-research-binary
priority: P0
effort: L
wave: 2
agent: general-purpose
status: pending
gap_id: G-06
verdict: BUILD
depends_on: change-prb-002-cli-subcommands
scope:
  - substrate/prometheus-research/src/http_server/mod.rs
  - substrate/prometheus-research/src/http_server/health.rs
  - substrate/prometheus-research/src/http_server/sse.rs
  - substrate/prometheus-research/src/http_server/rest.rs
  - substrate/prometheus-research/src/agui/mod.rs
  - substrate/prometheus-research/src/agui/emit.rs
---

# Change: HTTP server with AG-UI SSE + surface-bridge emit

## Problem

No real-time progress streaming. The HTMX UI prototype (`docs/deep-research/deep-research-ui.html`)
has no live backend to connect to.

## Solution

Implement `--mode server` on port 7891 with Axum 0.8:

### Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | `{ status, version, pid }` |
| POST | `/api/v1/jobs` | Start job — body: `{ query, depth, max_sources, citation_style }` |
| GET | `/api/v1/jobs/:id` | Get job status (reads checkpoint) |
| DELETE | `/api/v1/jobs/:id` | Cancel job |
| GET | `/api/v1/jobs/:id/events` | SSE stream of AG-UI events (`text/event-stream`) |
| GET | `/components/:name` | A2UI component HTML fragment (stub → filled in change-005) |
| GET | `/static/*path` | Static file serving for htmx.min.js, alpine.min.js, etc. |

### AG-UI event types

```rust
pub enum AguiEvent {
    AgentStatus { job_id, stage, stage_name, progress, status, tokens, timestamp },
    AgentMessage { job_id, message, level, timestamp },
    AgentError   { job_id, message, stage, timestamp },
    A2uiComponent { job_id, component, props, timestamp },
}
```

### SSE channel

Use `tokio::sync::broadcast::<AguiEvent>(128)`. The `/events` route subscribes
to the channel and streams events. The job daemon (when started later) publishes
to the broadcast sender.

### surface-bridge emit

`agui::emit::emit_to_surface_bridge(event)` POSTs a `UiIntent` to
`http://127.0.0.1:7890/mcp/render-ui-intent` for Tier 2 MCP App rendering.
Failure is non-fatal (log warn, continue).

## Acceptance Criteria

- [ ] `prometheus-research --mode server` starts on port 7891
- [ ] `GET /health` returns 200 with JSON body
- [ ] `POST /api/v1/jobs` returns `{ job_id }` and spawns background job
- [ ] `GET /api/v1/jobs/:id/events` returns `text/event-stream` content-type
- [ ] `curl -N http://localhost:7891/api/v1/jobs/<id>/events` streams SSE events
- [ ] `GET /static/htmx.min.js` returns 200 (file vendored in next change)
- [ ] `cargo build --release` — 0 errors
