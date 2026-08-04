---
id: sync-status-skill
title: /sync-status Skill
sidebar_label: /sync-status
---

# /sync-status

Show the current bounded Sovereign Sync status response from the same service
layer used by REST and signed push execution.

## Trigger phrases

- "sync status"
- "check sync"
- "is sync running"
- "how many peers"

## What it shows

- Current local node state
- Connected-peer summary
- Requested domain or all domains

## Output

```
sovereign-sync status
─────────────────────
State     : idle
Peers     : 0
Domain    : all
```

The MCP implementation returns the live transport state, durable endpoint ID,
bounded peer summary, and receipt-backed domain activity. Use the push receipt
and its per-peer applied state—not the process-level status alone—to prove
replication. See [Exactly what syncs](./data-scope).

## Source

[`skills/learn/sync-status/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-status/SKILL.md)
