---
id: surface-bridge
title: surface-bridge
---

# surface-bridge

An Axum HTTP server on `127.0.0.1:7890` providing Tier 2 UI rendering for
learn-domain skills: `/health`, `/mcp/detect-surface-tier`,
`/mcp/render-ui-intent`, and `/mcp/collect-response`.

Skills never render UI directly — they emit a `UiIntent` and the bridge
resolves the harness's capability tier. Skill installation builds the binary; `install-mcp-services.sh` renders and starts the supported macOS/Linux service template. See [service operations](/docs/guide/service-operations).

*Canonical source: [`substrate/surface-bridge`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/surface-bridge) (crate README).*
