---
id: change-003-native-commands
title: Generate .claude/commands/ from SKILL.md frontmatter
phase: phase-developer-ux
gaps: [G1-B2]
priority: P0
effort: M
agent: native-tool
status: proposed
---

# change-003 — Native Commands Directory

## Context

`register-slash-commands.sh` injects into `~/.opencode/opencode.json` as a manual install step. Claude Code natively recognizes `.claude/commands/*.md` at project scope without any install. This change creates that directory and a `generate-commands.js` script that populates it from SKILL.md frontmatter — eliminating the Claude Code install requirement.

## Files

| File | Action |
|------|--------|
| `scripts/generate-commands.js` | Create Node script that generates command files |
| `.claude/commands/*.md` | Generated — committed to repo (≥88 files) |
| `package.json` | Add `generate:commands`, deprecate `register:commands` |

## Tasks

- [ ] Create `scripts/generate-commands.js` (ES module, uses `js-yaml`, `fs`, `path`)
- [ ] Script: find all `SKILL.md` files under `skills/` (skip `imported/`, `node_modules`)
- [ ] Script: parse frontmatter, skip if no `name` or `description`, write `.claude/commands/<name>.md`
- [ ] Command file format: `description:` frontmatter (JSON-stringified, ≤200 chars) + `{{file:<relPath>}}` body + `$ARGUMENTS`
- [ ] Create `.claude/` and `.claude/commands/` directories
- [ ] Run `node scripts/generate-commands.js` → verify ≥88 files generated
- [ ] Spot-check 3 command files for valid frontmatter + `{{file:...}}` reference
- [ ] Add `"generate:commands": "node scripts/generate-commands.js"` to `package.json`
- [ ] Rename `"register:commands"` → `"register:commands:opencode"` in `package.json` (preserves opencode-specific injection for users who need it)
- [ ] Run `npm run validate` → exits 0
- [ ] Run `node scripts/generate-commands.js` twice → identical output (idempotent)
- [ ] Invoke `code-reviewer` agent on `generate-commands.js` (>3 files changed)

## Acceptance Criteria

1. `ls .claude/commands/ | wc -l` ≥ 88
2. Each generated file has valid `description:` frontmatter and `{{file:...}}` body
3. `node scripts/generate-commands.js` is idempotent
4. `npm run validate` exits 0
5. `"generate:commands"` present in `package.json`
6. `"register:commands"` renamed to `"register:commands:opencode"`
