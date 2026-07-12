# Remote Marketplace Resolution — change-cpv-006 (quick test)

_2026-07-12. User chose the non-destructive quick remote test over a full git-subdir publish._

## Verified ✅

`codex plugin marketplace add https://github.com/Prometheus-AGS/prometheus-skill-system`
cloned the pushed repo to `~/.codex/.tmp/marketplaces/prometheus-skill-pack` and
**resolved all 11 plugins** from the remote clone, each mapped to its correct
subdir (umbrella `.`, `skills/react/...`, `skills/process`, ..., `tools/disk-space-guardian`,
`substrate/prometheus-research`). Exit 0. Cleaned up afterward (codex clean).

So the committed marketplace resolves **remotely from a GitHub URL**, not just via
local `marketplace add .`.

## Deferred (by user choice)

The full **git-subdir** publish path — `CODEX_MARKETPLACE_SOURCE=git-subdir` +
committing/publishing a git-subdir marketplace, then resolving its
`{source:git-subdir,url,ref,path}` entries — was **not** exercised. The generator
supports it (verified byte-for-byte last phase); publishing it is a future decision.
Note: the committed marketplace uses `local` sources, which resolved fine after the
remote clone; a git-subdir publish would matter for referencing pinned commits /
monorepo subdirs without a full clone.
