---
type: Lesson
id: delivery-cadence-source-freezing-requires-deploy-worktrees
title: Delivery Cadence Source Freezing Requires Deploy Worktrees
tags:
- vis:project
- kbd:reflect
- phase:team-aware-learning-memory-impl
- delivery-cadence
- deploy-worktree
- source-freezing
- iteration-repair
- checkpoint-guards
links:
- karpathy-session-8167fd7a02c353d1
- karpathy-session-8a8cb7c5e1025bf6
sources:
- id: learning
  resource: learning:cf4a0d4d09608dcd
generated:
  by: pk/1.11.0
  at: 2026-10-04T22:37:12.032657+00:00
created_at: 2026-10-04T22:37:12.032657+00:00
updated_at: 2026-10-04T22:37:12.032657+00:00
revision: 0
content_hash: 3aea9bd64c0b8c03f539811e618cf6cd16690bca230dba6b5df868bb84f4b646
---

## Lesson

`delivery-cadence` freezes every source named in `ready`; do **not** point those sources at the actively edited main checkout. Use a dedicated deploy worktree instead, because normal bookkeeping or local writes in the main checkout can invalidate the frozen source and trip checkpoint guards.[^learning]

## Operational Constraints

- `ready` records the source state for the iteration.
- `finish` refuses unchanged sources, so an iteration cannot be completed by simply retrying without an admissible source-state change.[^learning]
- If `finish` fails, the only repair path is to run `ready` again for that **same iteration**.[^learning]
- Do not consume the iteration's `ready` against the main checkout; if that checkout changes after freeze, the iteration can become unrecoverable without same-iteration repair.

## Practice

- Maintain a clean deploy worktree, e.g. `deploy/main`, for cadence-managed sources.
- Run `ready` using the deploy worktree path, not the developer checkout.
- Keep developer tooling, KBD bookkeeping, and unrelated edits out of the frozen source tree.

This generalizes the failure mode recorded in [Karpathy session 8167fd7a02c3](/karpathy-session-8167fd7a02c353d1.md), where the cadence engine rejected a build checkpoint because the frozen source was the main checkout and changed after freeze, and the repair approach in [Karpathy session 8a8cb7c5e102](/karpathy-session-8a8cb7c5e1025bf6.md), where iteration 1 was re-readied from the deploy worktree.

[^learning]: Reflection lesson from `learning:cf4a0d4d09608dcd`.