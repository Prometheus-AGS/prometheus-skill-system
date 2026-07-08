# change-prui-001-skill-md-update

## Summary

Update `skills/research/deep-research/SKILL.md` to document the `prometheus-research`
binary (v1.6.0). The file currently has zero references to the binary, its MCP tools,
the HTTP API on :7891, or the AG-UI SSE stream. This change adds a
`## Background Execution (prometheus-research)` section covering all integration points.

## Goal

G-01: Update `deep-research` SKILL.md

## Files Changed

- `skills/research/deep-research/SKILL.md` — add new section documenting binary usage

## Acceptance Criteria

- [ ] New section present in SKILL.md after `## Quick Start`
- [ ] Covers: verifying binary is running, `research_start` / `research_status` / `research_cancel` / `research_export` MCP tools
- [ ] Covers: SSE stream at `GET /api/v1/jobs/{id}/events`
- [ ] Covers: `render_component` for A2UI fragments
- [ ] Notes that launchd auto-starts `--mode mcp`; manual `--mode server` for HTTP UI
- [ ] SKILL.md passes `npm run validate:skill skills/research/deep-research`

## Risk

Low. Prose-only change; no code modified.
