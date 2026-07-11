---
id: change-cpd-005-env-provisioning-helper
title: "Env-provisioning helper: seed ~/.codex/config.toml env for the 7 MCP servers from the environment"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-04]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - scripts/
  - docs/codex-plugin.md
---

# change-cpd-005-env-provisioning-helper

**Objective.** Let a fresh machine provision the plugin's 7 MCP servers' keys/tokens from the environment, no committed secrets.

## Tasks

- [x] Provide a helper (new script or extend configure-mcp-all-tools.sh) seeding ~/.codex/config.toml env for TAVILY_API_KEY, FORGE_MCP_TOKEN, etc. from the environment
- [x] Reuse the tavily/firecrawl setup pattern; bash-3.2 safe if launchd-invoked; NEVER commit secret values
- [x] Document usage in docs/codex-plugin.md
- [x] Verify: after running it, `codex doctor` stops warning for the provided keys
