---
id: change-cpi-007-build-tooling
title: "Generator + validator + installer integration for the Codex plugin & marketplace"
phase: phase-codex-plugin-implementation
gaps: [G-07]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-002-plugin-manifest, change-cpi-003-codex-marketplace, change-cpi-004-plugin-mcp, change-cpi-005-hooks-wiring, change-cpi-006-skills-bundle]
scope:
  - scripts/build-codex-plugin.js
  - scripts/validate-skills.js
  - scripts/install-platforms.ts
  - package.json
---

# change-cpi-007-build-tooling

**Objective.** Make the Codex artifacts reproducibly generated + validated, wired into the existing build/install flow (mirrors build-marketplace.js).

## Tasks

- [x] Add a `build:codex`-equivalent that regenerates .codex-plugin/plugin.json, .agents/plugins/marketplace.json, plugin .mcp.json from source — idempotent
- [x] Add validation (manifest fields, path-in-root, marketplace schema) analogous to validate-skills.js; wire into npm scripts
- [x] Integrate install-platforms.ts codex target + codex-sync-skills.sh so install provisions the plugin
- [x] Run the generator twice; assert no diff (idempotent) and artifacts validate
