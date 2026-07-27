---
id: prometheus-research
title: prometheus-research
---

# prometheus-research

Background deep-research daemon on `127.0.0.1:7891` (v1.6.0): five MCP tools
(`research_start/status/cancel/export`, `render_component`), an AG-UI SSE
event stream, and an A2UI registry of eight server-rendered HTMX fragments
(HTMX 2.0.8 + Alpine.js vendored).

Auto-starts via `com.prometheus.research.plist` launchd service; installed by
`scripts/install-binaries.sh`.

*Canonical source: [`substrate/prometheus-research`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/prometheus-research) — modules: `a2ui`, `agui`, `config`.*
