---
id: change-credibility-002-bind-loopback
title: Bind forge-mcp to 127.0.0.1 + add --bind CLI flag
phase: phase-credibility-closure
priority: P0
effort: S
wave: 1
agent: claude
status: done
gap_id: P0-B
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-mcp/src/lib.rs
  - tools/forge-rs/crates/forge-cli/src/main.rs
---

# change-credibility-002 — Bind forge-mcp to 127.0.0.1 + add --bind CLI flag

## Context

`tools/forge-rs/crates/forge-mcp/src/lib.rs:58` binds to all interfaces:
```rust
let addr = format!("0.0.0.0:{}", self.port);
```

The README and startup banner claim `localhost`/`127.0.0.1`. This discrepancy means forge-mcp is reachable from any network interface on the host machine, not just loopback. Combined with no authentication (C03), this creates a remote attack surface.

## Scope

1. Change default bind from `0.0.0.0` to `127.0.0.1` in `forge-mcp/src/lib.rs`
2. Add `bind_addr: String` field to `ForgeServer` struct
3. Add `--bind <addr>` CLI argument to the `serve` subcommand in `forge-cli/src/main.rs`
4. Print a security warning to stderr when `--bind 0.0.0.0` is used

## Implementation Notes

In `forge-mcp/src/lib.rs`:
```rust
pub struct ForgeServer {
    port: u16,
    bind_addr: String,  // new field
    skills_root: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pk_mcp_url: Option<String>,
}

impl ForgeServer {
    pub fn new(port: u16, bind_addr: Option<String>, ...) -> Self {
        Self {
            port,
            bind_addr: bind_addr.unwrap_or_else(|| "127.0.0.1".to_string()),
            ...
        }
    }
}

// In run():
let addr = format!("{}:{}", self.bind_addr, self.port);
if self.bind_addr == "0.0.0.0" {
    eprintln!("Warning: forge-mcp bound to 0.0.0.0 — reachable from all network interfaces");
}
```

## Verification

- `cargo build -p forge-mcp` clean
- Default server startup → listener on `127.0.0.1:8943` (not `0.0.0.0:8943`)
- `forge serve --bind 0.0.0.0` works but prints warning
