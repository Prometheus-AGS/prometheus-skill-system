---
id: change-cpi-001-runtime-spike
title: "Spike: verify Codex plugin runtime (install, non-managed hooks, .mcp.json env) in codex-cli 0.144.1"
phase: phase-codex-plugin-implementation
gaps: [G-01, G-05, G-06]
priority: P0
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - .kbd-orchestrator/phases/phase-codex-plugin-implementation/references/runtime-spike-findings.md
  - /tmp (throwaway plugin)
---

# change-cpi-001-runtime-spike

**Objective.** De-risk the two empirical unknowns before committing 004/005 scope. Build a minimal throwaway Codex plugin and observe real behavior in codex-cli 0.144.1.

## Tasks

- [x] Build a minimal throwaway plugin: .codex-plugin/plugin.json + .agents/plugins/marketplace.json + one skill dir + a no-op PascalCase hook + a one-server .mcp.json that sets an env var
- [x] codex plugin marketplace add <path> && codex plugin install <name>; confirm install + the skill appears in a fresh session
- [x] Trust and invoke the plugin hook; record whether non-managed plugin hooks actually FIRE (vs the config.toml [hooks] no-op)
- [x] Confirm whether plugin .mcp.json honors `env` (server sees the var) or requires ~/.codex/config.toml env-passthrough
- [x] Write references/runtime-spike-findings.md with verdicts that gate 004 (mcp env strategy) and 005 (hooks scope); clean up the throwaway plugin
