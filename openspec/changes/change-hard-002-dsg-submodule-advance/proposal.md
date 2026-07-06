# change-hard-002-dsg-submodule-advance

**Status**: done

## Summary

Advance the `tools/disk-space-guardian` submodule pointer from v0.1.3
(`b7d8f30`) to v0.1.4 (`abe2e1c`), and make an optional minor SKILL.md update.

## Problem

The v0.1.4 release fixed the macOS Intel (`x86_64-apple-darwin`) CI runner
issue (switched from congested `macos-13` to `macos-latest`). All 4 release
artifacts are now present and correct. The submodule pointer in the skill-pack
still references the v0.1.3 commit, which is misleading for users who
`git submodule update --init` and expect Path B to resolve to the latest version.

## Fix

1. Update `tools/disk-space-guardian` to track the v0.1.4 tag commit (`abe2e1c`)
2. Optional: add a "requires v0.1.4+" note to the SKILL.md install section

## Files changed

- `tools/disk-space-guardian` (submodule pointer — `.gitmodules` unchanged)
- `skills/devops/disk-space-guardian/SKILL.md` (optional one-line note)

## Acceptance criteria

- [ ] `git submodule status tools/disk-space-guardian` shows `abe2e1c` (v0.1.4)
- [ ] Committed on the worktree branch
- [ ] `npm run validate:strict skills/devops/disk-space-guardian` passes (0 errors)
