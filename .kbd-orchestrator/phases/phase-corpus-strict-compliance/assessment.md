# KBD Assessment — phase-corpus-strict-compliance

> **Phase**: phase-corpus-strict-compliance
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Assessed**: 2026-04-29
> **Prior phase**: phase-developer-ux (complete — `--strict` flag shipped)

---

## Phase Goals

| # | Goal |
|---|------|
| G1 | Backfill `version`, `license`, and `metadata.tags` in all native (non-submodule) skills so `npm run validate:strict` exits 0 on the full corpus |
| G2 | Fix `zeespec-interrogator` script permissions (`chmod +x`) so no warnings fire |
| G3 | Add `validate:strict` as the primary CI gate (replaces or supplements `validate`) |
| G4 | Handle submodule skills (artifact-refiner, sycophancy-correction) correctly — do NOT edit them directly; either skip them in strict mode or open upstream PRs |

---

## Corpus State Snapshot

### Strict Validation Baseline

```
npm run validate:strict   → 158 ERRORS, 6 WARNINGS
npm run validate          → 0 ERRORS, 6 WARNINGS  (warnings = zeespec-interrogator script perms)
```

### Error Breakdown by Field

| Field | Error Count | Affected Skills |
|-------|-------------|----------------|
| `metadata.tags` missing/empty | 170 | ~84 skills (some have 2-3 field misses each) |
| `version` missing | 128 | ~55 skills |
| `license` missing | 18 | ~9 skills (7 from submodules) |

**Note:** Error counts exceed skill counts because a single skill can trigger 3 errors (1 per missing field). 84 unique skills have at least one strict error.

### Violation Breakdown by Source

