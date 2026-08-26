---
type: SessionRecord
id: karpathy-session-7b406a72fa980eb7
title: Karpathy session 7b406a72fa98
tags:
- karpathy
- session-learning
sources:
- session:69948acc-ad6d-4a8c-a02a-ba8a453e0345
timestamp: 2026-08-26T18:14:28.422050+00:00
created_at: 2026-08-26T18:14:28.422050+00:00
updated_at: 2026-08-26T18:14:28.422050+00:00
revision: 0
---

## Delta

Fixed and committed (`f1aabc79`). Root cause: the earlier dedup fix (`e402abd7`) keyed on raw transcript byte size, assuming it's identical between the twin `stop`/`executor_complete` hook firings — but other hooks sharing the Stop event append lines to the same transcript in the ~1s gap between them, so the byte count moves in the common case. Reproduced it exactly against this session's own transcript (2771786 vs 2786050 bytes at the two real capture times) and verified the new fix — anchoring on the last assistant-message UUID instead — collapses both to the same identity while still advancing for genuinely new turns. Fail-open fallback to the old byte-size behavior is preserved if the transcript is unreadable.

Want this pushed to `origin/main`?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 69948acc-ad6d-4a8c-a02a-ba8a453e0345
- Captured: 2026-08-26T18:14:26.961962Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-1b1dfa94622bc16b.md
- .prometheus/knowledge/wiki/karpathy-session-8017ce6415a17a44.md
- .prometheus/knowledge/wiki/karpathy-session-96b197bd660fb49e.md
- .prometheus/knowledge/wiki/karpathy-session-d8f51b4893ffc91b.md
- crates/prometheus-exec/.prometheus/
- shared/scripts/__pycache__/
