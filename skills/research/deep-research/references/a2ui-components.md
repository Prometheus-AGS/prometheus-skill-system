# A2UI components served by prometheus-research

Eight server-rendered HTMX fragments. Each is registered in
`substrate/prometheus-research/src/a2ui/registry.rs` under the exact name below
and rendered by the matching module in `src/a2ui/components/`. The `/components/{name}`
route (`http_server/mod.rs`) accepts `?job_id=<id>` and returns a fragment with
`hx-swap-oob="true"` on an element whose `id` is the component name, so a client
can inject it into an existing page without a full swap. The `render_component`
MCP tool and the `a2ui.component` SSE event carry the same `{component, props}` pair.

Prop schemas below are derived from the renderers' accessors, so they are the
contract the Rust actually enforces. Every prop is optional at the renderer:
a missing prop falls back to the default in the "Default" column rather than
failing. An unknown component name renders a dashed placeholder naming the
missing component instead of a 404.

## `graph_view`

Knowledge graph minimap: topic chips and claim chips.

| Prop | Type | Default |
|---|---|---|
| `topics` | string[] | `[]` |
| `claims` | string[] | `[]` |

Values are display strings, not the `graph.json` objects; the caller maps
`graph.json.topics[].name` and `graph.json.claims[].text` before rendering.

## `source_list`

Sources with credibility scores.

| Prop | Type | Default |
|---|---|---|
| `sources` | object[] | `[]` |
| `sources[].title` | string | `"Untitled"` |
| `sources[].url` | string | `"#"` |
| `sources[].domain` | string | `"unknown"` |
| `sources[].date` | string | `""` |
| `sources[].credibility_score` | number 0 to 100 | `0` |

Maps directly from `sources/credibility.json` entries.

## `contradiction_panel`

One contradiction with its resolution status. Render once per entry of
`contradictions.json`.

| Prop | Type | Default |
|---|---|---|
| `claim_a` | string | `"Claim A not provided"` |
| `claim_b` | string | `"Claim B not provided"` |
| `resolution` | string | `"Under analysis"` |
| `strategy` | string | `"compare"` |

`strategy` is the `strategy_tried` value from `contradictions.json`
(`source_authority`, `recency`, `consensus`, `escalate`).

## `progress_ring`

Stage progress ring.

| Prop | Type | Default |
|---|---|---|
| `stage` | integer | `0` |
| `total_stages` | integer, minimum 1 | `10` |
| `pct` | number, clamped 0 to 100 | `0` |
| `stage_name` | string | `"Processing"` |

The daemon emits this from the checkpoint's `stage`, `stage_name`, and `progress`.

## `media_card`

Media attachment card.

| Prop | Type | Default |
|---|---|---|
| `type` | string (`image`, `video`, `audio`, `document`, other) | `"unknown"` |
| `title` | string | `"Media"` |
| `url` | string | `"#"` |
| `confidence` | number 0 to 1 | `0` |

## `stage_timeline`

Ten-stage execution timeline.

| Prop | Type | Default |
|---|---|---|
| `stages` | object[] | `[]` |
| `stages[].name` | string | `"Stage"` |
| `stages[].status` | string (`pending`, `running`, `complete`, `blocked`, `failed`) | `"pending"` |

Build `stages` from `manifest.json.stages_completed` plus the checkpoint's
current stage; a stage the driver marked blocked carries `status: "blocked"`.

## `markdown_viewer`

Rendered Markdown, used for `report.md`, `plan.md`, and the provenance sidecar.

| Prop | Type | Default |
|---|---|---|
| `content` | string (Markdown) | `""` |

Rendering uses pulldown-cmark on the server; the client receives HTML.

## `citation_list`

Formatted citation list.

| Prop | Type | Default |
|---|---|---|
| `citations` | object[] | `[]` |
| `citations[].author` | string | `"Unknown"` |
| `citations[].title` | string | `"Untitled"` |
| `citations[].year` | string or integer | `"n.d."` |
| `citations[].url` | string | `""` |
| `citations[].publisher` | string | `""` |
| `style` | string (`apa`, `mla`, `chicago`, `ieee`, `vancouver`) | `"apa"` |

Maps from `citations.json`; the renderer formats per `style`, so pass the
structured fields rather than `citations[].formatted`.

## Components that do not exist

`confidence-meter` and `export-card` appeared in earlier drafts of SKILL.md and
were never registered. Confidence is shown by `progress_ring` callers or the
`markdown_viewer` on `index.md`; export state is read from `manifest.json`.
