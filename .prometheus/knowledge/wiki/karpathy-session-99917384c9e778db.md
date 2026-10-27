---
type: SessionRecord
id: karpathy-session-99917384c9e778db
title: Karpathy session 99917384c9e7
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T13:51:38.924806+00:00
created_at: 2026-10-09T13:51:38.924806+00:00
updated_at: 2026-10-09T13:51:38.924806+00:00
revision: 0
---

## Delta

Both backup folders are deleted, and `~/.prometheus/backups` is now empty. That's gone for good, since I used `rm -rf` rather than the Trash:
- `foreign-skills-20261009-064751` (13 MB, the 279 older skill copies);
- `learning-queue-20261009-072214` (1.6 MB, the queue copy).

The Codex config backup the memory helper made in `~/.codex/` is separate and still there. The queue copy is gone, so there is no backup of the learning queue from before the drain. The server shows all 145 operations committed, so I'd expect that to be fine.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T13:51:37.434421Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- tools/surreal-memory-server
- crates/prometheus-exec/.prometheus/
- history.txt
