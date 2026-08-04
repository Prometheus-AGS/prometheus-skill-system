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

- Returns the live bounded peer summary from the shared sync service
- Reports the durable local endpoint ID and enrolled peer endpoint IDs
- Separates the random paired-group secret from distinct endpoint identities
- Points to pairing, allow-list, and transport diagnostics

## Current behavior

`/health` reports service status and version. The `sync-peers` MCP tool and
authenticated `GET /api/v1/sync/peers` route call the same service layer and
return the durable endpoint identity, transport state, enrolled peers, and
bounded live-neighbor status.

Enrollment is explicit: export a redacted pairing ticket on one machine,
import it on the other, and verify the endpoint-to-signing-key allow-list.
Add/remove operations are not health-route side effects. Follow the
[two-machine pairing procedure](./pair-two-machines), and never log or paste a
complete ticket into an issue or diagnostic report.

## Source

[`skills/learn/sync-peers/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-peers/SKILL.md)
