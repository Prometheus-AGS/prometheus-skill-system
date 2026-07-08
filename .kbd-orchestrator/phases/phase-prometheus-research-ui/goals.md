# Goals — phase-prometheus-research-ui

## Context

`phase-prometheus-research-binary` shipped the `prometheus-research` Rust binary (v1.6.0)
with an Axum HTTP server on :7891, AG-UI SSE streaming, 8 A2UI HTMX component endpoints,
5 MCP tools, and a launchd auto-start service. The binary is complete but not yet wired into
the `deep-research` skill front-end or the surface-bridge Tier 2 MCP App flow.

This phase closes that gap: real-time research visualization in the browser via HTMX + SSE,
the `render_component` MCP tool wired into surface-bridge so Claude Code can render A2UI
fragments in its artifact panel, and a CI job so the binary never regresses silently.

## Goals

- [ ] **G-01: Update `deep-research` SKILL.md** — add instructions for starting
  `prometheus-research --mode server` and connecting to the SSE stream; describe how to
  use `research_start` / `research_status` / `research_cancel` MCP tools from within a
  research session

- [ ] **G-02: Ship polished `deep-research-ui.html`** — a standalone HTMX 2.0.8 + Alpine.js
  UI served at `http://localhost:7891` that connects to the AG-UI SSE stream and hot-swaps
  the 8 A2UI components as the research progresses; replaces the prototype in
  `docs/deep-research/deep-research-ui.html`

- [ ] **G-03: Wire `render_component` into surface-bridge Tier 2 flow** — update
  `skills/learn/ui-surface/SKILL.md` (or add a new skill step) so that when an MCP tool
  returns a `render_component` result, the agent knows to POST it to
  `http://127.0.0.1:7890/mcp/render-ui-intent` for Tier 2 iframe rendering

- [ ] **G-04: Add CI job for `prometheus-research` binary** — create or extend the GitHub
  Actions workflow to build `substrate/prometheus-research` and run `cargo test` on every PR,
  preventing silent regressions (matches the pattern in `phase-sovereign-sync-hardening`
  CI change)

- [ ] **G-05: Integration smoke test** — a shell script or test that starts
  `prometheus-research --mode server`, hits `/health`, starts a job, verifies SSE events,
  and cancels — giving CI a real end-to-end gate beyond unit tests
