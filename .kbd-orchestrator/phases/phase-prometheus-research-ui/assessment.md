# Assessment — phase-prometheus-research-ui

_Generated: 2026-07-08_

## Executive Summary

`prometheus-research` v1.6.0 is fully operational: HTTP server :7891, AG-UI SSE stream,
8 A2UI component endpoints, 5 MCP tools, launchd auto-start. The missing layer is the
user-facing front-end integration: the existing `docs/deep-research/deep-research-ui.html`
is a 4339-line Alpine.js simulation — it calls `simulateProgress()` / `simulateSongResearch()`
rather than making any real HTTP calls. The `deep-research` SKILL.md has zero references to
the binary. The `ui-surface` Tier 2 MCP App iframe path is marked `STUBBED`.
The `prometheus-research` crate has no CI coverage.

**Assessment verdict:** PROCEED. All five goals are well-scoped, no external blockers.
The sovereign-sync CI workflow is a ready-made template for G-04. The existing simulation
UI is rich enough that G-02 (replacing simulation with real SSE calls) is the dominant
work item.

---

## Goal Coverage

| Goal | Status | Gap |
|------|--------|-----|
| G-01: Update `deep-research` SKILL.md | NOT MET | Zero mentions of `prometheus-research`, `research_start`, `--mode server`, :7891, or SSE |
| G-02: Polished `deep-research-ui.html` with real SSE | NOT MET | `startResearch()` calls `simulateProgress()` / `simulateSongResearch()` — no `EventSource`, no `fetch` to :7891 |
| G-03: `render_component` surface-bridge Tier 2 wiring | PARTIAL | surface-bridge is running (port 7890 responds `/health` OK); `render_ui_intent` endpoint exists and accepts POST; `ui-surface` SKILL.md says Tier 2 is `STUBBED` — protocol mismatch between prometheus-research `UiIntent` shape and surface-bridge `UiIntent` type (see below) |
| G-04: CI job for `prometheus-research` | NOT MET | No `.github/workflows/*.yml` references `substrate/prometheus-research` |
| G-05: Integration smoke test | NOT MET | `substrate/prometheus-research/scripts/` does not exist; no shell smoke test |

---

## Detailed Gap Analysis

### G-01: `deep-research` SKILL.md

`skills/research/deep-research/SKILL.md` (284 lines) has no reference to:
- `prometheus-research --mode server` to start the HTTP backend
- `prometheus-research --mode mcp` (already auto-started via launchd)
- `research_start` / `research_status` / `research_cancel` MCP tools
- The SSE stream at `GET /api/v1/jobs/{id}/events`
- Background job execution that survives context-window exhaustion

**Implementation:** add a `## Background Execution (prometheus-research)` section after the
`## Quick Start` section, covering: verifying the binary is running, starting a job via
`research_start` MCP tool, monitoring via `research_status`, cancelling, and connecting
the HTMX UI to the live stream.

### G-02: `docs/deep-research/deep-research-ui.html`

Current state (v3.0.0): rich simulation with 4339 lines. Key findings:
- `startResearch()` creates a local Alpine job object, then forks to either
  `simulateSongResearch(rjob)` or `simulateProgress(rjob)` — both are pure client-side timers
- No `EventSource`, no `fetch()` to any localhost endpoint
- HTMX 2.0.8 is loaded from `unpkg.com` (CDN, not the vendored binary copy)
- Alpine.js 3.14.3 is loaded from jsDelivr CDN

**Gap:** Replace `simulateProgress()` and `simulateSongResearch()` call sites in `startResearch()`
with:
1. `POST http://127.0.0.1:7891/api/v1/jobs` → get `job_id`
2. `new EventSource('/api/v1/jobs/' + jobId + '/events')` via the HTMX ext-sse extension
   (or native `EventSource`) to receive AG-UI events
3. Map incoming `AguiEvent` types to Alpine state updates (progress ring, stage timeline,
   sources list, contradictions, graph view)
4. Switch from CDN HTMX/Alpine to the vendored copies served by the binary at `/static/`
   (allows full offline use)

The existing simulation code can stay as a `demo` mode behind a toggle — valuable for
showcasing the UI without a running binary.

**Architecture note:** the HTMX SSE extension (`htmx-ext-sse`) is the cleanest path since it is
already vendored. Add `hx-ext="sse"` to the job event container, point `sse-connect` to the
job events endpoint, and use `sse-swap` to update the appropriate component div when AG-UI
events arrive. For the initial job POST and job list, use `hx-post` / `hx-get` targeting
:7891.

### G-03: `render_component` surface-bridge Tier 2 wiring

Two sub-gaps:

**Sub-gap A — Protocol mismatch:**
`prometheus-research` `emit.rs` sends:
```json
{ "intent": "research.event", "payload": { <AguiEvent> } }
```
But `surface-bridge` `UiIntent` struct expects:
```json
{ "intent_type": "...", "title": "...", "body": "...", "options": [...], "multiselect": false, "request_id": "..." }
```
The `render_ui_intent` handler deserializes `Json<UiIntent>` — the incoming shape from
`prometheus-research` will fail deserialization (`title`, `request_id` are missing).

