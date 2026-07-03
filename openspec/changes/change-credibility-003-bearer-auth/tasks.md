# Tasks: change-credibility-003-bearer-auth

- [ ] Add `validate-request` feature to `tower-http` in `tools/forge-rs/crates/forge-mcp/Cargo.toml`
- [ ] Check if `uuid` crate is present in forge-mcp deps; add `uuid = { version = "1", features = ["v4"] }` if missing
- [ ] Generate token in `ForgeServer::run()`: env var override or UUID fallback
- [ ] Print token to stderr at startup with override instruction
- [ ] Wrap `/mcp` route with `ValidateRequestHeaderLayer::bearer(&token)` as a route layer
- [ ] Keep `/health` route outside the auth layer
- [ ] Run `cargo build --workspace` — verify clean
- [ ] Test: POST /mcp without auth → 401; with correct Bearer → 200; GET /health → 200
