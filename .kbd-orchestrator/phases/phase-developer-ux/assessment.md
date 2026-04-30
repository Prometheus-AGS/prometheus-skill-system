# KBD Assessment — phase-developer-ux

> **Phase**: phase-developer-ux
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Assessed**: 2026-04-29
> **Prior phase**: phase-librefang-wasm-onramp (complete, 2/2 changes DONE)

---

## Phase Goals

| # | Goal |
|---|------|
| G1 | Migrate slash-commands to native `commands/` directory format recognized by Claude Code and OpenCode |
| G2 | Create `ideation-mindmap` skill that generates a 6-branch concept tree via surreal-memory |
| G3 | Extend the skills validator to enforce `version`, `license`, and `metadata.tags` fields in strict mode |

---

## Codebase State Snapshot

### Current slash-command registration (B2 gap)

**What exists:**
- `scripts/register-slash-commands.sh` — Python-embedded bash script (~120 lines) that:
  1. Reads all `SKILL.md` files under `skills/` (excluding `imported/`)
  2. Injects `command` entries into `~/.opencode/opencode.json`
  3. Creates prompt files in `~/.codex/prompts/`
- `package.json` scripts: `"register:commands"` and `"unregister:commands"` calling the bash script
- No `commands/` directory at project root or in `.claude/`
- No `.claude/` directory inside the project (only global `~/.claude/`)

**What Claude Code natively recognizes:**
- `~/.claude/commands/*.md` — global user commands (89 files found on this machine)
- `.claude/commands/*.md` — project-scoped commands (none exist in this repo)
- File format: minimal YAML frontmatter with `description:`, `argument-hint:` (optional), no required `name:` field (filename is the command name)
- Example from `~/.claude/commands/code-review.md`:
  ```yaml
  ---
  description: Code review — local uncommitted changes or GitHub PR
  argument-hint: [pr-number | pr-url | blank for local review]
  ---
  ```
- No `command: true` flag needed (presence in `commands/` is sufficient)

**Gap summary (G1-B2):**
The current `register-slash-commands.sh` install step is a manual user action. Native `commands/` format in the project root (`.claude/commands/`) would be picked up by Claude Code automatically at project scope without any install script. OpenCode reads these through its Claude Code compatibility layer. The goal is to create `.claude/commands/` populated with one `.md` file per skill.

**Scope decision:** The B2 change generates `.claude/commands/*.md` files from existing `SKILL.md` frontmatter data, then updates `package.json` to remove/deprecate the `register:commands` script. The bash script can remain for backward compat with OpenCode-only users but the Claude Code install requirement is eliminated.

---

### ideation-mindmap skill baseline (H1 gap)

**What exists:**
- No skill named `ideation-mindmap` anywhere in `skills/`
- `surreal-memory` MCP server is active (confirmed in `.mcp.json`)
- `surreal-memory` exposes `generate_ideation_mindmap` tool
- `/start-business-build` pipeline references "ideation expansion" as Stage 1 but does not invoke a dedicated skill for it; Stage 1 is implicit
- `skills/process/native-agent/skills/start-business-build/SKILL.md` lists the pipeline stages; Stage 1 says "ideation expansion" without specifying a skill invocation

**surreal-memory `generate_ideation_mindmap` tool signature** (from MCP server):
```
generate_ideation_mindmap(topic: string, depth?: number, branches?: number)
→ Mindmap with 6 branches (default), each branch is a concept cluster
```

**Gap summary (G2-H1):**
A new `ideation-mindmap` skill must be created at `skills/process/ideation-mindmap/`. The skill wraps `generate_ideation_mindmap` with structured output formatting, prompt engineering for business concept expansion, and clear invocation by `/start-business-build` Stage 1. It is the "stage-zero onramp" — the first step in any business build that takes a free-form concept and produces 6 structured branches for the zeespec interrogator to constrain.

---

### Validator strict mode (A2 gap)

**Current validator state** (`scripts/validate-skills.js`):

| Field | Current behavior |
|-------|----------------|
| `name` | **Required** — errors if absent |
| `description` | **Required** — errors if absent |
| `license` | **Warn** only — `addWarning()` with "Missing recommended frontmatter field: license" |
| `version` | Not checked |
| `metadata.tags` | Not checked |

**Current warning count:** 9 skills produce "Missing recommended frontmatter field: license" warnings (validator currently only warns on `license`).

