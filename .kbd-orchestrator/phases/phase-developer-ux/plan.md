# KBD Plan — phase-developer-ux

> **Phase**: phase-developer-ux
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Backend**: native-kbd
> **Planned**: 2026-04-29
> **Assessment**: `.kbd-orchestrator/phases/phase-developer-ux/assessment.md`

---

## Change Order

| Order | Change ID | Gap | Priority | Effort | Agent |
|-------|-----------|-----|----------|--------|-------|
| 1 | `change-001-strict-validator` | G3-A2 | P1 | XS | native-tool |
| 2 | `change-002-ideation-mindmap` | G2-H1 | P0 | S | native-tool |
| 3 | `change-003-native-commands` | G1-B2 | P0 | M | native-tool |

**Rationale:**
- change-001 is XS (2 files, ~30 lines) and has zero deps — done first so change-002 can immediately validate against `--strict`
- change-002 creates the new skill and validates it is clean under strict mode — proves A2 works
- change-003 is highest surface area (88+ generated files) — done last after validator and skill are settled

---

## change-001-strict-validator

**Goal:** Add `--strict` flag to `validate-skills.js` that escalates `license` warn → error and adds `version`/`metadata.tags` as required errors. Standard `npm run validate` behavior unchanged.

**Gaps closed:** G3-A2

**Files:**
- `scripts/validate-skills.js` — add `--strict` parsing + enforcement block
- `package.json` — add `"validate:strict": "node scripts/validate-skills.js --strict"`

**Tasks:**
- [ ] Read current `validateSkill()` flow in `validate-skills.js` (already done in assessment)
- [ ] In `main()`, extract `strictMode = args.includes('--strict')` before `filteredArgs = args.filter(a => a !== '--strict')`
- [ ] Pass `strictMode` into `SkillValidator` constructor (or as field on instance set before `validateSkill` calls)
- [ ] In `validateSkill()`, replace existing license warn block with:
  ```js
  if (strictMode) {
    if (!frontmatter.license) {
      this.addError(skillName, 'Strict: missing required field: license');
    }
    if (!frontmatter.version) {
      this.addError(skillName, 'Strict: missing required field: version');
    }
    const tags = frontmatter.metadata?.tags;
    if (!tags || !Array.isArray(tags) || tags.length === 0) {
      this.addError(skillName, 'Strict: missing required field: metadata.tags (must be non-empty array)');
    }
  } else {
    if (!frontmatter.license) {
      this.addWarning(skillName, 'Missing recommended frontmatter field: license');
    }
  }
  ```
- [ ] Add `"validate:strict": "node scripts/validate-skills.js --strict"` to `package.json` scripts
- [ ] Run `npm run validate` → must exit 0, same warning count as before
- [ ] Run `npm run validate:strict` → must exit non-zero (expected: 54+ skills missing fields)
- [ ] Run `npm run validate:strict skills/rust/librefang-wasm-skill` → must exit 0

**Acceptance criteria:**
1. `npm run validate` exits 0, warnings unchanged
2. `npm run validate:strict` exits non-zero
3. `npm run validate:strict skills/rust/librefang-wasm-skill` exits 0

**QA gate:** 2 files — skip QA agent (below 3-file threshold); verify by running the three commands above

---

## change-002-ideation-mindmap

**Goal:** Create `skills/process/ideation-mindmap/SKILL.md` — a stage-zero onramp for `/start-business-build` that calls `generate_ideation_mindmap` via surreal-memory to produce a 6-branch concept tree. Update `start-business-build` Stage 1 to invoke it explicitly.

**Gaps closed:** G2-H1

**Files:**
- `skills/process/ideation-mindmap/SKILL.md` — new skill file
- `skills/process/native-agent/skills/start-business-build/SKILL.md` — update Stage 1 to reference `/ideation-mindmap`

**Tasks:**
- [ ] Create `skills/process/ideation-mindmap/` directory
- [ ] Write `SKILL.md` with the following frontmatter:
  ```yaml
  ---
  name: ideation-mindmap
  description: Stage-zero onramp for /start-business-build. Takes a one-line business concept and generates a 6-branch concept mindmap via surreal-memory, structuring raw ideas into actionable branches ready for zeespec constraint capture.
  license: MIT
  version: '1.0.0'
  authors:
    - Prometheus AGS
  metadata:
    category: process
    tags: [ideation, mindmap, surreal-memory, business-build, stage-zero]
  triggers:
    keywords:
      - ideation mindmap
      - concept tree
      - expand idea
      - business concept branches
    semantic: >
      User provides a one-line business concept or outcome and wants it
      structured into branched concept clusters before deeper specification.
  ---
  ```
- [ ] Write body content covering:
  - **When to invoke**: any time user has a raw business concept and needs it structured before `/zeespec-interrogate`
  - **MCP call**: `generate_ideation_mindmap(topic: $ARGUMENTS, branches: 6)`
  - **Output format**: numbered branch list (Branch 1–6) with 3–5 sub-bullets per branch describing concept clusters
  - **Handoff**: after output, prompt user "Which branches resonate? Accept all 6 or select a subset for `/zeespec-interrogate`"
  - **Integration note**: `/start-business-build` Stage 1 invokes this skill automatically; can also be invoked standalone
