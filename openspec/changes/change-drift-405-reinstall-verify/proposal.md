## Why

The phase exists to unblock `update-skill-pack.sh`, which refuses a dirty source tree and
so cannot ship the merged preflight-models.sh fix (PR #55, c0d2de1) to installed surfaces.
With c400-c404 landed the tree is clean and the reinstall can run.

## What Changes

- Land c400-c404 to main, pull, PUSH, then run `update-skill-pack.sh --force`.
- Verify the installed script carries the fix, and that it works with no env crutch.

## Impact

- Terminal barrier: needs a fully clean tree and is the only outward-facing change.
- `require_clean_source` runs FIVE times; a step that regenerates files mid-run re-dirties
  the tree and aborts a later check, so commit between stages.
