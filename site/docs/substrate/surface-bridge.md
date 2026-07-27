---
id: surface-bridge
title: surface-bridge
---

# surface-bridge

An Axum HTTP server on `127.0.0.1:7890` providing Tier 2 UI rendering for
learn-domain skills: `/health`, `/mcp/detect-surface-tier`,
`/mcp/render-ui-intent`, and `/mcp/collect-response`.

Skills never render UI directly — they emit a `UiIntent` and the bridge
resolves the harness's capability tier. Installed as a macOS launchd service
by `install-skills-flat.sh`.

*Canonical source: [`substrate/surface-bridge`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/surface-bridge) (crate README).*
