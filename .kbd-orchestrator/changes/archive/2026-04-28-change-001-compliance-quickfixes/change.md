---
id: change-001-compliance-quickfixes
title: Tighten validator + remove dangling refs + populate empty plugin dirs
phase: phase-compliance-and-power-multiplier
gaps: [A1, A3, B1]
priority: P1
effort: XS
agent: code-simplifier
evolver_item_id: null
status: DONE
completed: 2026-04-28
---

# change-001 — Compliance Quickfixes

## Context

Three small compliance defects from §2 of the assessment. None are functional bugs,
but they make the pack fail strict validation against agentskills.io v2026 and the
Claude marketplace plugin schema:

- **A1**: `validate-skills.js:23` allows `description` up to 1024 chars; the spec
  caps it at 200 (the discovery summary). Several SKILL.md files exceed 200 chars.
- **A3**: `skills/documentation/` and `skills/ui-ux/` are empty directories listed
  by the marketplace as containing skills.
- **B1**: `.claude-plugin/plugin.json:41` declares `"mcpServers": "./.mcp.json"`
  but the file does not exist.

## Scope

In:

- Update `scripts/validate-skills.js` schema for `description.maxLength` to 200.
- Update any SKILL.md `description` fields that exceed 200 chars (move long-form
  content to body or to a new `summary` field if needed).
- Either delete `skills/documentation/` and `skills/ui-ux/` from the tree (and from
  `.claude-plugin/plugin.json`), OR create stub skills with valid SKILL.md files.
  **Recommendation**: delete for now; populate later when real skills exist.
- Create a real `.mcp.json` listing the four MCP servers
  (`forge-rs`, `pk-cherry`, `surreal-memory`, `liter-llm`) OR remove the field
  from `plugin.json`. **Recommendation**: create `.mcp.json` since the servers exist.

Out:

- Optional/forward-compat schema additions (A2, A4) — those land in change-006.
- Plugin-native `commands/` migration (B2) — separate phase.

## Deliverables

1. `scripts/validate-skills.js` — `description.maxLength: 200`.
2. `.mcp.json` at repo root with the four MCP server declarations.
3. Updated SKILL.md files (description cap).
4. Deleted empty plugin directories OR populated stubs.
5. Marketplace listing in `marketplace/marketplace.json` reconciled with whichever
   choice was made for documentation/ui-ux.

## Acceptance Criteria

- `npm run validate` passes with the new 200-char cap.
- `cat .mcp.json | jq '.servers | length'` returns 4.
- `find skills -type d -empty` returns no listed-as-skill directories.
- `jq '.mcpServers' .claude-plugin/plugin.json` resolves to a real file.

## Files to Touch

- `scripts/validate-skills.js`
- `.mcp.json` (new)
- `.claude-plugin/plugin.json` (only if removing the mcpServers field)
- `marketplace/marketplace.json` (remove documentation/ui-ux entries if deleting)
- Any SKILL.md whose description exceeds 200 chars

## Test Plan

- Run `npm run validate` — expect green.
- Hand-test: install plugin via `npm run install:project`, confirm Claude Code
  loads the four MCP servers without errors.
