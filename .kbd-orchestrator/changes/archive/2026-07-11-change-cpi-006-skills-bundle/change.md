---
id: change-cpi-006-skills-bundle
title: "Wire plugin skills pointer to the real-dir tree within catalog budget; keep UAR ingestion intact"
phase: phase-codex-plugin-implementation
gaps: [G-04, G-08]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: [change-cpi-002-plugin-manifest]
scope:
  - .codex-plugin/plugin.json
  - scripts/codex-sync-skills.sh
  - config/codex-catalog.txt
---

# change-cpi-006-skills-bundle

**Objective.** Expose skills to the Codex plugin using real directories (not symlinks), curated within the fixed catalog budget, without disturbing the skills/<domain>/<name>/SKILL.md tree UAR ingests.

## Tasks

- [x] Set plugin.json `skills` pointer to the skill tree (real dirs via codex-sync-skills.sh; Codex ignores symlinked skill dirs)
- [x] Curate catalog membership in config/codex-catalog.txt to stay within the ~130-entry full-description budget; report cost via codex-catalog-stat.py
- [x] Confirm skills/<domain>/<name>/SKILL.md layout is unchanged (UAR $UAR_BUILTIN_SKILLS_DIR submodule ingestion must still load them)
- [x] Verify skills load in a fresh Codex session after install