**Skills missing `version` AND `metadata.tags`:** 54 out of 78 top-level skill files — nearly 70% of the corpus lacks both fields. Many also lack `license` already generating warnings.

The validator comment at line 126 reads:
```js
// Warn when license is absent (forward-compat with future strict validation)
```
This is the explicit forward-compat placeholder that A2 upgrades to strict mode enforcement.

**Gap summary (G3-A2):**
Add `--strict` CLI flag to `validate-skills.js`. When `--strict` is passed:
- `license` → **error** (not warn)
- `version` → **error** if absent
- `metadata.tags` → **error** if absent or empty array

Standard mode (no `--strict`) retains current behavior — warnings only for `license`, no checks for `version`/`metadata.tags`. This preserves backward compatibility for existing CI while allowing opt-in enforcement for new skills. `package.json` gets a new `"validate:strict"` script.

**Note:** The 54 skills missing `version`+`tags` are NOT in scope for this phase to fix — A2 only adds the enforcement gate. Backfilling the corpus is a separate pass.

---

## Gap Table

| Gap | Priority | Effort | Goal | Description |
|-----|----------|--------|------|-------------|
| G1-B2 | P0 | M | G1 | Create `.claude/commands/*.md` from SKILL.md files; deprecate `register:commands` script |
| G2-H1 | P0 | S | G2 | Create `skills/process/ideation-mindmap/SKILL.md` wrapping `generate_ideation_mindmap` |
| G3-A2 | P1 | XS | G3 | Add `--strict` flag to `validate-skills.js` for `version`/`license`/`metadata.tags` enforcement |

---

## Implementation Blueprint

### G1-B2 — Native commands/ directory

**Files to create/modify:**
1. `scripts/generate-commands.js` — Node script that reads all SKILL.md frontmatter, generates `.claude/commands/*.md` files, one per skill with `description` from frontmatter
2. `.claude/commands/` — directory populated by the script (committed to repo)
3. `package.json` — add `"generate:commands": "node scripts/generate-commands.js"`, deprecate `register:commands`

**Command file format:**
```markdown
---
description: <frontmatter.description truncated to 200 chars>
argument-hint: [optional arguments]
---

<SKILL.md body content, or $file reference to the SKILL.md>
```

**Claude Code native command content strategy:**
The simplest approach is to use the `{{file:...}}` syntax that some commands use, pointing directly at the SKILL.md. This avoids duplication — the command file is a thin wrapper:
```markdown
---
description: <from frontmatter>
---

{{file:skills/<category>/<name>/SKILL.md}}

$ARGUMENTS
```

