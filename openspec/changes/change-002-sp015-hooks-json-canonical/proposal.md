## Why

Two paths resolve to `hooks.json`:
- `hooks/hooks.json` — physical file (canonical)
- `.claude-plugin/hooks/hooks.json` — resolves via directory symlink (`.claude-plugin/hooks → ../hooks`)

The task doc (SP-015) proposed making `hooks/hooks.json` a symlink to `.claude-plugin/hooks/hooks.json`. Assessment found the actual state is the inverse: the directory-level symlink already prevents drift. However:

1. The SP-015 acceptance criteria (`hooks/hooks.json` is a symlink) are not met.
2. There is no CI check confirming the symlink relationship, so future `npm run build` regressions could silently break it.
3. The canonical path decision (which of the two is authoritative for the Claude Code runtime) has not been explicitly documented.

This change validates the existing setup is correct, adds the CI guard, and documents the canonical path decision explicitly.

## What Changes

- Read `plugin.json` to confirm which path the Claude Code runtime reads hooks from.
- If `hooks/hooks.json` (physical file) is correct canonical: document this, add CI check that `.claude-plugin/hooks` is a symlink (`test -L .claude-plugin/hooks`).
- If `.claude-plugin/hooks/hooks.json` should be canonical: swap — make `hooks/hooks.json` a symlink to `../.claude-plugin/hooks/hooks.json` and update the `npm run build` script to maintain it.
- Add one-line CI check to `.github/workflows/validate.yml`.
- Update any documentation referencing the dual-file pattern.

## Capabilities

### New Capabilities
- `hooks-json-integrity-check`: CI assertion that the hooks.json symlink relationship is intact on every PR.

### Modified Capabilities

## Impact

- `plugin.json` — read-only inspection
- `.github/workflows/validate.yml` — add one-line symlink check
- `hooks/hooks.json` or `.claude-plugin/hooks` — one becomes a symlink (direction TBD by plugin.json inspection)
- `docs/` — update any doc mentioning the dual-file pattern
- No runtime behavior change; purely structural
