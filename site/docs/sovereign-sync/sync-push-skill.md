---
id: sync-push-skill
title: /sync-push Skill
sidebar_label: /sync-push
---

# /sync-push

Create or exactly replay a signed push for a named sync domain. MCP and REST
use the same service and durable receipt store.

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

These names are validated against the live default-deny manifest. Unknown and
`Local` domains are rejected before export.

## Quick push

```bash
# Submit a pre-signed canonical request through the private Unix transport
curl --unix-socket "$SOCKET" -s -X POST http://localhost/api/v2/sync/pushes \
  -H 'Content-Type: application/json' \
  --data @signed-push.json

# Recover the durable receipt after response loss
curl --unix-socket "$SOCKET" -s \
  http://localhost/api/v2/sync/pushes/<push-id>
```

The receipt's canonical payload hash makes same-ID/same-payload retries exact.
Reusing the ID with a different payload returns `409`. Per-peer states and
ordered SSE events distinguish receipt, apply, and rejection; a local
`accepted` state alone is not remote-apply proof. See
[Signed pushes and receipts](./signed-pushes-and-receipts) and
[Exactly what syncs](./data-scope).

## Source

[`skills/learn/sync-push/SKILL.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/learn/sync-push/SKILL.md)
