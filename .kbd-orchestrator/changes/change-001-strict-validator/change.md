---
id: change-001-strict-validator
title: Add --strict flag to validate-skills.js
phase: phase-developer-ux
gaps: [G3-A2]
priority: P1
effort: XS
agent: native-tool
status: proposed
---

# change-001 — Strict Validator

## Context

`scripts/validate-skills.js` currently warns on missing `license` and does not check `version` or `metadata.tags`. A `// forward-compat` comment at line 126 flags the intent to add strict enforcement. This change adds `--strict` as a CLI flag that escalates these to errors.

## Files

| File | Action |
|------|--------|
| `scripts/validate-skills.js` | Add `strictMode` flag parsing + enforcement block |
| `package.json` | Add `"validate:strict"` script |

## Tasks

- [ ] In `main()`: extract `strictMode = args.includes('--strict')` and `filteredArgs = args.filter(a => a !== '--strict')`
- [ ] Pass `strictMode` to `SkillValidator` (constructor param or instance field set before loop)
- [ ] In `validateSkill()`, replace license-warn block with strict-aware block (errors in strict, warn in standard)
- [ ] Add version and metadata.tags checks under strict mode
- [ ] Add `"validate:strict": "node scripts/validate-skills.js --strict"` to `package.json`
- [ ] `npm run validate` → exit 0, same warnings
- [ ] `npm run validate:strict` → exit non-zero (expected)
- [ ] `npm run validate:strict skills/rust/librefang-wasm-skill` → exit 0

## Acceptance Criteria

1. `npm run validate` exits 0, same warning count as baseline (9 license warnings + script-executable warnings)
2. `npm run validate:strict` exits non-zero
3. `npm run validate:strict skills/rust/librefang-wasm-skill` exits 0
