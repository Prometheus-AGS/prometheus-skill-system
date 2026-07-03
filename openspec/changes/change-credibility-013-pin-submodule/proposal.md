---
id: change-credibility-013-pin-submodule
title: Pin sycophancy-correction submodule to a stable commit SHA
phase: phase-credibility-closure
priority: P2
effort: XS
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-F
verdict: BUILD
scope:
  - skills/imported/sycophancy-correction
  - .gitmodules
---

# change-credibility-013 — Pin sycophancy-correction submodule to a stable commit SHA

## Context

The `skills/imported/sycophancy-correction` submodule is currently on a floating HEAD. If the upstream repo pushes a breaking change, the next `git submodule update --remote` will silently break the sycophancy gate and the reflector hook.

The correct pattern is to pin to a known-good commit SHA (detached HEAD) and only advance the pin deliberately after validation.

## Scope

1. Navigate to `skills/imported/sycophancy-correction`
2. Run `git log --oneline -5` to pick the most recent stable commit
3. Check out that specific SHA (`git checkout <sha>`)
4. `cd ../..` and commit the submodule pointer update
5. Add a comment to `.gitmodules` noting the pin policy

## Implementation Notes

```bash
cd skills/imported/sycophancy-correction
git log --oneline -5   # pick the HEAD or last known-good SHA
SHA=$(git rev-parse HEAD)
git checkout $SHA
cd ../../..
git add skills/imported/sycophancy-correction
git status  # shows "modified: skills/imported/sycophancy-correction (new commits)"
```

The submodule is now pinned. To advance: repeat the process after validating the new SHA manually.

Add to `.gitmodules` as a comment:
```
# Pin policy: advance SHA only after validating sycophancy gate in CI
```

## Verification

- `git submodule status` shows a specific SHA (not `+` prefix which means dirty/uncommitted)
- The SHA matches the chosen commit
- After a `git submodule update --init` the correct commit is checked out
