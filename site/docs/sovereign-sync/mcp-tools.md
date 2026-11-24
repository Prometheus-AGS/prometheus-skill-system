---
id: mcp-tools
title: MCP Tools
sidebar_label: MCP Tools
---

# Connected MCP tools

Companion's `sovereign-sync --mode mcp` serves stdio tools. Select the existing
host with `--socket` or `SOVEREIGN_SYNC_SOCKET`. Registration never starts a
host or creates an independent P2P identity; unavailable host means unavailable.

| Tool | Request and result boundary |
|---|---|
| `sync-status` | Optional `domain` annotates the selected host JSON; not convergence |
| `sync-peers` | Selected host peer/transport snapshot; not enrollment |
| `sync-push` | Explicit `domain`, optional unchanged `signed_request`; intent persisted before one POST |
| `sync-push-receipt` | Original `request_id`, looked up on the same host |

A new push uses the named project's enrolled signer. Generic syncable domains
require exactly one registered project to choose that signer; resolve ambiguity.
Read [durable intent and uncertainty](/docs/sovereign-sync/signed-pushes-and-receipts)
before replay. The source rejects unsupported `--prefix-tools` and
`UAR_SKILL_SERVICE_URL` passthrough; do not advertise tool aliases or an
undeclared extra tool set. Use [Companion registration](/docs/sovereign-sync/installation).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
