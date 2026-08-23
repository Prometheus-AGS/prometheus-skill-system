## Why

20 `.windsurf/skills` files are deleted, but the evidence contradicts a simple 'the CLI
stopped emitting it': `openspec init --tools` still lists windsurf, `skill-system.json:144`
declares `.windsurf/skills` a managed symlink harness, `.windsurf/workflows` keeps 19
tracked files, and sibling harnesses kept their skills trees. Nothing yet explains why only
this harness lost its skills.

## What Changes

- DECIDE (blocks the rest): (A) the deletion is a bug — regenerate `.windsurf/skills`;
  or (B) Windsurf is retired — remove `skill-system.json:144` and every other reference.
- Under (B), re-run the consumers of `skill-system.json` and commit their output.

## Impact

- `.windsurf/workflows` (19 files) is out of scope either way.
- Under (B) this alters the declared harness set that c404 reads.
