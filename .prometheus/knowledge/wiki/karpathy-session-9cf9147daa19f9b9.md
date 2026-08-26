---
type: SessionRecord
id: karpathy-session-9cf9147daa19f9b9
title: Karpathy session 9cf9147daa19
tags:
- karpathy
- session-learning
sources:
- session:69948acc-ad6d-4a8c-a02a-ba8a453e0345
timestamp: 2026-08-26T18:27:37.700094+00:00
created_at: 2026-08-26T18:27:37.700094+00:00
updated_at: 2026-08-26T18:27:37.700094+00:00
revision: 0
---

## Delta

Summary: the compiled `prometheus-learning-worker` Rust binary has no dedup/identity logic at all (verified via grep), so rebuilding it wouldn't have changed anything. The actual fix lives in `enqueue-learning-job.py`, which hooks execute from a **signed generation snapshot** (`~/.prometheus/plugins/prometheus-skill-pack/current`), not live from the repo — and that snapshot predated my fix. I regenerated and deployed a new signed generation (`ff17bca0...`, commit `3f64cb4d`), verified manifests (27/27) and the target-receipt matrix, and confirmed via a live re-run of the exact duplicate-reproduction test that the deployed script now correctly collapses the twin-hook duplicate.

Want this pushed too?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 69948acc-ad6d-4a8c-a02a-ba8a453e0345
- Captured: 2026-08-26T18:27:36.463657Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-1b1dfa94622bc16b.md
- .prometheus/knowledge/wiki/karpathy-session-2147406b572230eb.md
- .prometheus/knowledge/wiki/karpathy-session-7b406a72fa980eb7.md
- .prometheus/knowledge/wiki/karpathy-session-8017ce6415a17a44.md
- .prometheus/knowledge/wiki/karpathy-session-92be3ebc04aa11ba.md
- .prometheus/knowledge/wiki/karpathy-session-96b197bd660fb49e.md
- .prometheus/knowledge/wiki/karpathy-session-d8f51b4893ffc91b.md
- .prometheus/knowledge/wiki/karpathy-session-e68e38c02d6bb735.md
- crates/prometheus-exec/.prometheus/
