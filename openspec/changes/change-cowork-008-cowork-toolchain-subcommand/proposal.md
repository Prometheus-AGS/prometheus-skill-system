---
id: change-cowork-008-cowork-toolchain-subcommand
title: cowork toolchain subcommand — prometheus toolchain health
phase: cowork-integration
priority: P0
effort: M
wave: 3
agent: general-purpose
status: done
gap_id: G-03
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/toolchain.rs (NEW)
  - cli/src/commands/mod.rs (add toolchain module)
  - cli/src/main.rs (add Toolchain variant + dispatch)
---

# change-cowork-008 — cowork toolchain subcommand

## Context

The `cowork toolchain` subcommand exposes the health of the prometheus stack's
required binaries and MCP services. It delegates to detect-toolchain.sh for
JSON output, pretty-prints the result, and provides a machine-readable exit
code for CI use. Satisfies G-03 (prometheus-pack awareness).

## Scope

1. Create `cli/src/commands/toolchain.rs` with:
   - `ToolchainSubcommand` enum: `Status`, `Check`, `Install`
   - `execute_status()` — shells to `detect-toolchain.sh --json`; parses
     JSON; pretty-prints Rust toolchain, binary locations, MCP service health
   - `execute_check()` — same as status but exits 0 (all healthy) or 1
     (any missing/unhealthy); suitable for CI pre-flight
   - `execute_install(tool: &str)` — prints actionable install instructions
     per known tool name; exits 0
   - Pack root resolution reuses `commands::pack::resolve_pack_root()`
2. Wire `Toolchain` variant into `Commands` enum in `main.rs`
3. Register `pub mod toolchain` in `commands/mod.rs`

## Sub-command surface

```
cowork toolchain status             # Pretty-print full toolchain health
cowork toolchain check              # Exit 0=healthy, 1=missing tools (CI-friendly)
cowork toolchain install <tool>     # Print install instructions for <tool>
```

## detect-toolchain.sh JSON shape (from shared/scripts/detect-toolchain.sh)

```json
{
  "rust": { "installed": true, "version": "1.79.0" },
  "binaries": {
    "cowork": { "installed": true, "path": "/usr/local/bin/cowork" },
    "dsg": { "installed": false },
    "pk": { "installed": true, "path": "..." }
  },
  "mcp_services": {
    "surreal-memory": { "healthy": true, "port": 23001 },
    "prometheus-knowledge": { "healthy": false }
  }
}
```

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass
- `cowork toolchain check` exits 1 when detect-toolchain.sh is absent (graceful degradation)
