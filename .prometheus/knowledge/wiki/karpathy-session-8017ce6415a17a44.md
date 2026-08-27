---
type: SessionRecord
id: karpathy-session-8017ce6415a17a44
title: Karpathy session 8017ce6415a1
tags:
- karpathy
- session-learning
sources:
- session:69948acc-ad6d-4a8c-a02a-ba8a453e0345
timestamp: 2026-08-26T18:14:08.659786+00:00
created_at: 2026-08-26T18:14:08.659786+00:00
updated_at: 2026-08-26T18:14:08.659786+00:00
revision: 0
---

## Delta

Confirmed the fix resolves the exact reproduced bug: with the old byte-size key, the two capture points produced different identities (2771786 vs 2786050 bytes, 2 more lines appended in between) causing the duplicate. With the new last-assistant-UUID anchor, both capture points now resolve to the same UUID (`0123ff28-...`), correctly collapsing the twin-hook duplicate.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 69948acc-ad6d-4a8c-a02a-ba8a453e0345
- Captured: 2026-08-26T18:09:03.535542Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- shared/scripts/enqueue-learning-job.py
- .prometheus/knowledge/wiki/karpathy-session-1b1dfa94622bc16b.md
- .prometheus/knowledge/wiki/karpathy-session-96b197bd660fb49e.md
- .prometheus/knowledge/wiki/karpathy-session-d8f51b4893ffc91b.md
- crates/prometheus-exec/.prometheus/
- shared/scripts/__pycache__/
