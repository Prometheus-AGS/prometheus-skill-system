---
id: change-cpv-005-env-roundtrip
title: "Confirm the MCP env round-trip clears codex doctor for a keyed server"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-02]
priority: P2
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - .kbd-orchestrator/phases/phase-codex-plugin-verify-and-publish/references/env-roundtrip.md
---

# change-cpv-005-env-roundtrip

**Objective.** Empirically confirm codex-provision-mcp-env.sh makes a plugin MCP server see its key.

## Tasks

- [x] Source keys from ~/.bash_profile (TAVILY_API_KEY, FORGE_MCP_TOKEN); run scripts/codex-provision-mcp-env.sh
- [x] Install the plugin; run `codex doctor` — confirm the env-var warning clears for the provided keys (or a keyed server initializes)
- [x] If inherit=all doesn't forward to MCP servers, fall back to per-server inline env + document
- [x] Record references/env-roundtrip.md; clean up ~/.codex