**Fix:** update `prometheus-research/src/agui/emit.rs` to send the `surface-bridge`-compatible
`UiIntent` struct (adding `intent_type`, `title`, `body`, `request_id` derived from the event).

**Sub-gap B — `ui-surface` SKILL.md still says Tier 2 is STUBBED:**
The surface-bridge IS running and `/mcp/render-ui-intent` works. The SKILL.md text is
outdated. Update `skills/learn/ui-surface/SKILL.md` Tier 2 section to document the real
workflow:
1. Agent calls `render_component("graph_view", props)` MCP tool
2. `prometheus-research --mode mcp` returns an HTML fragment
3. Agent POSTs `UiIntent` to `http://127.0.0.1:7890/mcp/render-ui-intent`
4. `surface-bridge` logs the intent; `/mcp/collect-response` polls for user response

### G-04: CI job for `prometheus-research`

Template: `.github/workflows/sovereign-sync.yml` — 3-job matrix (fmt / clippy / test)
per crate, `dtolnay/rust-toolchain@stable`, `actions/cache@v4`.

New file: `.github/workflows/prometheus-research.yml`

Path triggers:
```yaml
paths:
  - ".github/workflows/prometheus-research.yml"
  - "substrate/prometheus-research/**"
```

Matrix: single crate `substrate/prometheus-research` × commands `[fmt, clippy, test]`.

Cache key: hash of `substrate/prometheus-research/Cargo.lock`.

### G-05: Integration smoke test

`substrate/prometheus-research/scripts/smoke-test.sh`:
1. Start `prometheus-research --mode server` in background, capture PID
2. Poll `GET /health` with retries until 200 (max 5s)
3. `POST /api/v1/jobs` with `{"query": "test smoke query"}`; assert `job_id` present
4. `GET /api/v1/jobs/{job_id}` → assert `status` field present
5. Open SSE stream; read first event; verify `Content-Type: text/event-stream`
6. `DELETE /api/v1/jobs/{job_id}` → assert `cancelled: true`
7. Kill server; verify it exits cleanly
8. Exit 0 on pass, 1 on any failure

This can be wired into the CI workflow as an optional step (skipped when binary is not
pre-built, enabled with `--features smoke` or a CI flag).

---

## Architecture Reference

### Port allocation (unchanged)

| Service | Port |
|---------|------|
| surface-bridge | 7890 |
| prometheus-research HTTP | **7891** |
| sovereign-sync | 7892 |

### prometheus-research REST API (live on :7891)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Service health check |
| `/api/v1/jobs` | POST | Start a research job; returns `job_id` |
| `/api/v1/jobs/{id}` | GET | Read checkpoint (status, stage, progress) |
| `/api/v1/jobs/{id}` | DELETE | Cancel job; sends SIGTERM |
| `/api/v1/jobs/{id}/events` | GET | AG-UI SSE stream (`text/event-stream`) |
| `/components/{name}` | GET | A2UI HTMX fragment (query-string props) |
| `/static/{*path}` | GET | Vendored JS: htmx.min.js, alpine.min.js, etc. |

### AG-UI event types (over SSE)

```json
{ "type": "agent.status",   "job_id": "...", "stage": 3, "stage_name": "retrieve", "progress": 35, "status": "running", "tokens": 0, "timestamp": "..." }
{ "type": "agent.message",  "job_id": "...", "message": "...", "level": "info", "timestamp": "..." }
{ "type": "agent.error",    "job_id": "...", "message": "...", "stage": 5, "timestamp": "..." }
{ "type": "a2ui.component", "job_id": "...", "component": "graph_view", "props": {}, "timestamp": "..." }
```

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Binary not on PATH in CI | Medium | Use `cargo run --` in CI smoke test rather than relying on PATH install |
| HTMX SSE extension cross-origin restriction | Low | Server already has `CorsLayer::new().allow_origin(Any)` |
| surface-bridge UiIntent deserialization failure | **High** | Fix `emit.rs` shape mismatch in G-03 before enabling Tier 2 |
| Alpine.js CDN→local script path breakage | Low | vendored files at `/static/` already work (confirmed by SSE tests) |
| SSE EventSource with `127.0.0.1` in browser | Low | Browser allows localhost EventSource with CORS wildcard |

---

## Open Questions

None. All integration points are clear.

---

## Change Plan Preview (for /kbd-plan)

| Change ID | Scope | Goals |
|-----------|-------|-------|
| `change-prui-001-skill-md-update` | `skills/research/deep-research/SKILL.md` | G-01 |
| `change-prui-002-htmx-ui-real-sse` | `docs/deep-research/deep-research-ui.html` | G-02 |
| `change-prui-003-surface-bridge-wiring` | `src/agui/emit.rs` + `skills/learn/ui-surface/SKILL.md` | G-03 |
| `change-prui-004-ci-workflow` | `.github/workflows/prometheus-research.yml` | G-04 |
| `change-prui-005-smoke-test` | `substrate/prometheus-research/scripts/smoke-test.sh` | G-05 |
