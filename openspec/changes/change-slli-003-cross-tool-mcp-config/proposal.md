# change-slli-003-cross-tool-mcp-config

**Phase**: self-learning-loop-integration
**Status**: DONE
**Priority**: 3 of 10
**Depends on**: change-slli-002
**Gaps closed**: MCP-CFG-1 through MCP-CFG-7

## Summary

Write all 7 MCP server entries into the native config files for ALL 7 supported AI tools. Builds on `change-install-004-wire-mcp-all-tools` (covers opencode, codex, zed for 5 servers) and extends it to 7 tools × 7 servers, using the launchd-hosted port endpoints from change-slli-002.

## Files Modified

### Claude Code: `~/.claude/settings.json`
Verify/update existing entries to use launchd-hosted SSE URLs. Add sequential-thinking and tavily if missing with launchd ports.

### OpenCode: `~/.opencode/config.json` (or `~/.config/opencode/config.json`)
```json
{
  "mcp": {
    "surreal-memory": {"type": "sse", "url": "http://localhost:23001/mcp/sse"},
    "prometheus-knowledge": {"type": "sse", "url": "http://localhost:8942/mcp"},
    "forge-rs": {"type": "sse", "url": "http://localhost:8943/mcp"},
    "sycophancy-correction": {"type": "sse", "url": "http://localhost:8944/mcp"},
    "liter-llm": {"type": "sse", "url": "http://localhost:8945/mcp"},
    "sequential-thinking": {"type": "sse", "url": "http://localhost:8946/mcp"},
    "tavily": {"type": "sse", "url": "http://localhost:8947/mcp"}
  }
}
```

### Codex: `~/.codex/config.yaml`
```yaml
mcpServers:
  surreal-memory:
    type: sse
    url: "http://localhost:23001/mcp/sse"
  prometheus-knowledge:
    type: sse
    url: "http://localhost:8942/mcp"
  forge-rs:
    type: sse
    url: "http://localhost:8943/mcp"
  sycophancy-correction:
    type: sse
    url: "http://localhost:8944/mcp"
  liter-llm:
    type: sse
    url: "http://localhost:8945/mcp"
  sequential-thinking:
    type: sse
    url: "http://localhost:8946/mcp"
  tavily:
    type: sse
    url: "http://localhost:8947/mcp"
```

### Kimi Code: `~/.kimi-code/config.toml`
```toml
[[mcp_servers]]
name = "surreal-memory"
type = "sse"
url = "http://localhost:23001/mcp/sse"

[[mcp_servers]]
name = "prometheus-knowledge"
type = "sse"
url = "http://localhost:8942/mcp"

# ... (repeat for all 7 servers)
```

### MiniMax: `~/.minimax/mcp/mcp.json`
```json
{
  "mcpServers": {
    "surreal-memory": {"type": "sse", "url": "http://localhost:23001/mcp/sse"},
    "prometheus-knowledge": {"type": "sse", "url": "http://localhost:8942/mcp"},
    "forge-rs": {"type": "sse", "url": "http://localhost:8943/mcp"},
    "sycophancy-correction": {"type": "sse", "url": "http://localhost:8944/mcp"},
    "liter-llm": {"type": "sse", "url": "http://localhost:8945/mcp"},
    "sequential-thinking": {"type": "sse", "url": "http://localhost:8946/mcp"},
    "tavily": {"type": "sse", "url": "http://localhost:8947/mcp"}
  }
}
```

### Cursor: `~/.cursor/mcp.json`
Same structure as MiniMax.

### Windsurf: `~/.codeium/windsurf/mcp/mcp.json`
Same structure as MiniMax.

## Files Created

- `scripts/configure-mcp-all-tools.sh` — idempotent writer:
  1. Reads service port table from `scripts/mcp-port-table.json`
  2. For each tool, detects config file path
  3. Uses `jq` (JSON) or `dasel` (TOML/YAML) to merge MCP entries without overwriting existing keys
  4. Reports update table: tool → config path → servers added/verified
- `scripts/mcp-port-table.json` — source of truth for service names, ports, and protocols:
  ```json
  {
    "surreal-memory":        {"port": 23001, "path": "/mcp/sse", "type": "sse"},
    "prometheus-knowledge":  {"port": 8942,  "path": "/mcp",     "type": "sse"},
    "forge-rs":              {"port": 8943,  "path": "/mcp",     "type": "sse"},
    "sycophancy-correction": {"port": 8944,  "path": "/mcp",     "type": "sse"},
    "liter-llm":             {"port": 8945,  "path": "/mcp",     "type": "sse"},
    "sequential-thinking":   {"port": 8946,  "path": "/mcp",     "type": "sse"},
    "tavily":                {"port": 8947,  "path": "/mcp",     "type": "sse"}
  }
  ```
- `install-skills-flat.sh` updated: call `configure-mcp-all-tools.sh` at end of every install run

## Acceptance Criteria

- Each of the 7 tool configs contains all 7 MCP server entries
- No existing entries deleted or corrupted
- `configure-mcp-all-tools.sh` is idempotent (run twice = same result)
- Script detects missing tool installs gracefully (skips if config dir does not exist)
