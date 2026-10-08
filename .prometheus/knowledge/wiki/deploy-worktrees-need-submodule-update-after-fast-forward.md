---
type: Lesson
id: deploy-worktrees-need-submodule-update-after-fast-forward
title: Deploy Worktrees Need Submodule Update After Fast-Forward
tags:
- vis:project
- kbd:reflect
- phase:team-aware-learning-memory-impl
- deploy-worktree
- git-submodules
- update-skill-pack
- delivery-cadence
- team-aware-learning-memory
- checkpoint-guards
links:
- delivery-cadence-source-freezing-requires-deploy-worktrees
sources:
- id: learning
  resource: learning:90adeb6bc53b712e
generated:
  by: pk/1.11.0
  at: 2026-10-04T22:37:20.047745+00:00
created_at: 2026-10-04T22:37:20.047745+00:00
updated_at: 2026-10-04T22:37:20.047745+00:00
revision: 0
content_hash: 11a9e0fbc90a237094245066cd04a31c15a8c7b0a0d619ef6f0c8d1f9deb0a2d
---

## Lesson

After fast-forwarding a dedicated deploy worktree, run `git submodule update` before invoking `update-skill-pack.sh`; otherwise the script can reject the checkout as a dirty tree.[^learning]

## Operational Guidance

- For cadence-managed deploy worktrees, apply this sequence after a fast-forward:

```bash
git pull --ff-only
git submodule update
./update-skill-pack.sh
```

- Treat submodule pointer mismatch after a fast-forward as deploy-worktree maintenance, not as an intentional source change.
- This refines the deploy-worktree practice described in [Delivery Cadence Source Freezing Requires Deploy Worktrees](/delivery-cadence-source-freezing-requires-deploy-worktrees.md): keeping the deploy worktree clean includes synchronizing submodules after updating the superproject.

[^learning]: reflection lesson from `team-aware-learning-memory-impl`