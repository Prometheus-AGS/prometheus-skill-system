# change-002-backfill-strict-fields

**Phase:** phase-corpus-strict-compliance
**Status:** PENDING
**Gaps:** G1 (backfill all native skills)
**Effort:** M

## Goal

Write and run `scripts/backfill-strict-fields.js` to auto-inject missing `version`, `license`,
and `metadata.tags` into all 77 native skills. After running, `npm run validate:strict` exits 0.

## Files

- `scripts/backfill-strict-fields.js` — new maintenance script
- `skills/**/*.md` — up to 77 SKILL.md files modified

## Tasks

- [ ] Write `scripts/backfill-strict-fields.js` per plan spec
- [ ] Run `--dry-run` and verify ~77 files listed
- [ ] Spot-check 3 SKILL.md files manually before running for real
- [ ] Run for real: `node scripts/backfill-strict-fields.js`
- [ ] Run `npm run validate` → 0 errors, 0 warnings
- [ ] Run `npm run validate:strict` → 0 errors
- [ ] Run script a second time → 0 modified (idempotency check)
- [ ] Fix any remaining manual edge cases

## Acceptance Criteria

1. `npm run validate:strict` exits 0 (native corpus clean)
2. `npm run validate` exits 0
3. Script is idempotent (second run: 0 modified)

## QA Gate

>3 files — invoke `code-reviewer` agent on `scripts/backfill-strict-fields.js` before running for real
