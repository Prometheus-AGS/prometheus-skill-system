# change-prui-002-htmx-ui-real-sse

## Summary

Replace the simulation engine in `docs/deep-research/deep-research-ui.html` with real
HTTP calls and an `EventSource` SSE connection to `prometheus-research` on `:7891`.
The current file calls `simulateProgress()` / `simulateSongResearch()` — pure client-side
timers with no real backend communication. This change wires the UI to the live binary.

## Goal

G-02: Ship polished `deep-research-ui.html` with real SSE

## Files Changed

- `docs/deep-research/deep-research-ui.html` — replace CDN scripts + simulation with real HTTP/SSE

## Acceptance Criteria

- [ ] HTMX loaded from `/static/htmx.min.js` (not `unpkg.com`)
- [ ] Alpine.js loaded from `/static/alpine.min.js` (not jsDelivr CDN)
- [ ] `startResearch()` POSTs to `http://127.0.0.1:7891/api/v1/jobs` and captures `job_id`
- [ ] SSE stream opened at `http://127.0.0.1:7891/api/v1/jobs/{job_id}/events` via `EventSource` or HTMX `hx-ext="sse"`
- [ ] `agent.status` events update the progress ring and stage timeline
- [ ] `agent.message` events append to the log panel
- [ ] `agent.error` events trigger the error state UI
- [ ] `a2ui.component` events swap in HTMX component fragments
- [ ] Cancel button calls `DELETE http://127.0.0.1:7891/api/v1/jobs/{job_id}`
- [ ] Demo mode preserved behind `?demo=1` query param

## Risk

Medium. Large existing file (4339 lines). Simulation logic must be preserved in demo mode.
CORS: server already sends `Access-Control-Allow-Origin: *` — no issue expected.
