---
id: change-cpv-002-format-fix
title: "Resolve the 32 prettier CI failures (reconcile local-vs-CI first)"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-01]
priority: P0
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - (prettier-flagged files, mostly tools/disk-space-guardian/)
  - .prettierignore
  - package.json
---

# change-cpv-002-format-fix

**Objective.** Make `Check Formatting` (npm run check-format) green in CI. Diagnose why local reports clean but CI flags 32.

## Tasks

- [x] Diagnose the discrepancy: compare pinned prettier version (CI npm ci) vs local; check .prettierignore + check-format glob
- [x] Apply the correct fix so CI passes: `npm run format` (or align prettier version/ignore) — the 32 files are mostly under tools/disk-space-guardian/
- [x] Run `npm run check-format` in a clean install (npm ci) → passes
- [x] Avoid reformatting files that shouldn't change (respect .prettierignore)
