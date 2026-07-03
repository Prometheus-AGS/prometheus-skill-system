# change-sync-015: /sync-push skill

**Phase:** phase-learn-sovereign-sync
**Tier:** 3 (parallelize with 013, 014 after Tier 2)
**Status:** pending
**Gap:** G-09

## Summary

Create `skills/sync/sync-push/SKILL.md`. Triggers a push of specified domains
(or all if none given) to all connected peers. Emits AG-UI stream events for
progress visibility.

## Files to change

- `skills/sync/sync-push/SKILL.md` — new skill

## Frontmatter

```yaml
---
name: sync-push
description: Push local knowledge domains to sovereign-sync peers; streams progress via AG-UI
version: '1.0.0'
license: MIT
metadata:
  author: travis-james
  category: sync
  tags: [sync, p2p, push, sovereign, ag-ui]
---
```

## Tasks

- [ ] Write SKILL.md with full instructions
- [ ] Include example for pushing a specific domain vs all domains
- [ ] Run `npm run validate:strict skills/sync/sync-push`
