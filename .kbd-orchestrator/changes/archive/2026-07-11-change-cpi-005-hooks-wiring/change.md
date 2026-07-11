---
id: change-cpi-005-hooks-wiring
title: "Wire plugin.json.hooks -> hooks/hooks.json (same PascalCase) + trust docs"
phase: phase-codex-plugin-implementation
gaps: [G-06]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-001-runtime-spike, change-cpi-002-plugin-manifest]
scope:
  - .codex-plugin/plugin.json
  - hooks/hooks.json
  - docs/
---

# change-cpi-005-hooks-wiring

**Objective.** Point the plugin at the existing PascalCase hooks/hooks.json; scope determined by the spike's hook-firing verdict.

## Tasks

- [x] Point plugin.json `hooks` at hooks/hooks.json (Codex plugin hooks share Claude's PascalCase event schema)
- [x] If 001 showed plugin hooks fire once trusted: document the non-managed trust flow (${PLUGIN_ROOT}/${PLUGIN_DATA}); ensure hook commands are PLUGIN_ROOT-relative
- [x] If 001 showed they DON'T fire: ship hooks.json but document as pending-upstream (mirror CLAUDE.md 'not yet ported' note) — do NOT claim a working hook surface
- [x] Update CLAUDE.md Codex section with the verified hook behavior
