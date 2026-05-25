# Tasks: change-install-004-wire-mcp-all-tools

- [ ] Read `.mcp.json` at repo root to confirm server names and configs
- [ ] Probe `~/.config/opencode/` — find config file (config.json, opencode.json, config.toml)
- [ ] Read existing opencode config and merge `mcpServers` block without removing existing entries
- [ ] Probe `~/.codex/` — find config file (config.yaml, config.json)
- [ ] Read existing codex config and add `mcpServers` block
- [ ] Probe `~/.config/zed/settings.json` — confirm format
- [ ] Read existing zed settings and merge `context_servers` block
- [ ] Verify opencode config contains `surreal-memory` entry
- [ ] Verify codex config contains `surreal-memory` entry
- [ ] Verify zed settings contain MCP entries
