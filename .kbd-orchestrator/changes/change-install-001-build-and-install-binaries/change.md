---
id: change-install-001-build-and-install-binaries
title: Build and install binaries
phase: machine-installation-2026-05-25
gaps: [G-BIN-1, G-BIN-2, G-SVC-3]
agent: claude-code
status: done
---

# change-install-001-build-and-install-binaries — Build and install binaries

## Summary

Build all missing project binaries and install every project binary to `~/.local/bin/`.

## Files Modified

- Shell: `cp` commands only — no source files changed
- Builds: `tools/prometheus-knowledge/`, `tools/liter-llm/`, `tools/forge-rs/`

## Acceptance Criteria

- `which pk-mcp` → `~/.local/bin/pk-mcp`
- `which liter-llm` → `~/.local/bin/liter-llm`
- `which forge-mcp` → `~/.local/bin/forge-mcp`
- `which prometheus` → `~/.local/bin/prometheus`
- `which forge` → `~/.local/bin/forge`

## Tasks

- [x] 1. Inspect Cargo.toml workspace members to confirm package names (`pk-mcp`, `forge-mcp`, `liter-llm`)
- [x] 2. `cargo build --release -p pk-mcp` in `tools/prometheus-knowledge/`
- [x] 3. `cargo build --release -p forge-mcp` in `tools/forge-rs/`
- [x] 4. `cargo build --release` in `tools/liter-llm/`
- [x] 5. `cp tools/prometheus-knowledge/target/release/pk-mcp ~/.local/bin/pk-mcp`
- [x] 6. `cp tools/liter-llm/target/release/liter-llm ~/.local/bin/liter-llm`
- [x] 7. `cp tools/forge-rs/target/release/forge-mcp ~/.local/bin/forge-mcp`
- [x] 8. `cp tools/prometheus-cli/target/release/prometheus ~/.local/bin/prometheus`
- [x] 9. `cp tools/forge-rs/target/release/forge ~/.local/bin/forge`
- [x] 10. Verify all 5 binaries are in PATH with `which`
