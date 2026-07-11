---
id: change-cpd-002-ci-validate-codex
title: "Add npm run validate:codex to CI (validate.yml) as a drift/validity gate"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-02]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - .github/workflows/validate.yml
---

# change-cpd-002-ci-validate-codex

**Objective.** Fail CI when the committed Codex artifacts are stale or invalid.

## Tasks

- [x] Add a `run: npm run validate:codex` step to validate.yml near the existing npm run validate step
- [x] Confirm it runs after npm ci and fails on drift/invalid (exit 1)
- [x] Validate the workflow YAML parses
