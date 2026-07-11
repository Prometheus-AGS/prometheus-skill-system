---
id: change-cpd-003-install-platforms-build-codex
title: "Run build:codex in the install-platforms.ts codex target so install provisions the plugin artifacts"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-01]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - scripts/install-platforms.ts
---

# change-cpd-003-install-platforms-build-codex

**Objective.** Make install regenerate the Codex plugin artifacts, not just sync skills.

## Tasks

- [x] In the codex install path, run `npm run build:codex` (or import + invoke the generator) to regenerate .codex-plugin/ + .agents/plugins/
- [x] Keep the existing codex skills sync (codex-sync-skills.sh) intact
- [x] Verify: running the codex install regenerates artifacts and `npm run validate:codex` passes
