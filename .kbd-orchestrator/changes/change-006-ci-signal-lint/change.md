---
id: change-006-ci-signal-lint
title: CI lint for progress-signal declarations
phase: position-and-handoff-guarantee
gaps: [F4]
priority: P3
effort: S
agent: claude-code
evolver_item_id: null
status: done
scope:
  - scripts/validate-progress-signals.js
  - scripts/progress-signals-baseline.json # scope expansion: ratchet baseline (23 pre-gate skills), required companion
  - package.json
  - .github/workflows/validate.yml
---

# change-006 — CI lint for progress-signal declarations

## Context

The progress-signal rule is mandatory prose; nothing catches a process skill
that omits it. A lint converts the convention into a merge gate.

## Scope

In:

- New `scripts/validate-progress-signals.js` (Node, no deps — match
  validate-skills.js style): scans `skills/process/**/SKILL.md` (recursing
  sub-skills); fails unless body matches
  `/Starting (kbd-|phase |task |change |[a-z-]+ — )/` style signal declaration
  (precise regex decided at implementation; must pass against all current
  process skills or explicitly list known-exempt files with reasons).
- `package.json`: `"validate:signals": "node scripts/validate-progress-signals.js"`.
- `.github/workflows/validate.yml`: add step invoking it in the existing
  validate job.

## Tasks

- [x] 1. Write the lint script; tune until current process skills pass honestly
- [x] 2. npm script + CI wiring
- [x] 3. Run `npm run validate:signals` green

## Verification

`npm run validate:signals` exits 0 on current tree; seeding a fixture skill
without signals makes it exit 1.
