---
id: change-prb-008-tag-v160
title: Commit prometheus-research crate, tag v1.6.0, push to origin/main
phase: phase-prometheus-research-binary
priority: P1
effort: S
wave: 4
agent: general-purpose
status: pending
gap_id: G-08
verdict: BUILD
depends_on: change-prb-007-tests
scope:
  - package.json
  - plugin.json
  - .claude-plugin/marketplace.json
  - CLAUDE.md
---

# Change: Commit + tag v1.6.0

## Problem

`prometheus-research` exists locally but is not committed, tagged, or
registered in the substrate documentation.

## Solution

1. Update `package.json` version to `1.6.0`
2. Update `plugin.json` version to `1.6.0`
3. Update `.claude-plugin/marketplace.json` — add `prometheus-research-server` plugin entry
4. Update `CLAUDE.md` substrate crates table — add `prometheus-research` row
5. `git add substrate/prometheus-research/` + all updated files
6. `git commit -m "feat(substrate): add prometheus-research Rust binary (v1.6.0)\n\n- HTTP server on :7891 with AG-UI SSE streaming\n- 5 MCP tools (research_start/status/cancel/export, render_component)\n- A2UI component registry: 8 server-rendered HTMX fragments\n- HTMX 2.0.8 + htmx-ext-sse + Alpine.js vendored static assets\n- launchd auto-start service\n- 8 integration tests"`
7. `git tag v1.6.0`
8. `git push origin main --tags`

## Acceptance Criteria

- [ ] `git log --oneline -1` shows the feat(substrate) commit
- [ ] `git tag -l v1.6.0` shows the tag
- [ ] `git push` completes without error
- [ ] `CLAUDE.md` substrate table includes `prometheus-research` row
- [ ] `package.json` version is `"1.6.0"`
