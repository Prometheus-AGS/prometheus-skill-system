## Why

The parent pins `prometheus-entity-management` at 1c40eaa; the checkout has moved twice
during this phase (55dc8dc, then 4485696) on branch `main-takeover-kimi`, and
`git branch -r --contains` places that commit on NO remote branch. Committing the pointer
would pin the parent to work a fresh clone cannot resolve. The submodule worktree was dirty
(52 files) when the phase opened and is clean as of planning — the facts move, so they must
be re-measured at execute time.

## What Changes

- DECIDE: (A) publish the submodule branch then re-pin to a resolvable commit, or
  (B) restore the parent's pin to 1c40eaa and leave the submodule checkout alone.
- (C) 'leave it dirty' is rejected: it blocks c405, which is this phase's purpose.

## Impact

- Blocks c405: `require_clean_source` uses plain `git status --porcelain`, which reports a
  submodule's modified content independently of its pointer.
- (A) needs the submodule repository's owner.
