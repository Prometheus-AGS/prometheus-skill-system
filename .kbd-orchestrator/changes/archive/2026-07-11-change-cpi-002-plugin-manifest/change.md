---
id: change-cpi-002-plugin-manifest
title: "Generate .codex-plugin/plugin.json from the Claude manifest + interface block"
phase: phase-codex-plugin-implementation
gaps: [G-02]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-001-runtime-spike]
scope:
  - .codex-plugin/plugin.json
  - scripts/build-codex-plugin.js (or extend build-marketplace.js)
---

# change-cpi-002-plugin-manifest

**Objective.** Emit a spec-conformant .codex-plugin/plugin.json mirroring .claude-plugin/plugin.json, adding the Codex interface block. Generated output, not hand-authored.

## Tasks

- [x] Map .claude-plugin/plugin.json fields → Codex plugin.json (name, version, description, author, repository, license, keywords, skills, mcpServers, hooks pointers; ./-relative in-root paths)
- [x] Add the `interface` block (displayName, shortDescription, longDescription, developerName, category, capabilities, brandColor, logo)
- [x] Emit .codex-plugin/plugin.json via a generator step (single source of truth = .claude-plugin + .mcp.json + hooks + skills)
- [x] Validate: required fields present, all component paths resolve inside root, matches references/codex-plugin-spec-digest.md
