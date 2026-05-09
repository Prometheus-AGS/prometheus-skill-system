# Execution — change-002-sp015-hooks-json-canonical

**Executed:** 2026-05-09  
**Backend:** OpenSpec  
**Agent role:** skill-pack-maintainer  
**Executor:** claude-sonnet-4-6

## Dispatch

Structural + CI change. plugin.json inspection first, then CI guard, then documentation.

## Canonical Path Decision

`plugin.json` (`.claude-plugin/plugin.json`) declares:
```json
"hooks": "./hooks/hooks.json"
```
This path is relative to `.claude-plugin/`. It resolves via the directory symlink
`.claude-plugin/hooks → ../hooks` to the physical file `hooks/hooks.json`.

**Verdict**: `hooks/hooks.json` is the canonical physical file. The existing setup
(directory symlink at `.claude-plugin/hooks`) is correct and no swap is needed.
All hook edits must target `hooks/hooks.json` directly.

## Files Modified

1. `.github/workflows/validate.yml`
   - Added `hooks-integrity` job with two assertions:
     - `test -L .claude-plugin/hooks` — symlink must exist
     - `test -f hooks/hooks.json` — canonical file must exist
   - Job runs on every push to main and every PR.

2. `CLAUDE.md` (skill-pack root)
   - Added one paragraph under the Dual-Format Support section documenting:
     - `hooks/hooks.json` is canonical
     - `plugin.json` resolves through the directory symlink
     - Always edit `hooks/hooks.json` directly
     - CI validates symlink via `hooks-integrity` job

## Skipped

- No symlink direction swap needed (existing setup is correct).
- No docs referencing a "dual-file" anti-pattern found; no additional doc cleanup needed.

## QA Gate

Applied: 2 files modified (≥3 threshold not met, doc-only + CI config). Skipped.

## Status

DONE
