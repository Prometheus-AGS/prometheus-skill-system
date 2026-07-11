---
id: change-cpi-008-parity-docs-uar
title: "Parity + publishing checklists, CLAUDE.md Codex docs, UAR-compat verification"
phase: phase-codex-plugin-implementation
gaps: [G-08, G-09]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-007-build-tooling]
scope:
  - docs/
  - CLAUDE.md
  - .kbd-orchestrator/phases/phase-codex-plugin-implementation/
---

# change-cpi-008-parity-docs-uar

**Objective.** Close the phase: document parity, publishing, and prove UAR deployment is unaffected.

## Tasks

- [x] Author a component-by-component parity checklist vs the Claude-Code plugin (skills/mcp/hooks/marketplace/apps)
- [x] Author a Codex publishing checklist (version bump, generate, validate, marketplace add, install smoke)
- [x] Update CLAUDE.md 'Codex CLI Integration' with the new plugin/marketplace surface + verified hook/mcp behavior
- [x] Verify UAR submodule ingestion still loads the skill tree and .codex/ regeneration is unaffected; record evidence
