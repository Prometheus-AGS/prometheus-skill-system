# change-sync-018: Workspace Cargo.toml + version bump + CLAUDE.md

**Phase:** phase-learn-sovereign-sync
**Tier:** 4 (after Tier 3, parallelize with 016 and 017)
**Status:** pending
**Gap:** general housekeeping

## Summary

Wire up the workspace, bump package versions to 1.5.0, and extend CLAUDE.md
with the sovereign-sync documentation section.

## Files to change

- `substrate/Cargo.toml` or root `Cargo.toml` — add sovereign-sync and sovereign-client to workspace members
- `package.json` — version 1.4.0 → 1.5.0
- `plugin.json` — version bump to match
- `CLAUDE.md` — add sovereign-sync section

## CLAUDE.md additions

New section: "Sovereign Sync (substrate/sovereign-sync)"
- Three binary modes table (mcp, daemon, server)
- UAR co-existence guide (env var detection, prefix-tools flag)
- BossFang MCP integration instructions
- Port usage table: 7890 surface-bridge, 7892 sovereign-sync daemon/server, 7891 reserved
- Config file location

## Tasks

- [ ] Check if root Cargo.toml exists; if not, check substrate/Cargo.toml
- [ ] Add sovereign-sync and sovereign-client to workspace members
- [ ] Bump version in package.json and plugin.json
- [ ] Write CLAUDE.md additions (read file first)
- [ ] `cargo check` in workspace — all crates resolve cleanly
