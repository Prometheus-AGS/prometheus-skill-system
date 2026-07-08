---
id: change-drs-006-validation
title: Run npm run validate:strict and fix all errors for deep-research skill
phase: phase-deep-research-skill
priority: P1
effort: S
wave: 3
agent: general-purpose
status: pending
gap_id: G-05
verdict: BUILD
depends_on: change-drs-005-references-hooks-agents
scope:
  - skills/research/deep-research/SKILL.md (fix frontmatter if needed)
  - skills/research/deep-research/skills/stage-*/SKILL.md (fix any failing sub-skills)
---

# change-drs-006 — Validation + Error Fix

## Context

After all SKILL.md files and supporting files are written, run strict validation
against the full `skills/research/deep-research/` tree. Fix any errors discovered.

## Validation Commands

```bash
# Primary: strict mode required for all new skills
npm run validate:strict skills/research/deep-research

# If any sub-skills fail individually:
npm run validate:strict skills/research/deep-research/skills/stage-01-planner
npm run validate:strict skills/research/deep-research/skills/stage-0N-<name>
```

## Common Errors to Anticipate

1. **Frontmatter name mismatch** — `name:` in frontmatter must match the containing
   directory name exactly. Sub-skills use `deep-research-stage-0N` names so they
   will NOT be caught by the directory-name check (they live under
   `skills/research/deep-research/skills/stage-0N-*/`). The validator checks the
   root `name:` against the directory containing the SKILL.md. Parent checks
   `deep-research`; sub-skills check `stage-0N-<slug>`.
   → Fix: ensure sub-skill names in frontmatter match their directory name.

2. **Missing required strict-mode fields** — `version`, `license`, `metadata.tags`
   → Fix: add missing fields

3. **Backslash in paths** — any `\` in SKILL.md content
   → Fix: replace with `/`

4. **Script not executable** — scripts in `scripts/` or `hooks/` without +x
   → Fix: `chmod +x scripts/\*.sh hooks/\*.sh`

5. **YAML parse error** — malformed frontmatter
   → Fix: quote fields with special characters

## Acceptance Criteria

- [ ] `npm run validate:strict skills/research/deep-research` exits 0
- [ ] Output shows 0 errors, 0 warnings for all files in the tree
- [ ] All scripts are executable
