---
id: sync-status-skill
title: /sync-status Skill
sidebar_label: /sync-status
---

# /sync-status

Show the live status of the local sovereign-sync node.

## Trigger phrases

- "sync status"
- "check sync"
- "is sync running"
- "how many peers"

## What it shows

- Node state (Disconnected → Connected → Idle)
- Number of connected peers
- Which sync domains are active

## Output

```
sovereign-sync status
─────────────────────
State     : Connected
Peers     : 2
Domains   : skill-index, learner-model
```

## Source

[`skills/learn/sync-status/SKILL.md`](https://github.com/prometheusags/prometheus-skill-pack/blob/main/skills/learn/sync-status/SKILL.md)
