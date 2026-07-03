# change-sync-013: /sync-status skill

**Phase:** phase-learn-sovereign-sync
**Tier:** 3 (after Tier 2)
**Status:** pending
**Gap:** G-09

## Summary

Create `skills/sync/sync-status/SKILL.md`. Shows current P2P sync status,
peer count, domain coverage, and last sync timestamp. Must pass 5-harness
validation (Claude Code, Kimi, MiniMax, OpenCode, Codex).

## Files to change

- `skills/sync/sync-status/SKILL.md` — new skill

## Frontmatter

```yaml
---
name: sync-status
description: Show current P2P sync status for all domains, peer count, and last sync timestamp
version: '1.0.0'
license: MIT
metadata:
  author: travis-james
  category: sync
  tags: [sync, p2p, status, sovereign]
---
```

## Tasks

- [ ] Write SKILL.md with frontmatter, description, when-to-use, instructions
- [ ] Instructions invoke sovereign-sync via MCP tool call or JSON-RPC
- [ ] Run `npm run validate:strict skills/sync/sync-status`
- [ ] Verify loads in Claude Code via /sync-status
