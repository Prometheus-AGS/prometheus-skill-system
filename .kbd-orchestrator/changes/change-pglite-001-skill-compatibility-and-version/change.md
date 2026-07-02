---
id: change-pglite-001-skill-compatibility-and-version
title: PGLite Skill Compatibility Field + Version Note
phase: pglite-certification-2026-05-25
gaps: [G1 (compatibility field), G2 (version pinning)]
priority: Medium
effort: 15 minutes
agent: claude-code
status: done
scope:
  - entity-realtime-local-first/SKILL.md
  - skills/react/prometheus-entity-skills/entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md
---

# change-pglite-001-skill-compatibility-and-version — PGLite Skill Compatibility Field + Version Note

## Problem

`entity-realtime-local-first/SKILL.md` passes strict validation but lacks:
1. A `compatibility` frontmatter field declaring library prerequisites
2. A version note so developers know which `@electric-sql/pglite` API surface the skill targets

Without these, users may wire the skill against incompatible library versions.

## Proposed Change

### File: `skills/react/prometheus-entity-skills/entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md`

**Add to frontmatter** (after `version`):
```yaml
compatibility: Requires @electric-sql/pglite ^0.2 and @electric-sql/client ^0.6 (ElectricSQL shape API)
```

**Add to Building Blocks section** (after the intro bullet for `createElectricAdapter`):
```
> **Tested API surface**: `@electric-sql/pglite ^0.2`, `@electric-sql/client ^0.6`. Shape message types and `ShapeStream.subscribe` may differ in future majors.
```

## Acceptance Criteria

- `compatibility` field present in frontmatter, ≤500 chars
- Version note visible in building blocks section  
- `npm run validate:strict` → 0 errors, 0 warnings
- No change to skill behavior or instructions

## Tasks

- [x] 1. `compatibility` field present in frontmatter, ≤500 chars
- [x] 2. Version note visible in building blocks section
- [x] 3. `npm run validate:strict` → 0 errors, 0 warnings
- [x] 4. No change to skill behavior or instructions
