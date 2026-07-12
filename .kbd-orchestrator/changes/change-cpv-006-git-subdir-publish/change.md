---
id: change-cpv-006-git-subdir-publish
title: "Publish a git-subdir marketplace and confirm remote resolution (GATED on user go-ahead)"
phase: phase-codex-plugin-verify-and-publish
gaps: [G-04]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-cpv-003-ci-green-verify]
scope:
  - (external publish — needs user decision)
---

# change-cpv-006-git-subdir-publish

**Objective.** Verify git-subdir sources resolve from a real remote. Requires an explicit publish decision.

## Tasks

- [ ] CONFIRM with the user before publishing externally; if declined, record a deliberate stage skip with reason
- [ ] Generate a git-subdir marketplace (CODEX_MARKETPLACE_SOURCE=git-subdir) to a publish location / branch and push
- [ ] codex plugin marketplace add <git-url> ; confirm the 11 plugins resolve from git-subdir sources
- [ ] Record evidence; decide whether git-subdir becomes the published default
