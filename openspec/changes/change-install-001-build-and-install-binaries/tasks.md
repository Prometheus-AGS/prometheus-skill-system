# Tasks: change-install-001-build-and-install-binaries

- [ ] Inspect Cargo.toml workspace members to confirm package names (`pk-mcp`, `forge-mcp`, `liter-llm`)
- [ ] `cargo build --release -p pk-mcp` in `tools/prometheus-knowledge/`
- [ ] `cargo build --release -p forge-mcp` in `tools/forge-rs/`
- [ ] `cargo build --release` in `tools/liter-llm/`
- [ ] `cp tools/prometheus-knowledge/target/release/pk-mcp ~/.local/bin/pk-mcp`
- [ ] `cp tools/liter-llm/target/release/liter-llm ~/.local/bin/liter-llm`
- [ ] `cp tools/forge-rs/target/release/forge-mcp ~/.local/bin/forge-mcp`
- [ ] `cp tools/prometheus-cli/target/release/prometheus ~/.local/bin/prometheus`
- [ ] `cp tools/forge-rs/target/release/forge ~/.local/bin/forge`
- [ ] Verify all 5 binaries are in PATH with `which`
