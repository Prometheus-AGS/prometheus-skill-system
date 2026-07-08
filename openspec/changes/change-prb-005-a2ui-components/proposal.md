---
id: change-prb-005-a2ui-components
title: Implement A2UI component registry with 8 HTMX fragments and vendored static assets
phase: phase-prometheus-research-binary
priority: P1
effort: XL
wave: 3
agent: general-purpose
status: pending
gap_id: G-06
verdict: BUILD
depends_on: change-prb-004-http-sse-server
scope:
  - substrate/prometheus-research/src/a2ui/mod.rs
  - substrate/prometheus-research/src/a2ui/registry.rs
  - substrate/prometheus-research/src/a2ui/components/graph_view.rs
  - substrate/prometheus-research/src/a2ui/components/source_list.rs
  - substrate/prometheus-research/src/a2ui/components/contradiction_panel.rs
  - substrate/prometheus-research/src/a2ui/components/progress_ring.rs
  - substrate/prometheus-research/src/a2ui/components/media_card.rs
  - substrate/prometheus-research/src/a2ui/components/stage_timeline.rs
  - substrate/prometheus-research/src/a2ui/components/markdown_viewer.rs
  - substrate/prometheus-research/src/a2ui/components/citation_list.rs
  - substrate/prometheus-research/src/static/htmx.min.js
  - substrate/prometheus-research/src/static/htmx-ext-sse.js
  - substrate/prometheus-research/src/static/htmx-ext-loading-states.js
  - substrate/prometheus-research/src/static/alpine.min.js
---

# Change: A2UI component registry + HTMX 2.0.8 static assets

## Problem

No server-side component rendering. The `render_component` MCP tool and
`GET /components/:name` HTTP route return stubs.

## Solution

### A2UI Component Registry

```rust
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Fn(serde_json::Value) -> String + Send + Sync>>,
}
```

Each component is a function `(props: serde_json::Value) -> String` (HTML fragment).

### 8 Components

| Name | Props | Output |
|------|-------|--------|
| `graph_view` | `{ topics[], claims[], relations[] }` | SVG knowledge graph with D3-lite pan/zoom via vanilla JS |
| `source_list` | `{ sources[] }` | Card list with credibility score badge, domain, date |
| `contradiction_panel` | `{ claim_a, claim_b, resolution, strategy }` | Two-column diff with verdict banner |
| `progress_ring` | `{ stage, total_stages, pct, stage_name }` | Circular SVG ring + stage label |
| `media_card` | `{ type, title, url, confidence }` | Audio/video/image card with confidence badge |
| `stage_timeline` | `{ stages[] }` | Horizontal pipeline with completed/active/pending indicators |
| `markdown_viewer` | `{ content }` | Server-rendered markdown → sanitised HTML via `pulldown-cmark` |
| `citation_list` | `{ citations[], style }` | Formatted citation list (APA/MLA/IEEE/Chicago) |

All components include `hx-swap-oob="true"` and an `id` matching their component name
so HTMX can hot-swap them when the server emits a matching `a2ui.component` AG-UI event.

### Static assets (vendored via `include_bytes!`)

Vendor HTMX 2.0.8 + plugins by downloading during build (or embed as raw bytes):
- `htmx.min.js` — HTMX 2.0.8
- `htmx-ext-sse.js` — SSE extension 2.2.2
- `htmx-ext-loading-states.js` — loading states 2.0.1
- `alpine.min.js` — Alpine.js 3.14.8

Serve from `/static/` via `axum::routing::get_service` with a byte-map handler.

### Wire render_component MCP tool

Update `change-prb-003`'s `render_component` stub to call
`ComponentRegistry::render(name, props)`.

## Acceptance Criteria

- [ ] `GET /components/graph_view` with JSON props returns non-empty HTML
- [ ] `GET /components/markdown_viewer` renders `# Hello` as `<h1>Hello</h1>`
- [ ] `GET /components/progress_ring` returns SVG element
- [ ] `GET /static/htmx.min.js` returns 200 with JS content
- [ ] `render_component` MCP tool returns HTML (not placeholder string)
- [ ] `cargo build --release` — 0 errors
- [ ] All 8 component names registered in `ComponentRegistry`