- [ ] Edit `start-business-build/SKILL.md` Stage 1 section to explicitly call `/ideation-mindmap $CONCEPT` (currently Stage 1 is described as "ideation expansion" without naming a skill)
- [ ] Run `npm run validate:skill skills/process/ideation-mindmap` → 0 errors
- [ ] Run `npm run validate:strict skills/process/ideation-mindmap` → 0 errors (all strict fields present)
- [ ] Run `npm run validate` → still 0 errors overall

**Acceptance criteria:**
1. `npm run validate:skill skills/process/ideation-mindmap` exits 0
2. `npm run validate:strict skills/process/ideation-mindmap` exits 0
3. `start-business-build` Stage 1 explicitly names `/ideation-mindmap`
4. `npm run validate` still exits 0 overall

**QA gate:** 2 files — skip QA agent; verify by running the three commands above

---

## change-003-native-commands

**Goal:** Create `.claude/commands/*.md` — one file per skill, populated from SKILL.md frontmatter — so Claude Code picks up all prometheus skills as project-scoped slash commands without any install step. Add `generate:commands` npm script. Deprecate `register:commands`.

**Gaps closed:** G1-B2

**Files:**
- `scripts/generate-commands.js` — new Node script
- `.claude/commands/*.md` — generated command files (committed to repo)
- `package.json` — add `"generate:commands"`, deprecate/remove `"register:commands"`

**Tasks:**
- [ ] Create `.claude/` and `.claude/commands/` directories
- [ ] Write `scripts/generate-commands.js`:
  ```js
  #!/usr/bin/env node
  // Generates .claude/commands/*.md from skills/*/SKILL.md frontmatter.
  // Run: node scripts/generate-commands.js
  import fs from 'fs';
  import path from 'path';
  import { fileURLToPath } from 'url';
  import yaml from 'js-yaml';

  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const ROOT = path.resolve(__dirname, '..');
  const SKILLS_DIR = path.join(ROOT, 'skills');
  const COMMANDS_DIR = path.join(ROOT, '.claude', 'commands');

  fs.mkdirSync(COMMANDS_DIR, { recursive: true });

  function findSkillMds(dir, results = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.name === 'imported' || entry.name === 'node_modules') continue;
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        findSkillMds(fullPath, results);
      } else if (entry.name === 'SKILL.md') {
        results.push(fullPath);
      }
    }
    return results;
  }

  const skillMds = findSkillMds(SKILLS_DIR);
  let generated = 0;

  for (const skillMdPath of skillMds) {
    const content = fs.readFileSync(skillMdPath, 'utf-8');
    const match = content.match(/^---\n([\s\S]*?)\n---/);
    if (!match) continue;
    let fm;
    try { fm = yaml.load(match[1]); } catch { continue; }
    if (!fm?.name || !fm?.description) continue;

    const relPath = path.relative(ROOT, skillMdPath).replace(/\\/g, '/');
    const desc = fm.description.slice(0, 200);
    const cmdContent = `---\ndescription: ${JSON.stringify(desc)}\n---\n\n{{file:${relPath}}}\n\n$ARGUMENTS\n`;

    fs.writeFileSync(path.join(COMMANDS_DIR, `${fm.name}.md`), cmdContent);
    generated++;
  }

  console.log(`Generated ${generated} command files in .claude/commands/`);
  ```
- [ ] Run `node scripts/generate-commands.js` — verify generates ≥88 `.md` files in `.claude/commands/`
- [ ] Spot-check 3 command files for correct frontmatter and `{{file:...}}` reference
- [ ] Add `"generate:commands": "node scripts/generate-commands.js"` to `package.json` scripts
- [ ] Deprecate `register:commands`: rename to `"register:commands:opencode"` with a comment, or add `"register:commands"` that prints a deprecation notice and calls the opencode-specific version
- [ ] Add `.claude/` to a gitignore exception if needed (it may be gitignored globally)
- [ ] Run `npm run validate` → 0 errors
- [ ] Run `npm run validate:strict` → same exit code as before (non-zero, expected)
- [ ] Verify `node scripts/generate-commands.js` is idempotent (run twice, identical output)

**Acceptance criteria:**
1. `ls .claude/commands/ | wc -l` ≥ 88
2. Each generated file has `description:` frontmatter and `{{file:...}}` body
3. `node scripts/generate-commands.js` run twice produces identical file set
4. `npm run validate` exits 0
5. `"generate:commands"` script present in `package.json`
6. `"register:commands"` deprecated or renamed to `"register:commands:opencode"`

**QA gate:** >3 files changed — invoke `code-reviewer` agent after implementation on the `generate-commands.js` script for correctness review

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| `{{file:...}}` not supported in project-scope `.claude/commands/` | Test with a single hand-written file before running the generator; fallback: inline description + `$SKILL_PROMPT` var |
| `.claude/` gitignored globally | Check `~/.gitignore_global`; add `!.claude/` exception if needed |
| `generate-commands.js` infinite loop on recursive skill dirs | `findSkillMds` skips `imported/` and `node_modules`; recurses only into dirs (not files) |

---

## Waypoint After Plan

```json
{
  "stage": "execute",
  "next_action": "/kbd-execute change-001-strict-validator",
  "changes_total": 3,
  "changes_completed": 0,
  "active_change": null
}
```

[kbd] Plan complete — advance to execution with `/kbd-execute change-001-strict-validator`
