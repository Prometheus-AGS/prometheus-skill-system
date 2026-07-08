# Tasks — change-prui-003-surface-bridge-wiring

- [ ] task-001: Read `substrate/prometheus-research/src/agui/emit.rs` — current `UiIntent` struct shape
- [ ] task-002: Read `substrate/surface-bridge/src/types.rs` — target `UiIntent` struct fields
- [ ] task-003: Rewrite `emit_to_surface_bridge()` in `emit.rs` to build the surface-bridge-compatible shape
- [ ] task-004: Run `cargo build -p prometheus-research` — verify clean compile
- [ ] task-005: Read `skills/learn/ui-surface/SKILL.md` lines 115–135 — current Tier 2 STUBBED block
- [ ] task-006: Rewrite Tier 2 section to document the live 4-step `render_component` → surface-bridge flow
- [ ] task-007: Run `npm run validate:skill skills/learn/ui-surface` — confirm pass
- [ ] task-008: Commit with message `fix(prometheus-research): align emit.rs UiIntent shape with surface-bridge; document Tier 2 as live`
