# Change: pglite-001 — PGLite Skill Compatibility Field + Version Note

**Phase**: pglite-certification-2026-05-25  
**Gaps closed**: G1 (compatibility field), G2 (version pinning)  
**Priority**: Medium  
**Effort**: 15 minutes

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

- [ ] `compatibility` field present in frontmatter, ≤500 chars
- [ ] Version note visible in building blocks section  
- [ ] `npm run validate:strict` → 0 errors, 0 warnings
- [ ] No change to skill behavior or instructions
