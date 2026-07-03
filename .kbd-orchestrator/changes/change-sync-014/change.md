# change-sync-014: /sync-peers skill

**Phase:** phase-learn-sovereign-sync
**Tier:** 3 (parallelize with 013, 015 after Tier 2)
**Status:** pending
**Gap:** G-09

## Summary

Create `skills/sync/sync-peers/SKILL.md`. Lists all known peers with
their EndpointId, last-seen timestamp, and domain sync coverage.

## Files to change

- `skills/sync/sync-peers/SKILL.md` — new skill

## Frontmatter

```yaml
---
name: sync-peers
description: List all known sovereign-sync peers, their connection status, and which domains they share
version: '1.0.0'
license: MIT
metadata:
  author: travis-james
  category: sync
  tags: [sync, p2p, peers, sovereign]
---
```

## Tasks

- [ ] Write SKILL.md
- [ ] Run `npm run validate:strict skills/sync/sync-peers`
