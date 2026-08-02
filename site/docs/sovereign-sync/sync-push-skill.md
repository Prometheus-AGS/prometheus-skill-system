---
id: sync-push-skill
title: /sync-push Skill
sidebar_label: /sync-push
---

# /sync-push

Request a push for a named sync domain. The current implementation acknowledges
the request but does not transmit domain state.

## Trigger phrases

- "sync my skills"
- "push to peers"
- "sync learning progress"
- "push sync"

## Requested domain names

| Domain | Privacy |
|--------|---------|
| `skill-index` | Recommended `Public` metadata |
| `learner-model` | Recommended `Trusted` |
| `kbd-control:<project-id>` | `Trusted`; signed authoritative Loro updates plus auxiliary presence |
| `open-spec:<project-id>` | Future project adapter |
| `surreal-memory` | `Local` — must remain ineligible |

These names describe the intended domain model. The `0.1.0` daemon does not
maintain a live manifest registry for the REST handler.

## Quick push

```bash
# Push skill index
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "skill-index"}'

# Push learner model
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "learner-model"}'
```

The current REST handler acknowledges a queued domain request. The structural
`Local` privacy invariant is enforced in the storage/CRDT library; the queue
acknowledgement itself is not export, peer delivery, import, or apply
confirmation. See [Exactly what syncs](./data-scope).

## Source

[`skills/learn/sync-push/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-push/SKILL.md)
