---
id: mcp-substrate
title: MCP Substrate
sidebar_label: MCP Substrate
---

# MCP Substrate

See the full chapter:
[docs/guide/05-mcp-substrate.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/05-mcp-substrate.md)

## Included MCP servers

| Server | Port / Transport | Purpose |
|--------|-----------------|---------|
| `surreal-memory` | HTTP REST + SSE | Knowledge graph, palace RAG, task streams |
| `sycophancy-correction` | stdio | Anti-sycophancy detection and correction |
| `sovereign-sync` | stdio (MCP) / HTTP :7892 (daemon) | P2P CRDT sync |
| `surface-bridge` | HTTP :7890 | Tier 2 UI rendering |
| `learner-model` | JSON-RPC stdio | FSRS-6 scheduler |

All MCP servers are installed by `bash scripts/install-skills-flat.sh`.
