---
id: sync-status-skill
title: /sync-status Skill
sidebar_label: /sync-status
---

# /sync-status

Show the current bounded Sovereign Sync status response. Live P2P state is not
wired to this response in `0.1.0`.

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

The current MCP implementation returns a bounded local summary and directs
clients to the authenticated REST endpoint. The REST sync status is also a
scaffold response in `0.1.0`: it does not read the live P2P node, peer mesh,
domain versions, or applied deltas. Use it to confirm tool/API availability,
not replication health. See [Exactly what syncs](./data-scope).

## Source

[`skills/learn/sync-status/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-status/SKILL.md)
