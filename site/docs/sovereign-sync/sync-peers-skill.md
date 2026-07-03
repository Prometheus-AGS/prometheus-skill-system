---
id: sync-peers-skill
title: /sync-peers Skill
sidebar_label: /sync-peers
---

# /sync-peers

Manage peers in the sovereign-sync P2P gossip network.

## Trigger phrases

- "add peer"
- "show peers"
- "list peers"
- "sync with \<device\>"

## What it does

- Lists connected peer node IDs and addresses
- Explains how to find your own node ID to share
- Describes automatic discovery via operator key grouping

## Finding your node ID

```bash
curl -s http://127.0.0.1:7892/health | jq .node_id
```

Share this with other devices — they can add it as a bootstrap peer.

## Source

[`skills/learn/sync-peers/SKILL.md`](https://github.com/prometheusags/prometheus-skill-pack/blob/main/skills/learn/sync-peers/SKILL.md)
