---
id: sync-push-skill
title: /sync-push Skill
sidebar_label: /sync-push
---

# /sync-push

Push the current state of a sync domain to all connected peers.

## Trigger phrases

- "sync my skills"
- "push to peers"
- "sync learning progress"
- "push sync"

## Supported domains

| Domain | Privacy |
|--------|---------|
| `skill-index` | Shareable |
| `learner-model` | Shareable |
| `surreal-memory` | **LocalOnly — never synced** |

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

## Privacy guarantee

Requesting `surreal-memory` domain is rejected at the REST layer with HTTP 400.
This is enforced in code — not configuration.

## Source

[`skills/learn/sync-push/SKILL.md`](https://github.com/prometheusags/prometheus-skill-pack/blob/main/skills/learn/sync-push/SKILL.md)
