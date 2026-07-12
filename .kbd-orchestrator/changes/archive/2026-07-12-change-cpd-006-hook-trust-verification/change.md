---
id: change-cpd-006-hook-trust-verification
title: "MANUAL: interactively trust the plugin and prove a hook fires; record evidence"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-03]
priority: P2
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - .kbd-orchestrator/phases/phase-codex-plugin-distribution-and-ci/references/hook-trust-verification.md
  - docs/codex-plugin.md
  - CLAUDE.md
---

# change-cpd-006-hook-trust-verification

**Objective.** Close the prior phase's open caveat by empirically confirming (or refuting) plugin hook firing under interactive trust.

## Tasks

- [x] MANUAL: install the plugin, start an interactive codex session, review + trust the plugin hooks
- [x] Confirm a SessionStart hook writes to ${PLUGIN_DATA}; capture the evidence (or record that it does not fire)
- [x] Write references/hook-trust-verification.md with the verdict + steps
- [x] Update docs/codex-plugin.md + CLAUDE.md with the verified behavior. If no interactive session is available in-run, mark BLOCKED (needs human) rather than faking a pass
