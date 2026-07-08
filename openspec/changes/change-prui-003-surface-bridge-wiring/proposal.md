# change-prui-003-surface-bridge-wiring

## Summary

Fix the HIGH-risk `UiIntent` schema mismatch between `prometheus-research` and
`surface-bridge`, and update `skills/learn/ui-surface/SKILL.md` to document Tier 2
as live (not STUBBED).

**Root cause:** `emit.rs` sends `{intent, payload}` — `surface-bridge` deserializes
`{intent_type, title, body, options, multiselect, request_id}`. Tier 2 MCP App iframe
path is completely broken until this is fixed. This change is prerequisite for G-02's
Tier 2 code path.

## Goal

G-03: Wire `render_component` into surface-bridge Tier 2 flow

## Files Changed

- `substrate/prometheus-research/src/agui/emit.rs` — fix `UiIntent` shape
- `skills/learn/ui-surface/SKILL.md` — update Tier 2 section from STUBBED to live

## AguiEvent → UiIntent Mapping

| AguiEvent type | `intent_type` | `title` | `body` |
|----------------|---------------|---------|--------|
| `agent.status` | `"progress"` | `event.stage_name` | `serde_json::to_string({stage, progress, status})` |
| `agent.message` | `"feedback"` | `"Agent message"` | `event.message` |
| `agent.error` | `"feedback"` | `"Research error"` | `event.message` |
| `a2ui.component` | `"prompt"` | `event.component` | `serde_json::to_string(event.props)` |

Common fields: `request_id = job_id`, `options = None`, `multiselect = false`

## Acceptance Criteria

- [ ] `emit_to_surface_bridge()` sends `{intent_type, title, body, options, multiselect, request_id}` shape
- [ ] Each `AguiEvent` variant maps to the correct `intent_type` per table above
- [ ] `request_id` set to `job_id` from the event
- [ ] `options` is `null` / `None`; `multiselect` is `false`
- [ ] `cargo build -p prometheus-research` succeeds with no new warnings
- [ ] `skills/learn/ui-surface/SKILL.md` Tier 2 section no longer says STUBBED
- [ ] Tier 2 section documents live 4-step flow: `render_component` MCP → HTML fragment → POST to surface-bridge → iframe renders fragment
- [ ] `npm run validate:skill skills/learn/ui-surface` passes

## Risk

**High** — Rust struct shape change. Must compile clean. SKILL.md change is prose-only.
Execute AFTER CI workflow (change-prui-004) is in place so CI catches any Rust regression.
