---
id: change-cpd-004-generator-git-sources
title: "Make marketplace source type configurable (local default, git-subdir/git for external publish)"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-05]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - scripts/build-codex-plugin.js
  - docs/codex-plugin.md
---

# change-cpd-004-generator-git-sources

**Objective.** Support publishing the marketplace beyond in-repo dogfood without changing the default.

## Tasks

- [x] Add a config knob (env/flag, e.g. CODEX_MARKETPLACE_SOURCE=local|git-subdir) — default local
- [x] For git-subdir emit source {source:'git-subdir', url, ref, path}; keep ./-relative path for local
- [x] Keep the generator idempotent + validating; document in docs/codex-plugin.md
- [x] Verify local default output is byte-unchanged (validate:codex green)
