---
id: change-credibility-003-bearer-auth
title: Add bearer token auth to forge-mcp /mcp endpoint
phase: phase-credibility-closure
priority: P0
effort: S
wave: 1
agent: claude
status: done
gap_id: P0-C
verdict: ADOPT
library: tower-http validate-request feature (already in workspace)
scope:
  - tools/forge-rs/crates/forge-mcp/src/lib.rs
  - tools/forge-rs/crates/forge-mcp/Cargo.toml
---

# change-credibility-003 — Add bearer token auth to forge-mcp /mcp endpoint

## Context

The `/mcp` POST handler in `forge-mcp` has no authentication. Any process on the machine (or network, given the 0.0.0.0 bind fixed in C02) can call `forge_enrich` and read arbitrary files via the uncanonicalized path (fixed in C04). Authentication is the third layer in the P0 security cluster.

## Scope

1. Enable `validate-request` feature in `tower-http` (already a workspace dep)
2. Wrap `/mcp` route with `ValidateRequestHeaderLayer::bearer(&token)`
3. Auto-generate token as UUID at startup; print to stderr once
4. Allow override via `FORGE_MCP_TOKEN` environment variable
5. `/health` endpoint remains unauthenticated

## Implementation Notes

`forge-mcp/Cargo.toml`:
```toml
tower-http = { version = "0.6", features = ["cors", "trace", "validate-request"] }
```

`forge-mcp/src/lib.rs` in `ForgeServer::run()`:
```rust
use tower_http::validate_request::ValidateRequestHeaderLayer;

let token = std::env::var("FORGE_MCP_TOKEN")
    .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
eprintln!("forge-mcp token: {token}  (set FORGE_MCP_TOKEN to use a fixed token)");

let mcp_router = Router::new()
    .route("/mcp", post(handle_mcp))
    .route_layer(ValidateRequestHeaderLayer::bearer(&token));

let app = mcp_router
    .route("/health", get(health))
    .with_state(state);
```

Note: `uuid` may need to be added to `forge-mcp/Cargo.toml` if not already present. Check first; use `uuid = { version = "1", features = ["v4"] }`.

## Verification

- `cargo build -p forge-mcp` clean
- POST `/mcp` without `Authorization: Bearer <token>` → 401
- POST `/mcp` with correct token → processes request normally
- GET `/health` → 200 without auth header
- `FORGE_MCP_TOKEN=mytoken forge serve` uses `mytoken`
