---
id: sync-peers-skill
title: /sync-peers Skill
sidebar_label: /sync-peers
---

# /sync-peers

Inspect the bounded peer summary and learn how Sovereign Sync pairing is
configured.

## Trigger phrases

- "add peer"
- "show peers"
- "list peers"
- "sync with \<device\>"

## What it does

- Describes the configured bootstrap-peer model
- Explains how to find the current endpoint ID in startup logs
- Separates the shared operator ID from distinct endpoint IDs
- Points to network and pairing limitations

## Current behavior

`/health` reports service status and version; it does not expose a node ID.
The current `sync-peers` MCP tool and authenticated
`GET /api/v1/sync/peers` route return the known peer summary. Bootstrap peer
tickets/addresses are configured in `config.toml`, not added through the
health route.

In `0.1.0`, the MCP and REST peer summaries are not connected to
`P2PNode` and return no live neighbors. Add/remove operations are not
implemented. Use the log-driven
[two-machine pairing procedure](./pair-two-machines) for connectivity
development.

## Source

[`skills/learn/sync-peers/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-peers/SKILL.md)
