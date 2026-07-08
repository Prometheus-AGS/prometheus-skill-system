# Tasks — change-prb-005-a2ui-components

- [x] Create all 8 component modules in `src/a2ui/components/` (graph_view, source_list, contradiction_panel, progress_ring, media_card, stage_timeline, markdown_viewer, citation_list)
- [x] Create `src/a2ui/components/mod.rs` with all 8 pub module declarations
- [x] Update `src/a2ui/mod.rs` to expose components module
- [x] Rewrite `src/a2ui/registry.rs` — HashMap-based registry with `RenderFn` type alias, all 8 components registered
- [x] Download and vendor HTMX 2.0.8 → `src/static/htmx.min.js` (51250 bytes)
- [x] Download and vendor htmx-ext-sse.js 2.2.2 → `src/static/htmx-ext-sse.js` (8896 bytes)
- [x] Download and vendor htmx-ext-loading-states.js 2.0.1 → `src/static/htmx-ext-loading-states.js` (5551 bytes)
- [x] Download and vendor Alpine.js 3.14.8 → `src/static/alpine.min.js` (44758 bytes)
- [x] Run `cargo build --release` — 0 errors (fixed r#..# delimiter collision with hex colors)
- [x] All 8 components return 200; markdown_viewer renders `# Hello` → `<h1>Hello</h1>`; `/static/htmx.min.js` → 200 51250 bytes; 3 unit tests pass
