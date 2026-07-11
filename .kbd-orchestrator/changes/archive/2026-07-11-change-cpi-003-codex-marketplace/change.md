---
id: change-cpi-003-codex-marketplace
title: "Emit .agents/plugins/marketplace.json (Codex source/policy schema) for the 11 plugins"
phase: phase-codex-plugin-implementation
gaps: [G-03]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-002-plugin-manifest]
scope:
  - .agents/plugins/marketplace.json
  - scripts/build-codex-plugin.js
  - docs/
---

# change-cpi-003-codex-marketplace

**Objective.** Produce a native Codex marketplace transforming the 11 Claude plugins; keep .claude-plugin/marketplace.json as the documented legacy fallback.

## Tasks

- [x] Transform each of the 11 plugins: Claude `source:"."` (path string) → Codex `source.source`(local|git-subdir)+`source.path` (./-relative)
- [x] Add per-plugin `policy.installation` (AVAILABLE / INSTALLED_BY_DEFAULT) and `policy.authentication`, plus `category`
- [x] Add top-level `name` + `interface.displayName`; write repo `.agents/plugins/marketplace.json`
- [x] Document personal-scope install (~/.agents/plugins/marketplace.json) and the .claude-plugin legacy-read fallback
- [x] Verify `codex plugin marketplace add .` lists the plugin(s) without error
