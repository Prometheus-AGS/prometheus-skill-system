---
id: change-cpv-004-real-hooks-codex
title: "Headless verify the REAL plugin hooks run under Codex with the portability fix"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-03]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: []
scope:
  - .kbd-orchestrator/phases/phase-codex-plugin-verify-and-publish/references/real-hooks-codex.md
---

# change-cpv-004-real-hooks-codex

**Objective.** Close the loop on last phase's ${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT} fix by running the actual (un-probed) hooks under Codex.

## Tasks

- [ ] build:codex + install plugin; DO NOT replace hooks.json this time (use the real hooks)
- [ ] codex exec --dangerously-bypass-approvals-and-sandbox --dangerously-bypass-hook-trust with a trivial prompt
- [ ] Confirm the real SessionStart hooks execute without empty-path errors (${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT} resolves to PLUGIN_ROOT); check ~/.prometheus/hooks.log / stderr
- [ ] Record references/real-hooks-codex.md; clean up ~/.codex (remove plugin+marketplace)
