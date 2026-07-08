# change-prui-003-surface-bridge-wiring

## Summary

Fix the HIGH-risk `UiIntent` schema mismatch between `prometheus-research` and
`surface-bridge`, and update `skills/learn/ui-surface/SKILL.md` to document
Tier 2 as live (not STUBBED).

**Root cause:** `prometheus-research/src/agui/emit.rs` sends `{intent, payload}` but
`surface-bridge/src/types.rs` deserializes `{intent_type, title, body, options, multiselect, request_id}`.
The `render_ui_intent` handler will fail to deserialize any event from `prometheus-research`
until this mismatch is fixed.

## Goal

G-03: Wire `render_component` into surface-bridge Tier 2 flow

## Files Changed

- `substrate/prometheus-research/src/agui/emit.rs` — fix `UiIntent` shape sent to surface-bridge
- `skills/learn/ui-surface/SKILL.md` — update Tier 2 section from STUBBED to live documentation

## Acceptance Criteria

- [ ] `emit_to_surface_bridge()` sends `{intent_type, title, body, options, multiselect, request_id}` (surface-bridge schema)
- [ ] `intent_type` mapped from `AguiEvent` variant: `agent.status` → `"progress"`, `agent.message` → `"feedback"`, `agent.error` → `"feedback"`, `a2ui.component` → `"prompt"`
- [ ] `request_id` set to `job_id` from the event
- [ ] `title` and `body` derived from event fields (not empty strings)
- [ ] `cargo build -p prometheus-research` succeeds with no new warnings
- [ ] `skills/learn/ui-surface/SKILL.md` Tier 2 section no longer says STUBBED
- [ ] Tier 2 section documents the live 4-step flow: `render_component` → MCP tool → HTML fragment → POST to surface-bridge
- [ ] `npm run validate:skill skills/learn/ui-surface` passes

## Risk

High. Rust struct change — must compile cleanly. SKILL.md change is prose-only.
