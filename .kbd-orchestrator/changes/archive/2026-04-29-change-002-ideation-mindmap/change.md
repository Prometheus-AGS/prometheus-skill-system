---
id: change-002-ideation-mindmap
title: Create ideation-mindmap skill (stage-zero onramp)
phase: phase-developer-ux
gaps: [G2-H1]
priority: P0
effort: S
agent: native-tool
status: proposed
---

# change-002 — ideation-mindmap Skill

## Context

No `ideation-mindmap` skill exists. The `surreal-memory` MCP exposes `generate_ideation_mindmap`. `/start-business-build` Stage 1 describes "ideation expansion" but does not invoke a named skill. This change creates the skill and wires Stage 1 to call it explicitly.

## Files

| File | Action |
|------|--------|
| `skills/process/ideation-mindmap/SKILL.md` | Create new skill |
| `skills/process/native-agent/skills/start-business-build/SKILL.md` | Update Stage 1 to invoke `/ideation-mindmap` |

## Tasks

- [ ] Create `skills/process/ideation-mindmap/` directory
- [ ] Write `SKILL.md` with full frontmatter (name, description, license, version, authors, metadata.tags, triggers)
- [ ] Body: MCP invocation, 6-branch output format, handoff prompt to `/zeespec-interrogate`
- [ ] Edit `start-business-build/SKILL.md` Stage 1 to name `/ideation-mindmap $CONCEPT`
- [ ] `npm run validate:skill skills/process/ideation-mindmap` → 0 errors
- [ ] `npm run validate:strict skills/process/ideation-mindmap` → 0 errors
- [ ] `npm run validate` → still exits 0 overall

## Acceptance Criteria

1. `npm run validate:skill skills/process/ideation-mindmap` exits 0
2. `npm run validate:strict skills/process/ideation-mindmap` exits 0
3. `start-business-build` Stage 1 explicitly names `/ideation-mindmap`
4. `npm run validate` exits 0
