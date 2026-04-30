# change-001-perms-and-exclude

**Phase:** phase-corpus-strict-compliance
**Status:** PENDING
**Gaps:** G2 (permissions), G4 (submodule exclusion)
**Effort:** XS

## Goal

Fix `zeespec-interrogator` script permissions (eliminating 6 warnings from `npm run validate`)
and add `--exclude-submodules` flag to `validate-skills.js` so submodule skills are skipped
when running `npm run validate:strict`.

## Files

- `skills/process/zeespec-interrogator/scripts/` — 6 scripts need `chmod +x`
- `scripts/validate-skills.js` — add `--exclude-submodules` to `findSkills()`
- `package.json` — update `validate:strict` to pass `--exclude-submodules`

## Tasks

- [ ] `chmod +x` all 6 zeespec-interrogator scripts
- [ ] Add `--exclude-submodules` flag handling to `findSkills()` in `validate-skills.js`
- [ ] Strip `--exclude-submodules` from `filteredArgs` in `main()` alongside `--strict`
- [ ] Update `package.json`: `"validate:strict": "node scripts/validate-skills.js --strict --exclude-submodules"`
- [ ] Run `npm run validate` → 0 errors, 0 warnings
- [ ] Run `npm run validate:strict` → no refine-* or sycophancy-correction errors; count drops to ~135

## Acceptance Criteria

1. `npm run validate` exits 0 with 0 warnings
2. `npm run validate:strict` no longer reports errors for submodule skills
3. Strict error count: 158 → ~135