**Key decisions:**
- Commit `.claude/commands/` to the repo (not gitignored) so project-scope commands work without install
- Keep `register-slash-commands.sh` for OpenCode-specific `opencode.json` injection (OpenCode doesn't read `.claude/commands/` natively as of current version)
- The script runs at build time / dev time, not at install time

**Acceptance criteria:**
1. `ls .claude/commands/` shows one `.md` file per skill (≥88 files)
2. Each file has valid YAML frontmatter with `description` populated
3. `npm run generate:commands` regenerates from source and produces identical output
4. `npm run validate` still passes (0 errors)
5. `register:commands` package.json script deprecated or removed
6. Claude Code picks up commands at project scope (verified by `--help` listing)

---

### G2-H1 — ideation-mindmap skill

**Files to create:**
1. `skills/process/ideation-mindmap/SKILL.md` — main skill file

**SKILL.md structure:**

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

**Body content:**
- Invoke `generate_ideation_mindmap(topic, branches=6)` via surreal-memory MCP
- Format output as numbered branch list with sub-bullets for each cluster
- Output is structured for handoff to `/zeespec-interrogate` (next stage in pipeline)
- Prompt the user to pick branches to pursue or accept all 6

**Stage 1 integration:** Update `start-business-build/SKILL.md` to explicitly invoke `/ideation-mindmap` in Stage 1 instructions (currently Stage 1 is implicit).

**Acceptance criteria:**
1. `npm run validate:skill skills/process/ideation-mindmap` passes (0 errors)
2. Skill SKILL.md has all required + recommended fields (name, description, license, version, metadata.tags)
3. `start-business-build` Stage 1 references `/ideation-mindmap` explicitly
4. Invoking `/ideation-mindmap "track competitor pricing"` produces a formatted 6-branch tree

---

### G3-A2 — Validator strict mode

**Files to modify:**
1. `scripts/validate-skills.js` — add `--strict` flag parsing and enforcement
2. `package.json` — add `"validate:strict"` script

**Implementation in `validate-skills.js`:**
```js
// Near top of main():
const strictMode = args.includes('--strict');
const filteredArgs = args.filter(a => a !== '--strict');

// In validateSkill(), after the license check:
if (strictMode) {
  if (!frontmatter.license) {
    this.addError(skillName, 'Strict: missing required field: license');
  }
  if (!frontmatter.version) {
    this.addError(skillName, 'Strict: missing required field: version');
  }
  if (!frontmatter.metadata?.tags || !Array.isArray(frontmatter.metadata.tags) || frontmatter.metadata.tags.length === 0) {
    this.addError(skillName, 'Strict: missing required field: metadata.tags (must be non-empty array)');
  }
} else {
  // existing warn-only for license
  if (!frontmatter.license) {
    this.addWarning(skillName, 'Missing recommended frontmatter field: license');
  }
}
```

**`package.json` addition:**
```json
"validate:strict": "node scripts/validate-skills.js --strict"
```

**Key design choices:**
- `--strict` is a separate flag, not a mode replacement, so `npm run validate` (standard) behavior is unchanged
- In strict mode, `license` escalates from warn → error (no double-emit)
- `metadata.tags` must be a non-empty array (empty `tags: []` fails)
- The flag filters itself out before passing remaining args to `skillsToValidate` path resolution

**Acceptance criteria:**
1. `npm run validate` still passes (0 errors, same warnings as today)
2. `npm run validate:strict` exits non-zero (due to 54+ skills missing fields)
3. `npm run validate:strict skills/rust/librefang-wasm-skill` exits 0 (that skill has all fields)
4. `npm run validate:strict skills/process/ideation-mindmap` exits 0 (new skill will have all fields)

---

## Verification Criteria (§ for plan)

| # | Check | Target |
|---|-------|--------|
| 1 | `npm run validate` green | 0 errors (warnings unchanged) |
| 2 | `.claude/commands/` committed with ≥88 `.md` files | Pass |
| 3 | `npm run generate:commands` idempotent | Regenerated output matches committed files |
| 4 | `/ideation-mindmap` skill validates with 0 errors in strict mode | Pass |
| 5 | `npm run validate:strict skills/process/ideation-mindmap` exits 0 | Pass |
| 6 | `npm run validate:strict` exits non-zero on full corpus | Expected (54+ skills lack required fields) |
| 7 | `start-business-build` Stage 1 references `/ideation-mindmap` | In SKILL.md |

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| `{{file:...}}` syntax not supported in project-scope commands | Medium | Verify with a test command file before bulk generation; fallback: inline the description + prompt |
| 88+ command files in `.claude/commands/` may bloat context | Low | Files are thin wrappers; Claude Code lazy-loads |
| `generate_ideation_mindmap` MCP tool signature changes | Low | Read current tool schema before coding the skill prompt |
| `--strict` used in CI breaks build for new skills missing fields | Low | Flag is opt-in; CI uses standard `npm run validate` |

---

## Change Order Recommendation

1. **change-001-strict-validator** (P1, XS) — simplest isolated change, no deps
2. **change-002-ideation-mindmap** (P0, S) — creates new skill; validates in strict mode as demo
3. **change-003-native-commands** (P0, M) — largest change; generates 88+ files; depends on validator passing

Rationale: Do strict-validator first (XS effort) so change-002's ideation-mindmap skill can be validated with `validate:strict` immediately, proving the A2 feature works with a real skill. change-003 (native commands) is last because it's the highest surface-area change.

---

## Assessment Verdict

**Assessment complete.** All three gaps are well-scoped with clear implementation paths. No blocking unknowns. Proceed to `/kbd-plan phase-developer-ux`.

| Gap | Status | Complexity | Blocking unknown? |
|-----|--------|-----------|-------------------|
| G1-B2 | OPEN | Medium | No — `commands/` format confirmed |
| G2-H1 | OPEN | Small | No — `generate_ideation_mindmap` tool confirmed |
| G3-A2 | OPEN | XS | No — validator code fully read |
