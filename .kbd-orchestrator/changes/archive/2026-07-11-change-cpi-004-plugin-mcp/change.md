---
id: change-cpi-004-plugin-mcp
title: "Emit plugin .mcp.json for the 7 MCP servers with the env strategy from the spike"
phase: phase-codex-plugin-implementation
gaps: [G-05]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-001-runtime-spike]
scope:
  - .mcp.json (source)
  - .codex-plugin/ or root .mcp.json for the plugin
  - scripts/build-codex-plugin.js
---

# change-cpi-004-plugin-mcp

**Objective.** Wire all 7 prometheus MCP servers into the plugin's MCP config in Codex format, applying the env approach 001 verified and carrying this session's server fixes.

## Tasks

- [x] Emit plugin .mcp.json (direct map or mcp_servers wrapper) for surreal-memory, sycophancy-correction, forge-rs, prometheus-knowledge, liter-llm, tavily, sequential-thinking
- [x] Apply the env strategy from 001: inline `env` if honored, else document ~/.codex/config.toml env-passthrough; NEVER commit secrets (keys from env)
- [x] Carry session fixes: forge FORGE_MCP_TOKEN bearer, liter-llm proxy config + stdio_trust_local, tavily name-collision (non-'tavily' key)
- [x] Verify each server initializes when the plugin is active
