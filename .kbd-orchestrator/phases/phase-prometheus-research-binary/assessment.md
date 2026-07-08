# Assessment — phase-prometheus-research-binary

_Generated: 2026-07-08_

## Executive Summary

The `deep-research` skill (commit `5397353`) delivers a complete 10-stage research pipeline as
SKILL.md instructions. However, it runs entirely within the agent's context window: no background
execution, no real-time progress streaming, and no persistence across sessions. This phase
scaffolds `prometheus-research` — a Rust CLI + Axum HTTP server + MCP server binary — to
eliminate those constraints.

**Assessment verdict:** PROCEED. All required integrations are clear from existing substrate
crates. Port 7891 is available (surface-bridge=7890, sovereign-sync=7892). The sovereign-sync
pattern (clap 4 + axum 0.8 + rmcp 1.8 + tokio 1) is the canonical template; prometheus-research
follows it exactly.

---

## Goal Coverage

| Goal | Status | Gap |
|------|--------|-----|
| G-01: Scaffold crate | NOT MET | Crate `substrate/prometheus-research/` does not exist |
| G-02: `start` subcommand | NOT MET | No job spawn or checkpoint write exists |
| G-03: `status` subcommand | NOT MET | No checkpoint reader exists |
| G-04: `cancel` subcommand | NOT MET | No SIGTERM or cancellation logic exists |
| G-05: MCP server (`--mode mcp`) | NOT MET | No MCP tool definitions exist |
| G-06: SSE streaming → surface-bridge | NOT MET | No AG-UI event emit logic exists |
| G-07: launchd plist | NOT MET | No plist file exists |
| G-08: Commit + tag v1.6.0 | NOT MET | Crate not yet in repo |

All 8 goals have a clear implementation path. No blockers.

---

## Architecture

### Binary location

```
substrate/prometheus-research/
├── Cargo.toml
├── src/
│   ├── main.rs           # clap CLI entry, mode dispatch
│   ├── lib.rs            # pub re-exports
│   ├── job/
│   │   ├── mod.rs
│   │   ├── checkpoint.rs # read/write ~/.research-jobs/<job-id>/checkpoint.json
│   │   ├── spawn.rs      # fork background process, write PID file
│   │   └── cancel.rs     # read PID, send SIGTERM, update checkpoint
│   ├── mcp_server/
│   │   ├── mod.rs        # rmcp 1.8 ServerHandler
│   │   └── tools.rs      # research_start, research_status, research_cancel, research_export, render_component
│   ├── http_server/
│   │   ├── mod.rs        # axum 0.8 router
│   │   ├── health.rs     # GET /health
│   │   ├── sse.rs        # GET /api/v1/jobs/:id/events  (AG-UI SSE stream)
│   │   ├── rest.rs       # POST /api/v1/jobs (start), GET /api/v1/jobs/:id (status), DELETE /api/v1/jobs/:id (cancel)
│   │   └── components.rs # GET /components/:name  (A2UI HTMX fragment registry)
│   ├── agui/
│   │   ├── mod.rs        # AguiEvent enum — all event types
│   │   └── emit.rs       # broadcast to SSE channel + POST to surface-bridge
│   ├── a2ui/
│   │   ├── mod.rs        # ComponentRegistry
│   │   ├── registry.rs   # named component → HTML fragment fn
│   │   └── components/
│   │       ├── graph_view.rs         # knowledge graph D3/HTMX SVG fragment
│   │       ├── source_list.rs        # source cards
│   │       ├── contradiction_panel.rs
│   │       ├── progress_ring.rs
│   │       ├── media_card.rs
│   │       ├── stage_timeline.rs
│   │       ├── markdown_viewer.rs    # markdown → safe HTML
│   │       └── citation_list.rs
│   └── static/
│       ├── htmx.min.js               # HTMX 2.0.8 (vendored)
│       ├── htmx-ext-sse.js           # htmx-ext-sse 2.x (vendored)
│       ├── htmx-ext-loading-states.js
│       └── alpine.min.js             # Alpine.js 3.x (vendored)
└── tests/
    ├── job_lifecycle.rs
    ├── mcp_tools.rs
    └── sse_stream.rs
```

### Port allocation

| Service | Port | Mode |
|---------|------|------|
| surface-bridge | 7890 | HTTP MCP App (Tier 2 UI) |
| prometheus-research | **7891** | AG-UI SSE + REST + A2UI components |
| sovereign-sync | 7892 | P2P CRDT sync |

### Mode dispatch

```
prometheus-research start "query" [--depth deep] [--max-sources 20] [--citation-style apa]
prometheus-research status <job-id>
prometheus-research cancel <job-id>
prometheus-research --mode mcp         # stdio MCP server
prometheus-research --mode server      # HTTP server on :7891
prometheus-research --mode status      # health check
```

---

## Cargo.toml dependencies

Mirror sovereign-sync precisely, adding:

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# HTTP server
axum = { version = "0.8", features = ["json"] }
tower-http = { version = "0.6", features = ["cors"] }
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"

# MCP
rmcp = { version = "1.8", features = ["server", "transport-io", "macros"] }
schemars = "1.0"

# Serde
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"
anyhow = "1"

# Dirs / process
dirs-next = "2"
nix = { version = "0.28", features = ["signal"] }   # SIGTERM on Unix

# Markdown rendering (server-side → safe HTML)
pulldown-cmark = "0.11"

# Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## AG-UI Event Schema

All events are JSON sent over SSE (`text/event-stream`) at
`GET /api/v1/jobs/:id/events`:

