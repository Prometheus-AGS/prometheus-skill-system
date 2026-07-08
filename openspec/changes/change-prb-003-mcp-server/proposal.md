---
id: change-prb-003-mcp-server
title: Implement MCP server mode with 5 research tools (rmcp 1.8 stdio)
phase: phase-prometheus-research-binary
priority: P0
effort: L
wave: 2
agent: general-purpose
status: pending
gap_id: G-05
verdict: BUILD
depends_on: change-prb-002-cli-subcommands
scope:
  - substrate/prometheus-research/src/mcp_server/mod.rs
  - substrate/prometheus-research/src/mcp_server/tools.rs
---

# Change: MCP server mode (`--mode mcp`)

## Problem

No MCP tool surface. Claude Code and other MCP-capable harnesses cannot trigger
background research jobs or query their status.

## Solution

Implement `--mode mcp` using rmcp 1.8 stdio transport following the sovereign-sync pattern.

### Tools

| Tool | Params | Returns |
|------|--------|---------|
| `research_start` | `query: String`, `depth: String`, `max_sources: u32`, `citation_style: String` | `{ job_id, started_at }` |
| `research_status` | `job_id: String` | `{ stage, stage_name, progress, status, elapsed_secs, tokens_used }` |
| `research_cancel` | `job_id: String` | `{ cancelled: bool, reason: String }` |
| `research_export` | `job_id: String`, `format: String` | `{ output_path: String }` |
| `render_component` | `name: String`, `props: serde_json::Value` | HTML fragment string |

### Implementation pattern

```rust
// Follow sovereign_sync::mcp_server exactly:
// - #[derive(rmcp::ServerHandler)]
// - #[tool(description = "...")] attribute macros
// - RunServerExt::run_with_stdio() transport
```

`render_component` calls into the A2UI registry (change-prb-005); for this
change, return a `"component not yet implemented"` placeholder.

## Acceptance Criteria

- [ ] `prometheus-research --mode mcp` starts without errors
- [ ] MCP inspector or `claude mcp list` shows 5 tools
- [ ] `research_start` called via MCP returns a valid job ID
- [ ] `research_status` returns checkpoint data for the job
- [ ] `research_cancel` sends SIGTERM and returns `cancelled: true`
- [ ] `cargo build --release` — 0 errors
