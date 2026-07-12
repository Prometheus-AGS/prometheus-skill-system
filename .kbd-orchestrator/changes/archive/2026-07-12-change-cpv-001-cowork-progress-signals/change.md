---
id: change-cpv-001-cowork-progress-signals
title: "Add ## Progress Signals section to cowork-management SKILL.md (unblock validate:signals)"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-01]
priority: P0
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - skills/process/cowork-management/SKILL.md
---

# change-cpv-001-cowork-progress-signals

**Objective.** Fix the pre-existing validate:signals CI failure by declaring the mandated Starting/Completed signal contract.

## Tasks

- [x] Add a `## Progress Signals` section to skills/process/cowork-management/SKILL.md per CLAUDE.md → Progress Signaling (Starting/Completed contract)
- [x] Run `npm run validate:signals` locally → passes
- [x] Confirm no other process skill regresses
