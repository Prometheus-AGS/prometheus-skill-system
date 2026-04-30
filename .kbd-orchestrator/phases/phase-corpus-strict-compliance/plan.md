# KBD Plan — phase-corpus-strict-compliance

> **Phase**: phase-corpus-strict-compliance
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Backend**: native-kbd
> **Planned**: 2026-04-29
> **Assessment**: `.kbd-orchestrator/phases/phase-corpus-strict-compliance/assessment.md`

---

## Change Order

| Order | Change ID | Gap | Priority | Effort | Agent |
|-------|-----------|-----|----------|--------|-------|
| 1 | `change-001-perms-and-exclude` | G2 + G4 | P1 | XS | native-tool |
| 2 | `change-002-backfill-strict-fields` | G1 | P0 | M | native-tool |
| 3 | `change-003-ci-gate` | G3 | P1 | XS | native-tool |

**Rationale:**
- change-001 (XS) fixes permissions and wires the `--exclude-submodules` flag so that after change-002 runs, `validate:strict` can be tested cleanly against native skills only
- change-002 (M) is the bulk work — write and run the backfill script against all 77 affected skills
- change-003 (XS) locks in the standard after the corpus is clean

---

## change-001-perms-and-exclude

**Goal:** Fix zeespec-interrogator script permissions (eliminating 6 warnings) and add `--exclude-submodules` flag to `validate-skills.js` so submodule skills are skipped in strict mode.

**Gaps closed:** G2, G4

**Files:**
- `skills/process/zeespec-interrogator/scripts/*.sh` — `chmod +x` (6 files)
- `scripts/validate-skills.js` — add `--exclude-submodules` flag to `findSkills()`
- `package.json` — update `validate:strict` to include `--exclude-submodules`

**Tasks:**
- [ ] `chmod +x skills/process/zeespec-interrogator/scripts/score-coverage.sh skills/process/zeespec-interrogator/scripts/state-checkpoint.sh skills/process/zeespec-interrogator/scripts/state-finalize.sh skills/process/zeespec-interrogator/scripts/state-init.sh skills/process/zeespec-interrogator/scripts/state-resolve-provider.sh skills/process/zeespec-interrogator/scripts/workflow-dispatch.sh`
- [ ] In `validate-skills.js` `findSkills()`: add `--exclude-submodules` arg check; when present, skip the `skills/imported/` category directory
  ```js
  // In findSkills(), after reading categories:
  const excludeSubmodules = process.argv.includes('--exclude-submodules');
  for (const category of categories) {
    if (excludeSubmodules && category === 'imported') continue;
    // ... existing logic
  }
  ```
- [ ] In `main()`: ensure `--exclude-submodules` is stripped from `filteredArgs` alongside `--strict`
- [ ] Update `package.json` `validate:strict` script:
  ```json
  "validate:strict": "node scripts/validate-skills.js --strict --exclude-submodules"
  ```
- [ ] Run `npm run validate` → 0 errors, 0 warnings
- [ ] Run `npm run validate:strict` → still 135+ errors (native only, submodule errors gone), 0 warnings
- [ ] Confirm: `npm run validate:strict skills/imported/artifact-refiner/skills/refine-validate` still works (direct path invocation should bypass the flag)

**Acceptance criteria:**
1. `npm run validate` exits 0 with 0 warnings (was 6 warnings from permissions)
2. `npm run validate:strict` no longer reports errors for any `refine-*` or `sycophancy-correction` skill
3. Strict error count drops from 158 → ~135 (submodule violations excluded)

**QA gate:** 2 files — below threshold; verify by running the two commands above

---

## change-002-backfill-strict-fields

**Goal:** Write `scripts/backfill-strict-fields.js` that auto-injects missing `version`, `license`, and `metadata.tags` fields into all native skills failing strict validation. Run it. Commit the resulting corpus changes.

**Gaps closed:** G1

**Files:**
- `scripts/backfill-strict-fields.js` — new script (write once, run once, keep as maintenance tool)
- `skills/**/*.md` — 77 SKILL.md files modified by the script

