---
id: change-cowork-012-skill-md
title: cowork SKILL.md + CLAUDE.md documentation
phase: cowork-integration
priority: P1
effort: S
wave: 4
agent: general-purpose
status: done
gap_id: G-05-cowork
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - skills/process/cowork-management/SKILL.md (new)
  - .claude-plugin/marketplace.json (add cowork-management entry)
  - CLAUDE.md (add cowork to Essential Commands)
---

# change-cowork-012 — cowork SKILL.md + CLAUDE.md documentation

## Context

The cowork CLI is now installed via submodule + install-binaries.sh. Agents and
humans need an agentskills.io-compliant SKILL.md that documents how to invoke
cowork, what commands exist, and how it integrates with the prometheus-skill-pack.
The CLAUDE.md Essential Commands section should also list cowork as the primary
skill-management utility.

## Strategy

1. Create `skills/process/cowork-management/SKILL.md` with full agentskills.io frontmatter
2. Document all major command groups: pack, toolchain, disk, install, plugins, config, doctor
3. Add references/ for detailed command table
4. Update `.claude-plugin/marketplace.json` with a cowork-management skill entry
5. Add cowork examples to CLAUDE.md Essential Commands

## Scope

1. Create `skills/process/cowork-management/SKILL.md` (agentskills.io compliant)
2. Create `skills/process/cowork-management/references/COMMANDS.md` (full command reference)
3. Update `.claude-plugin/marketplace.json` — add cowork-management plugin entry
4. Update `CLAUDE.md` — add cowork to Essential Commands section
5. Run `npm run validate:strict skills/process/cowork-management`
6. Commit

## Verification

- `npm run validate:strict skills/process/cowork-management` exits 0
- `skills/process/cowork-management/SKILL.md` has valid frontmatter
- `.claude-plugin/marketplace.json` contains cowork-management entry
- `CLAUDE.md` mentions `cowork pack update`, `cowork toolchain status`, `cowork disk scan`