| Source | Skills w/ Errors | Notes |
|--------|-----------------|-------|
| Native skills (skills/* excl. imported/) | 77 | Directly editable |
| Submodule: artifact-refiner | 7 sub-skills | Read-only — must open upstream PR or exclude from strict |
| Submodule: sycophancy-correction | 1 | Read-only — same treatment |

**Submodule detail:**
- `skills/imported/artifact-refiner/skills/refine-{a2ui,content,image,logo,status,ui,validate}` — all 7 missing `license`, `version`, `metadata.tags` (21 errors)
- `skills/imported/sycophancy-correction/` — missing `version`, `metadata.tags` (2 errors)

**Total submodule errors: 23 of 158 (14.6%)** — these cannot be fixed without touching submodule repos.

### Error Breakdown by Category (Native Skills Only)

| Category | Skills w/ Errors | Primary Miss |
|----------|-----------------|--------------|
| `react/` | ~28 skills | `version` + `metadata.tags` |
| `process/` | ~28 skills | `version` + `metadata.tags` |
| `rust/` | ~8 skills | `metadata.tags` only (most have `version` + `license`) |
| `devops/` | ~4 skills | `version` + `metadata.tags` |
| `testing/`, `go/`, `tauri/`, etc. | ~9 skills | varies |

### Skills with All 3 Fields Missing (worst case — native only)

None — the 7 worst-case skills (refine-*) are all in the submodule. All native skills have at least `license`.

### Skills Missing Exactly 2 Fields (version + tags — most common pattern)

57 native skills fall in this bucket. Typical frontmatter looks like:
```yaml
---
license: MIT
name: kbd-assess
description: >
  ...
---
```
Missing: `version` and `metadata: { tags: [...] }`.

### Skills Missing Only `metadata.tags` (have version + license)

18 native skills — primarily rust skills (`async-patterns`, `axum-patterns`, `actor-model`, etc.) and a few process skills. These have `version: '1.0.0'` and `license: MIT` but no `metadata:` block at all.

### zeespec-interrogator Script Permissions

6 shell scripts in `skills/process/zeespec-interrogator/scripts/` are not executable:
- `score-coverage.sh`, `state-checkpoint.sh`, `state-finalize.sh`, `state-init.sh`, `state-resolve-provider.sh`, `workflow-dispatch.sh`

Fix: `chmod +x skills/process/zeespec-interrogator/scripts/*.sh`

---

## Gap Table

| Gap | Priority | Effort | Goal | Description |
|-----|----------|--------|------|-------------|
| G1-BACKFILL-NATIVE | P0 | M | G1 | Add `version`, `license`, `metadata.tags` to all 77 native skills with violations |
| G2-PERMS | P1 | XS | G2 | `chmod +x` zeespec-interrogator scripts |
| G3-CI | P1 | XS | G3 | Update `package.json` to add `validate:strict` as primary gate; update `CLAUDE.md` contributing docs |
| G4-SUBMODULES | P1 | S | G4 | Exclude submodule skills from strict validation OR open upstream PRs for the two submodule repos |

---

## Implementation Blueprint

### G1-BACKFILL-NATIVE — Bulk frontmatter backfill

**Strategy:** Write a Node.js script (`scripts/backfill-strict-fields.js`) that:

1. Walks `skills/` recursively, skipping `imported/`
2. For each `SKILL.md` with a strict violation:
   - Reads frontmatter
   - Derives `version: '1.0.0'` if missing
   - Derives `license: MIT` if missing
   - Derives `metadata.tags` from the skill's category path if missing:
     - `skills/react/` → `[react, entity-management, typescript]`
     - `skills/process/` → `[process, orchestration, automation]`
     - `skills/rust/` → `[rust, patterns]`
     - `skills/devops/` → `[devops, gitops, kubernetes]`
     - `skills/testing/` → `[testing, bdd, automation]`
     - `skills/go/` → `[go, patterns]`
     - `skills/tauri/` → `[tauri, desktop, rust]`
     - `skills/python/` → `[python, ffi, bridge]`
     - `skills/htmx/` → `[htmx, alpine, lit, frontend]`
     - `skills/flutter/` → `[flutter, dart, rust, ffi]`
     - `skills/architecture/` → `[architecture, clean-architecture, patterns]`
     - `skills/typescript/` → `[typescript, patterns]`
   - Injects missing fields into the frontmatter block (YAML-aware insertion, preserving all existing fields)
3. Reports which files were modified and what was added

**YAML injection approach:** Use `js-yaml` to parse frontmatter, add missing fields, then serialize back and splice into the file. Preserve the exact body (everything after the closing `---`).

**Alternative:** Since `js-yaml` round-tripping can reorder keys, use a targeted string-insertion approach:
- If `version` missing: insert `version: '1.0.0'` after the last field in the `---` block before closing `---`
- If `license` missing: same
- If `metadata.tags` missing and `metadata:` block exists: insert `  tags: [<category-derived>]` under `metadata:`
- If no `metadata:` block: insert full `metadata:\n  category: <cat>\n  tags: [<tags>]` block

**Sub-skill handling:** Sub-skills (e.g., `skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md`) inherit the parent category for tag derivation.

**Acceptance criteria:**
1. `npm run validate:strict` exits 0 on all native skills (when run with path exclusion of `imported/`)
2. `npm run validate` still exits 0
3. All 77 modified skills retain identical body content
4. No existing frontmatter fields are removed or reordered incorrectly

---

### G2-PERMS — Script permissions

**Files:** 6 shell scripts in `skills/process/zeespec-interrogator/scripts/`

**Fix:** `chmod +x skills/process/zeespec-interrogator/scripts/*.sh`

**Acceptance criteria:**
- `npm run validate` produces 0 warnings (currently 6 permission warnings)

---

### G3-CI — Promote validate:strict

**Files:**
- `package.json` — add `"validate:ci": "node scripts/validate-skills.js --strict --exclude-submodules"` (once submodule exclusion is wired) OR just document that `validate:strict` is the new developer gate
- `CLAUDE.md` — update contributing section to require all three fields in new skills; reference `validate:strict`

**Acceptance criteria:**
1. `CLAUDE.md` contributing workflow mentions `npm run validate:strict` as the required pre-commit check for new skills
2. New skills are expected to pass `validate:strict` before merge
3. `package.json` scripts section is clear about which is the CI gate

---

### G4-SUBMODULES — Exclude submodule skills from strict validation

**Options:**

**Option A (preferred): Add `--exclude-submodules` flag to validator**
- In `findSkills()`, skip `skills/imported/` when `--exclude-submodules` is passed
- Add `"validate:strict": "node scripts/validate-skills.js --strict --exclude-submodules"` to replace current
- The current validator already has no explicit `imported/` skip — adding it here makes the behavior explicit
- Submodule owners can run strict validation against their own repos independently

**Option B: Open upstream PRs**
- For `artifact-refiner` — update sub-skill frontmatter (they're MIT-licensed already, just need version + tags)
- For `sycophancy-correction` — same minimal additions
- Risk: upstreams may not merge quickly; blocks G1 completion

**Recommendation: Option A** for immediate phase completion. Option B is a follow-up contribution to open-source the standard.

---

### Change order recommendation

1. **change-001-perms-and-submodule-exclude** (XS+XS): Fix zeespec-interrogator permissions AND add `--exclude-submodules` flag. Bundle together as both are trivial and unblocking.
2. **change-002-backfill-script** (S): Write and run `backfill-strict-fields.js` script to auto-add missing fields to native skills.
3. **change-003-ci-gate** (XS): Update `CLAUDE.md` and `package.json` to document `validate:strict` as the CI gate.

**Rationale:**
- change-001 gets `validate:strict` to a clean "native-only" baseline
- change-002 is the bulk work — run the script, verify, commit the 77+ modified SKILL.md files
- change-003 locks in the standard so it doesn't regress

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| YAML serialization reorders/drops fields when round-tripping | High | Use targeted string insertion rather than full yaml.dump(); only add missing fields, never rewrite existing ones |
| Sub-skill detection — backfill script misses nested skills | Medium | Test against known sub-skill directories (`skills/process/kbd-process-orchestrator/skills/`) before bulk run |
| Category-derived tags are too generic / wrong for some skills | Low | Script generates tags; developer reviews diff before committing; tags can always be refined later |
| `--exclude-submodules` flag silently hides real upstream quality issues | Low | Acceptable for this phase; document that submodules should independently pass strict |
| backfill introduces whitespace/encoding changes that break YAML | Medium | Test on 3 representative files before bulk run; compare before/after with `npm run validate` |

---

## Verification Criteria

| # | Check | Target |
|---|-------|--------|
| 1 | `npm run validate:strict` exits 0 (with submodule exclusion) | 0 errors, 0 warnings |
| 2 | `npm run validate` exits 0 | 0 errors (currently 6 warnings from permissions) |
| 3 | All 77 modified SKILL.md files parse as valid YAML | `npm run validate` green |
| 4 | `CLAUDE.md` contributing section updated | Manual review |
| 5 | `scripts/backfill-strict-fields.js` is committed and idempotent | Running it twice produces no changes second run |

---

## Assessment Verdict

**Assessment complete.** Three-change plan is well-scoped. The bulk of the work is the backfill script (change-002) — once written and run, it generates the corpus changes. No blocking unknowns. Proceed to `/kbd-plan phase-corpus-strict-compliance`.

| Gap | Status | Complexity | Blocking unknown? |
|-----|--------|-----------|-------------------|
| G1-BACKFILL-NATIVE | OPEN | Medium | No — path clear |
| G2-PERMS | OPEN | XS | No |
| G3-CI | OPEN | XS | No |
| G4-SUBMODULES | OPEN | Small | No — Option A is clear |
