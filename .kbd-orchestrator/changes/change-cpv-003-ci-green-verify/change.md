---
id: change-cpv-003-ci-green-verify
title: "Push and confirm the Validate Skills run goes green with validate:codex executing"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-01]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-cpv-001-cowork-progress-signals, change-cpv-002-format-fix]
scope:
  - (CI verification — no repo files)
---

# change-cpv-003-ci-green-verify

**Objective.** Verify G-01 end-to-end: a real GitHub Actions run is green and my validate:codex step actually runs.

## Tasks

- [ ] Push 001+002; watch the Validate Skills workflow via `gh run view`
- [ ] Confirm the 'Validate Codex plugin artifacts are in sync' step shows ✓ (not `-`/skipped)
- [ ] Confirm the whole workflow is green (all jobs pass)