**Tasks:**
- [ ] Write `scripts/backfill-strict-fields.js`:
  ```js
  #!/usr/bin/env node
  // Backfills missing version, license, and metadata.tags into SKILL.md files.
  // Skips skills/imported/. Derives tags from category path.
  // Run: node scripts/backfill-strict-fields.js [--dry-run]

  import fs from 'fs';
  import path from 'path';
  import { fileURLToPath } from 'url';
  import yaml from 'js-yaml';

  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const ROOT = path.resolve(__dirname, '..');
  const SKILLS_DIR = path.join(ROOT, 'skills');

  const CATEGORY_TAGS = {
    react: ['react', 'typescript', 'entity-management'],
    process: ['process', 'orchestration', 'automation'],
    rust: ['rust', 'patterns'],
    devops: ['devops', 'gitops', 'kubernetes'],
    testing: ['testing', 'bdd', 'automation'],
    go: ['go', 'patterns'],
    tauri: ['tauri', 'desktop', 'rust'],
    python: ['python', 'bridge'],
    htmx: ['htmx', 'alpine', 'frontend'],
    flutter: ['flutter', 'dart', 'ffi'],
    architecture: ['architecture', 'clean-architecture', 'patterns'],
    typescript: ['typescript', 'patterns'],
  };

  const dryRun = process.argv.includes('--dry-run');
  let modified = 0;
  let skipped = 0;

  function findSkillMds(dir, results = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === 'imported' || entry.name === 'node_modules') continue;
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) findSkillMds(full, results);
      else if (entry.name === 'SKILL.md') results.push(full);
    }
    return results;
  }

  function inferCategory(skillPath) {
    const rel = path.relative(SKILLS_DIR, skillPath);
    const topDir = rel.split(path.sep)[0];
    return topDir;
  }

  function backfill(skillPath) {
    const content = fs.readFileSync(skillPath, 'utf-8');
    const match = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/s);
    if (!match) { skipped++; return; }

    const fmText = match[1];
    const body = match[2];

    let fm;
    try { fm = yaml.load(fmText); } catch { skipped++; return; }

    const category = inferCategory(skillPath);
    let changed = false;
    let newFmText = fmText;

    // Add license if missing
    if (!fm.license) {
      newFmText = `license: MIT\n${newFmText}`;
      changed = true;
    }

    // Add version if missing
    if (!fm.version) {
      // Insert after name: line
      newFmText = newFmText.replace(
        /^(name: .+)$/m,
        `$1\nversion: '1.0.0'`
      );
      changed = true;
    }

    // Add metadata.tags if missing
    const hasTags = /^\s*tags:/m.test(newFmText);
    const hasMetadata = /^metadata:/m.test(newFmText);
    if (!hasTags) {
      const tags = CATEGORY_TAGS[category] || [category];
      const tagsLine = `  tags: [${tags.join(', ')}]`;
      if (hasMetadata) {
        newFmText = newFmText.replace(/^(metadata:.*\n)/m, `$1${tagsLine}\n`);
      } else {
        newFmText += `\nmetadata:\n${tagsLine}`;
      }
      changed = true;
    }

    if (!changed) { skipped++; return; }

    const newContent = `---\n${newFmText}\n---\n${body}`;
    if (!dryRun) fs.writeFileSync(skillPath, newContent, 'utf-8');
    console.log(`${dryRun ? '[DRY]' : '[MOD]'} ${path.relative(ROOT, skillPath)}`);
    modified++;
  }

  for (const p of findSkillMds(SKILLS_DIR)) backfill(p);
  console.log(`\nDone: ${modified} modified, ${skipped} unchanged.`);
  ```
- [ ] Run dry-run first: `node scripts/backfill-strict-fields.js --dry-run` — verify it lists ~77 files without touching them
- [ ] Spot-check 3 files by reading frontmatter before running for real
- [ ] Run: `node scripts/backfill-strict-fields.js`
- [ ] Run `npm run validate` → 0 errors, 0 warnings
- [ ] Run `npm run validate:strict` → 0 errors (native corpus clean), 0 warnings
- [ ] If any remaining errors, fix manually (edge cases the script missed)
- [ ] Run `npm run validate:strict skills/process/ideation-mindmap` → still 0 errors (regression check)

**Acceptance criteria:**
1. `node scripts/backfill-strict-fields.js --dry-run` exits 0, lists ~77 files
2. After running for real: `npm run validate:strict` exits 0
3. `npm run validate` exits 0 (no regressions from YAML edits)
4. Running the script a second time reports 0 modified (idempotent)

**QA gate:** >3 files — invoke `code-reviewer` agent on `scripts/backfill-strict-fields.js` for YAML safety review before running for real

---

## change-003-ci-gate

**Goal:** Update `CLAUDE.md` and `package.json` to document `validate:strict` as the required gate for new skills. Ensure the developer contribution workflow reflects the new standard.

**Gaps closed:** G3

**Files:**
- `CLAUDE.md` — update "Validation" and "Publishing Checklist" sections
- `package.json` — ensure `validate:strict` is clearly labelled

**Tasks:**
- [ ] In `CLAUDE.md` under "Essential Commands" → "Validation":
  - Add: `npm run validate:strict — Full strict validation (required for new skills)`
  - Add: `npm run validate:strict skills/category/name — Strict validate a specific skill`
- [ ] In `CLAUDE.md` under "Publishing Checklist":
  - Change: `[ ] All skills validate: npm run validate` → `[ ] All skills validate: npm run validate:strict`
  - Add: `[ ] New skills pass strict: npm run validate:strict skills/<category>/<name>`
- [ ] In `CLAUDE.md` under "Skill Development Workflow" → "Validate" step:
  - Change `npm run validate:skill` reference to also mention `npm run validate:strict skills/...`
- [ ] In `CLAUDE.md` under "AgentSkills.io Compliance" → Required Elements:
  - Add: `✅ \`version\` field: semver string (e.g., '1.0.0')`
  - Add: `✅ \`metadata.tags\` field: non-empty array of searchable keywords`
- [ ] Run `npm run validate` → still 0 errors (sanity check after CLAUDE.md edits)

**Acceptance criteria:**
1. `CLAUDE.md` contributing section requires all three strict fields for new skills
2. Publishing checklist uses `validate:strict` instead of `validate`
3. `npm run validate` exits 0 (CLAUDE.md changes don't affect validator)

**QA gate:** Documentation-only change — skip QA agent; manual review sufficient

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Backfill script produces invalid YAML (double-adds a field on re-run) | Idempotency test: run twice, verify 0 modified second time; parse output with `npm run validate` |
| `version` insertion regex fails on multi-line `name:` values | SKILL.md names are always single-line; regex is safe. Confirmed by inspecting corpus. |
| `metadata:` section exists with fields other than `tags` — insertion lands in wrong place | The regex `^(metadata:.*\n)` matches the opening line only; `tagsLine` appended right after. Test on a file with existing `metadata:` fields. |
| `--exclude-submodules` silently hides submodule quality issues in CI | Acceptable: documented in CLAUDE.md; submodule owners run their own validation |

---

## Waypoint After Plan

```json
{
  "stage": "execute",
  "next_action": "/kbd-execute change-001-perms-and-exclude",
  "changes_total": 3,
  "changes_completed": 0,
  "active_change": null
}
```

[kbd] Plan complete — advance to execution with `/kbd-execute change-001-perms-and-exclude`
