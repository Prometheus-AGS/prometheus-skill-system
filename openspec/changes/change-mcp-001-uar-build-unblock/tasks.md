# Tasks

- [ ] Commit sse-stream 0.2.4 in Cargo.lock and the ContentBlock alias in both mcp_server.rs files
- [ ] Add sse-stream = "0.2.4" to UAR's own [dependencies] — a FLOOR, not a pin
- [ ] cargo clean, then cargo check --lib finishes clean
- [ ] cargo test --lib provenance passes 8/8 from cold
- [ ] PROVE the floor: cargo update -p sse-stream --precise 0.2.2 must be REFUSED by cargo
