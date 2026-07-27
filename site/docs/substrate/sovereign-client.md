---
id: sovereign-client
title: sovereign-client
---

# sovereign-client

The Rust SDK for [sovereign-sync](/docs/sovereign-sync/overview): a
`reqwest`-based client for the REST API plus an `eventsource-stream`
consumer for the AG-UI SSE feed. Entry point: `SovereignClient::new(base_url)`.

Use it from Tauri/desktop apps or services that need CRDT domain sync
without speaking MCP.

*Canonical source: [`substrate/sovereign-client`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/sovereign-client).*
