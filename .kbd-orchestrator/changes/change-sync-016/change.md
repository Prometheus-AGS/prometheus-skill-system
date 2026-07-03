# change-sync-016: install-skills-flat.sh extension

**Phase:** phase-learn-sovereign-sync
**Tier:** 4 (after Tier 3)
**Status:** pending
**Gap:** G-03, G-09

## Summary

Extend the existing `scripts/install-skills-flat.sh` to: compile the
sovereign-sync binary (if Rust toolchain available), install it, register
it as a launchd daemon, and inject MCP server entries into all 4 supported
harness configs (Claude Desktop, Kimi, OpenCode, Codex).

## Files to change

- `scripts/install-skills-flat.sh` — extend (DO NOT rewrite from scratch)

## Harness MCP registration

| Harness | Config file | Format |
|---------|-------------|--------|
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` | JSON mcpServers |
| Kimi Desktop | `~/.kimi-code/config.toml` | TOML [[mcp_servers]] |
| OpenCode | `~/.config/OpenCode/mcp.json` | JSON mcpServers |
| Codex | `~/.codex/mcp.json` | JSON mcpServers |
| MiniMax | skip — no MCP config support |

## UAR co-existence

Detect UAR process on port 8080 or `UAR_SKILL_SERVICE_URL` env var. If detected,
add `--prefix-tools sovereign:` to launchd plist and all MCP config entries.

## Tasks

- [ ] Read current install-skills-flat.sh
- [ ] Add Rust binary compile + install section (guard on `which cargo`)
- [ ] Write launchd plist for --mode daemon
- [ ] Add MCP registration for all 4 harnesses (idempotent — don't add twice)
- [ ] Add UAR detection + prefix-tools mode
- [ ] Test: run with --uninstall flag removes all entries cleanly
