# Tasks: change-credibility-002-bind-loopback

- [ ] Add `bind_addr: String` field to `ForgeServer` struct in `forge-mcp/src/lib.rs`
- [ ] Change default bind from `"0.0.0.0"` to `"127.0.0.1"` in `ForgeServer::new()`
- [ ] Update `run()` to use `self.bind_addr` instead of hardcoded `"0.0.0.0"`
- [ ] Add stderr warning when bind_addr is `"0.0.0.0"`
- [ ] Add `--bind <addr>` CLI arg to `forge serve` in `forge-cli/src/main.rs`
- [ ] Run `cargo build --workspace` to verify clean compilation
- [ ] Verify default startup binds to 127.0.0.1 (check tracing output)
