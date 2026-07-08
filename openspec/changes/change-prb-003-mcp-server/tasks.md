# Tasks — change-prb-003-mcp-server

- [x] Read sovereign-sync MCP server pattern for reference
- [x] Implement `src/mcp_server/tools.rs` — 5 tool parameter structs with JsonSchema + serde defaults
- [x] Implement `src/mcp_server/mod.rs` — ResearchMcpServer with 5 tools via `#[tool_router]` + `serve_stdio()`
- [x] Implement `research_start` tool — calls `job::spawn::spawn_job()`
- [x] Implement `research_status` tool — calls `job::checkpoint::read()`
- [x] Implement `research_cancel` tool — calls `job::cancel::cancel_job()`
- [x] Implement `research_export` tool — stub returning placeholder output path
- [x] Implement `render_component` tool — stub (calls A2UI registry placeholder)
- [x] Verify `--mode mcp` already wired in `main.rs` (done in change-001)
- [x] Run `cargo build --release` — 0 errors (3 tests pass)
- [x] Smoke test: `echo '{}' | prometheus-research --mode mcp` starts MCP server, logs "Starting..."
