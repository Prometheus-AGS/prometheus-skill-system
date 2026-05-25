# change-install-004-wire-mcp-all-tools

**Phase**: machine-installation-2026-05-25  
**Status**: PENDING  
**Gaps closed**: G-MCP-1, G-MCP-2, G-MCP-3  
**Depends on**: change-install-002

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
