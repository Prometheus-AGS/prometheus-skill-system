# change-sync-003: sovereign-sync crate scaffold

**Phase:** phase-learn-sovereign-sync
**Tier:** 1 (after Tier 0; all Tier 1 changes need this scaffold)
**Status:** pending
**Library:** cand-009 (axum 0.8.x)
**Gap:** G-02

## Summary

Create the `substrate/sovereign-sync/` crate with the binary entry point
and config loading. Three modes: `mcp` (stdio MCP server), `daemon`
(P2P sync daemon on :7892), `server` (Axum HTTP on :7892).

## Files to change

- `substrate/sovereign-sync/Cargo.toml` — new crate
- `substrate/sovereign-sync/src/main.rs` — clap CLI, mode dispatch
- `substrate/sovereign-sync/src/config.rs` — TOML config loading
- Update workspace `Cargo.toml` to include new crate

## Key structure

```
sovereign-sync [--mode mcp|daemon|server] [--config PATH] [--port PORT]
```

Config file default: `~/.config/sovereign-sync/config.toml`

```toml
[node]
skills_dir = "~/.claude/skills"
operator_id = "<hex>"

[peers]
bootstrap = []

[server]
port = 7892
```

## Tasks

- [ ] Initialize crate with `cargo new --bin`
- [ ] Add clap for CLI parsing
- [ ] Implement config loading (toml + serde)
- [ ] Stub out three mode dispatch branches
- [ ] Add to workspace members
- [ ] `cargo check` clean
