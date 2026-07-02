---
id: change-install-004-wire-mcp-all-tools
title: Wire mcp all tools
phase: machine-installation-2026-05-25
gaps: [G-MCP-1, G-MCP-2, G-MCP-3]
depends_on: [change-install-002]
agent: claude-code
status: done
---

# change-install-004-wire-mcp-all-tools — Wire mcp all tools

## Summary

Wire all 5 MCP servers from the repo's `.mcp.json` into the native config files for opencode, codex, and zed.

## MCP Servers to Wire

| Name | Type | Endpoint/Command |
|---|---|---|
| `surreal-memory` | SSE | `http://localhost:23001/sse` |
| `sycophancy-correction` | stdio | `sycophancy-correction` |
| `forge-rs` | SSE | `http://localhost:8943/sse` |
| `prometheus-knowledge` | SSE | `http://localhost:8942/sse` |
| `liter-llm` | stdio | `liter-llm` |

## Files Modified

- `~/.config/opencode/config.json` (or equivalent) — `mcpServers` block
- `~/.codex/config.yaml` — `mcpServers` block
- `~/.config/zed/settings.json` — `context_servers` block

## Acceptance Criteria

- opencode config contains `surreal-memory` MCP entry
- codex config contains `surreal-memory` MCP entry
- zed settings contain at least one `context_servers` MCP entry
- No existing config entries corrupted or removed

## Tasks

- [x] 1. Read `.mcp.json` at repo root to confirm server names and configs
- [x] 2. Probe `~/.config/opencode/` — find config file (config.json, opencode.json, config.toml)
- [x] 3. Read existing opencode config and merge `mcpServers` block without removing existing entries
- [x] 4. Probe `~/.codex/` — find config file (config.yaml, config.json)
- [x] 5. Read existing codex config and add `mcpServers` block
- [x] 6. Probe `~/.config/zed/settings.json` — confirm format
- [x] 7. Read existing zed settings and merge `context_servers` block
- [x] 8. Verify opencode config contains `surreal-memory` entry
- [x] 9. Verify codex config contains `surreal-memory` entry
- [x] 10. Verify zed settings contain MCP entries