```json
{ "type": "agent.status",  "job_id": "...", "stage": 3, "stage_name": "retrieve",
  "progress": 35, "status": "running", "tokens": 45000, "timestamp": "..." }

{ "type": "agent.message", "job_id": "...", "message": "Retrieved 12 chunks",
  "level": "info", "timestamp": "..." }

{ "type": "agent.error",   "job_id": "...", "message": "...", "stage": 5,
  "timestamp": "..." }

{ "type": "a2ui.component", "job_id": "...", "component": "graph_view",
  "props": { ... }, "timestamp": "..." }
```

The HTMX UI (`docs/deep-research/deep-research-ui.html`) connects with:
```js
const es = new EventSource('/api/v1/jobs/' + jobId + '/events');
```

---

## A2UI Component Registry

Each component is an HTMX HTML fragment served at `GET /components/<name>`:

| Component name | Input props | Renders |
|----------------|-------------|---------|
| `graph_view` | `topics[]`, `claims[]`, `relations[]` | SVG knowledge graph (D3-lite via vanilla JS) |
| `source_list` | `sources[]` with scores | Source cards with credibility badges |
| `contradiction_panel` | `claim_a`, `claim_b`, `resolution` | Side-by-side diff with verdict |
| `progress_ring` | `stage`, `total_stages`, `pct` | Circular SVG progress ring |
| `media_card` | `type`, `title`, `url`, `confidence` | Audio/video/image card |
| `stage_timeline` | `stages[]` with status | Horizontal stage progress bar |
| `markdown_viewer` | `content` (raw markdown) | Server-rendered safe HTML |
| `citation_list` | `citations[]` | Formatted citation list (APA/MLA/etc.) |

Components embed HTMX `hx-trigger="sse:a2ui.component[component=='<name>']"` so
they hot-swap themselves when the server emits a matching AG-UI event.

---

## MCP Tools (stdio mode)

| Tool | Params | Returns |
|------|--------|---------|
| `research_start` | `query`, `depth`, `max_sources`, `citation_style` | `job_id`, `started_at` |
| `research_status` | `job_id` | `stage`, `progress`, `status`, `elapsed_secs` |
| `research_cancel` | `job_id` | `cancelled`, `reason` |
| `research_export` | `job_id`, `format` | `output_path` |
| `render_component` | `name`, `props` (JSON) | HTML fragment string |

---

## MCP App UI Integration

The `render_component` MCP tool enables Claude Code to embed A2UI fragments in
its artifact panel via the surface-bridge Tier 2 iframe mechanism:

1. Agent calls `render_component("graph_view", { topics, claims, relations })`
2. `prometheus-research --mode mcp` returns an HTML fragment
3. Agent POSTs a `UiIntent` to `http://127.0.0.1:7890/mcp/render-ui-intent`
4. surface-bridge renders the fragment inside the MCP App iframe

---

## HTMX 2.0.8 + Plugin Stack

Vendored into `src/static/` (served at `/static/`):

| File | Version | Purpose |
|------|---------|---------|
| `htmx.min.js` | 2.0.8 | Core HTMX |
| `htmx-ext-sse.js` | 2.2.2 | SSE extension for AG-UI streaming |
| `htmx-ext-loading-states.js` | 2.0.1 | Loading spinner during AJAX |
| `alpine.min.js` | 3.14 | Alpine.js for reactive state |

The existing `docs/deep-research/deep-research-ui.html` already uses this
stack. The binary serves the static files so the HTML works standalone
(point browser at `http://localhost:7891`).

---

## Job Checkpoint Format

Written to `~/.research-jobs/<job-id>/checkpoint.json`:

```json
{
  "job_id": "job-20260708-abc123",
  "query": "Analyze quantum computing competitive landscape 2026",
  "depth": "deep",
  "max_sources": 20,
  "citation_style": "apa",
  "status": "running",
  "stage": 3,
  "stage_name": "retrieve",
  "progress": 35,
  "pid": 12345,
  "started_at": "2026-07-08T10:00:00Z",
  "last_updated_at": "2026-07-08T10:05:30Z",
  "tokens_used": 45000,
  "sources_found": 12,
  "output_dir": "~/.research-jobs/job-20260708-abc123/"
}
```

---

## launchd Plist

Path: `substrate/prometheus-research/com.prometheus.research.plist`  
Install: `~/Library/LaunchAgents/com.prometheus.research.plist`

Service starts `prometheus-research --mode mcp` on login (stdio MCP, no HTTP
port). When users want the HTTP server (SSE streaming, HTMX UI), they run
`prometheus-research --mode server` manually.

---

## Open Questions

None. All integration points are clear from sovereign-sync and surface-bridge.

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `nix` crate SIGTERM cross-platform | Low | macOS/Linux only; Windows not required |
| HTMX static file size in binary | Low | ~200KB gzipped with `include_bytes!` |
| SSE channel backpressure | Medium | Use `tokio::sync::broadcast` with capacity=128 |
| MCP tool schema drift (rmcp 1.8) | Low | Follow sovereign-sync exact pattern |

---

## Change Plan Preview (for /kbd-plan)

| Change ID | Scope | Goals Covered |
|-----------|-------|---------------|
| change-prb-001 | Scaffold crate + Cargo.toml + main.rs | G-01 |
| change-prb-002 | `start`, `status`, `cancel` subcommands + checkpoint | G-02, G-03, G-04 |
| change-prb-003 | MCP server mode (`--mode mcp`) + 5 tools | G-05 |
| change-prb-004 | HTTP server + SSE + surface-bridge emit | G-06 |
| change-prb-005 | A2UI component registry + 8 components + static assets | G-06 (UI side) |
| change-prb-006 | launchd plist + install-binaries.sh wiring | G-07 |
| change-prb-007 | Tests (job lifecycle, MCP tools, SSE stream) | all |
| change-prb-008 | Commit + tag v1.6.0 | G-08 |
