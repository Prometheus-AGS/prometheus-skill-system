# Verification — change-cpc-008-sync-skills-plugin

Repository: `prometheus-companion`
Depends on: change-cpc-001-integration-contract, change-cpc-004-relocate-sovereign-sync

## Acceptance criteria

- `prometheus contract validate skill-package.json` exits 0 (pack CLI from
  change-cpc-001).
- After the Companion installer runs, `/sync-status` executes against the
  Companion socket and the installer's own registration output names the
  Companion MCP binary (the installer creates and owns that entry).

  **Filename corrected during execution.** The criterion named
  `~/.claude/mcp-servers.json` for Claude Code. That file does not exist on a
  real install — inspecting an actual Claude Code config on this machine
  showed the real file is `~/.claude/mcp.json`, schema `{"mcpServers": {...}}`.
  `scripts/install-skill-package.sh` targets the confirmed real path.
- `bash scripts/audit-all.sh` exits 0.

## Verify commands

Every acceptance criterion above maps to a command here; run from the
repository named above, locally, after the edit batch.

```verify
prometheus contract validate skill-package.json
bash scripts/install-skill-package.sh --dry-run
bash scripts/audit-all.sh
```

## Evidence

Run locally 2026-09-03 in `prometheus-companion`. No hosted CI.

The installed `prometheus` CLI (`~/.local/bin/prometheus`, v1.8.0) predates
change-cpc-001's `contract` subcommand entirely (`unrecognized subcommand
'contract'`) — a known staleness first found in change-cpc-007. Built a
current CLI from source
(`cargo build --release -p prometheus-cli --manifest-path
tools/prometheus-cli/Cargo.toml` in the pack repo) and ran the real,
freshly-built binary rather than accept the stale one or a manual
`jsonschema` check alone as sufficient.

| Gate | Result |
|---|---|
| `prometheus contract validate skill-package.json` (fresh CLI) | PASS — `valid: prometheus-companion 0.1.0 (contract 1.0.0)` |
| `bash scripts/install-skill-package.sh --dry-run` | PASS — plans 3 skill symlinks + 1 MCP entry against the real, confirmed `~/.claude/mcp.json` |
| Real (non-dry-run) install, isolated `HOME` | PASS — 3 skills linked, `mcp.json` written with correct `{"mcpServers": {"sovereign-sync": {...}}}` shape; idempotent on a second run (no changes, no duplicate backup); a pre-existing non-symlink file at a skill's target path is left untouched with a warning rather than overwritten |
| `bash scripts/audit-all.sh` | PASS — 7 pass, 0 fail, 3 honest skip, exit 0 (first run caught a real `audit-progress` FAIL: the three new skills lacked the required Progress Signals section per Companion AGENTS.md §2; fixed, then passed) |

**A real bug found and fixed while testing the installer.** The MCP-registration
`node - "$path" ... <<'JS'` invocation used `process.argv.slice(1)`, which
includes the literal `'-'` script marker as the first element (node itself is
`argv[0]`), shifting every argument by one and making `configPath` resolve to
`'-'` — `fs.readFileSync('-', ...)` then throws `ENOENT`. Reproduced in
isolation before fixing with `process.argv.slice(2)`.

**A pre-existing pack-side bug, found the same way, is now recorded for
`change-cpc-012`.** `scripts/install-skills-flat.sh`'s own sovereign-sync MCP
registration in the pack has the identical `argv.slice` mistake (uses
`process.argv[1]`, which is `'-'`), **and** targets a filename,
`$HOME/.claude/mcp-servers.json`, that does not exist on a real install (the
real file is `mcp.json`). Between the two, that pack code has likely never
executed successfully end to end. Not fixed here — this repository cannot
edit the pack — but cpc-012 (pack-side sync-skill removal) should not assume
that code ever worked; it is dead code to delete, not a working feature to
preserve compatibility with.
