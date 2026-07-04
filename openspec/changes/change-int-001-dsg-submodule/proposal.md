---
id: change-int-001-dsg-submodule
title: Add disk-space-guardian as git submodule at tools/disk-space-guardian
phase: cowork-integration
priority: P1
effort: S
wave: 5
agent: general-purpose
status: done
gap_id: G-05-dsg
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - .gitmodules (add tools/disk-space-guardian entry)
  - docs/SUBMODULES.md (document dsg entry)
---

# change-int-001 — Add dsg as git submodule

## Context

disk-space-guardian (dsg) is a Rust CLI already implemented and tested. It needs
to be registered as a git submodule at `tools/disk-space-guardian` so that:
1. `install-binaries.sh` can build it from source on any machine
2. The skill-pack pins to a known-good dsg commit
3. The integration layer (change-int-002 through int-004) has a local source

## Strategy

Add the submodule pointing at the GQAdonis/disk-space-guardian GitHub repo,
then document it in docs/SUBMODULES.md with its purpose and pin policy.

## Scope

1. Add `tools/disk-space-guardian` to .gitmodules (via git submodule add or
   direct edit if the repo is already local)
2. Update docs/SUBMODULES.md with dsg entry
3. Verify `git submodule status` shows entry
4. Update KBD orchestrator
5. Commit

## Verification

- `.gitmodules` contains `tools/disk-space-guardian` entry
- `docs/SUBMODULES.md` documents dsg
- `git submodule status` shows the entry (may show `-` prefix if not initialized)
