# Plan: pglite-certification-2026-05-25

**Phase goal**: Close the 5 non-blocking gaps identified in the PGLite skills certification assessment, achieving full certification-quality polish for `entity-realtime-local-first` and its parent plugin manifests.

**Source assessment**: `.kbd-orchestrator/phases/assess/pglite-certification-assessment-2026-05-25.md`  
**Change backend**: OpenSpec  
**Total changes**: 3  
**Estimated effort**: 1–2 hours (all changes are small targeted edits)

---

## Change Register

### change-001 — pglite-skill-compatibility-and-version

**Gap closed**: G1 + G2  
**Files**: `skills/react/prometheus-entity-skills/entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md`  
**Agent**: general-purpose (file edit)

**What to do**:
1. Add `compatibility` frontmatter field: `Requires @electric-sql/pglite ^0.2 and @electric-sql/client ^0.6`
2. In the "Building blocks" section, add a version callout noting the tested API surface

**Acceptance criteria**:
- `npm run validate:strict` still passes 0 errors
- `compatibility` field present and ≤500 chars
- Version note visible in building blocks section

---

### change-002 — realtime-plugin-json-license

**Gap closed**: G3  
**Files**: `skills/react/prometheus-entity-skills/entity-graph-realtime/.claude-plugin/plugin.json`  
**Agent**: general-purpose (file edit)

**What to do**:
1. Add `"license": "MIT"` field to the plugin.json

**Acceptance criteria**:
- JSON is valid after edit
- `license` field present

---

### change-003 — top-level-plugin-pglite-keywords-and-opencode-bump

**Gaps closed**: G4 + G5  
**Files**:
- `.claude-plugin/plugin.json` — add `"pglite"` and `"electricsql"` to `keywords` array
- `.opencode/package.json` — bump `@opencode-ai/plugin` from `1.14.29` to `^1.15.0`  

**Agent**: general-purpose (file edits)

**What to do**:
1. In `.claude-plugin/plugin.json`, insert `"pglite"` and `"electricsql"` into the `keywords` array
2. In `.opencode/package.json`, update `"@opencode-ai/plugin": "1.14.29"` → `"@opencode-ai/plugin": "^1.15.0"` and same for `@opencode-ai/sdk`
3. Run `npm install` in `.opencode/` to regenerate `package-lock.json` (optional — lock file update can be a separate follow-up)

**Acceptance criteria**:
- `npm run validate:strict` still passes
- `pglite` keyword discoverable in top-level plugin manifest
- `@opencode-ai/plugin` version is `^1.15.0` or higher

---

## Execution Order

| # | Change ID | Effort | Blocker? |
|---|-----------|--------|---------|
| 1 | change-001-pglite-skill-compatibility-and-version | 15 min | None |
| 2 | change-002-realtime-plugin-json-license | 5 min | None |
| 3 | change-003-top-level-plugin-pglite-keywords-and-opencode-bump | 15 min | None |

All changes are independent and can be executed in any order. Ordering above is by impact (highest first).

---

## Post-Execution Verification

After all 3 changes:

```bash
# Validate all skills still pass
npm run validate:strict

# Verify JSON validity of edited plugin files
python3 -m json.tool .claude-plugin/plugin.json > /dev/null && echo "top-level plugin.json: valid"
python3 -m json.tool skills/react/prometheus-entity-skills/entity-graph-realtime/.claude-plugin/plugin.json > /dev/null && echo "realtime plugin.json: valid"

# Confirm compatibility field exists
grep "compatibility:" skills/react/prometheus-entity-skills/entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md

# Confirm pglite keyword in top-level plugin
grep "pglite" .claude-plugin/plugin.json
```

Expected: all pass, no errors.

---

## Certification Status After This Phase

| Standard | Pre-phase | Post-phase |
|----------|-----------|-----------|
| agentskills.io strict | ✅ CERTIFIED | ✅ CERTIFIED (enhanced) |
| Claude Code plugin/marketplace | ✅ CERTIFIED | ✅ CERTIFIED (keywords improved) |
| OpenCode compatibility | ✅ CERTIFIED | ✅ CERTIFIED (dep bump) |
| Sycophancy gate | ✅ PASS | ✅ PASS (no change needed) |
