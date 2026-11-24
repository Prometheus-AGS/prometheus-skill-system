---
id: architecture
title: Architecture
sidebar_label: Architecture
---

# Connected architecture

Headless and windowed Companion share the same-user Unix socket and local signed
KBD authority. `sovereign-sync --mode daemon` offers the same production
supervisor/consumer path as an alternative host. Choose one socket owner.
`--mode server` disables P2P; stdio `--mode mcp` connects tools to an existing host.

Local control is installed before optional P2P startup. Disabled or failed peer
transport does not manufacture a replacement signer or require losing local
control. An explicit pack manifest may attach service-supervisor reports;
its absence leaves local authority independent. Neither service reports nor
transport readiness proves a peer applied signed project state.

The MCP Unix HTTP client disables proxies and redirects, bounds its requests
and has no TCP fallback. It reports an unreachable host as unavailable rather
than reporting a private empty transport as the selected host's state.

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
