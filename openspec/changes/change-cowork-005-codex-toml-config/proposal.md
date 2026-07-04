---
id: change-cowork-005-codex-toml-config
title: Codex TOML config writer + MCP stanzas + goal templates
phase: cowork-integration
priority: P0
effort: M
wave: 2
agent: general-purpose
status: done
gap_id: G-04
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/codex_config.rs (NEW — TOML merge + template copy logic)
  - cli/src/commands/mod.rs (add codex_config module)
  - cli/src/commands/install.rs (call configure_codex after codex agent install)
---

# change-cowork-005 — Codex TOML config writer + goal templates

## Context

When `cowork install --agent codex` completes, cowork should:
1. Merge Prometheus MCP server stanzas into `~/.codex/config.toml` (idempotent)
2. Set `goals.enabled = true` in config.toml (idempotent)
3. Copy goal prompt templates (`continuation.md`, `budget_limit.md`) from the
   prometheus skill-pack's `skills/process/kbd-goal/templates/codex/` to
   `~/.codex/goals/`

This ports the logic from `scripts/configure-mcp-all-tools.sh` (merge_toml_mcp)
and `scripts/kbd-goal-codex-setup.sh` into Rust using the `toml` crate already
in Cargo.toml, making the config idempotent and testable.

## MCP Stanzas to inject

| Key | Type | URL/Command |
|-----|------|-------------|
| surreal-memory | sse | http://localhost:23001/mcp/sse |
| prometheus-knowledge | http | http://localhost:8942/mcp |
| forge-rs | http | http://localhost:8943/mcp |
| sycophancy-correction | stdio | /usr/local/bin/sycophancy-correction |
| sequential-thinking | stdio/npx | npx -y @modelcontextprotocol/server-sequential-thinking |

## Scope

1. Create `cli/src/commands/codex_config.rs` with:
   - `merge_codex_toml(config_path: &Path) -> Result<Vec<String>>` — idempotent TOML merge
   - `set_goals_enabled(config_path: &Path) -> Result<bool>` — idempotent goals.enabled = true
   - `copy_goal_templates(templates_src: &Path, goals_dir: &Path) -> Result<Vec<String>>` — copy templates
   - `configure_codex(pack_root: Option<&Path>) -> Result<()>` — top-level orchestrator
2. Register module in `cli/src/commands/mod.rs`
3. Call `configure_codex` in `install.rs` after completing codex agent install

## Verification

- `cargo build --release` exits 0
- `cargo test` — all existing + new tests pass
