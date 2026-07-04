---
id: change-push-002-submodule-pointer
title: Advance tools/cowork-skills submodule pointer to v0.2.0
phase: phase-cowork-push-and-release
priority: P0
effort: XS
wave: 2
agent: general-purpose
status: pending
gap_id: G-03
verdict: BUILD
scope:
  - prometheus-skill-pack worktree (claude/charming-diffie-309eef branch)
  - tools/cowork-skills (gitlink — submodule pointer)
---

# change-push-002 — Advance submodule pointer to v0.2.0

## Context

The tools/cowork-skills gitlink currently points to 53e6b31 (upstream v0.1.5).
After change-push-001 pushes and tags v0.2.0, the remote has the new SHA.
This change fetches it and updates the pointer in the skill-pack.

## Strategy

1. `git -C tools/cowork-skills fetch --tags`
2. `git -C tools/cowork-skills checkout v0.2.0` (detached HEAD at tag)
3. `git add tools/cowork-skills`
4. `git commit` in skill-pack with the new pointer
5. `git submodule status tools/cowork-skills` — confirm new SHA shown

## Scope

1. Fetch tags in cowork-skills submodule
2. Checkout v0.2.0 tag
3. Stage and commit submodule pointer in skill-pack
