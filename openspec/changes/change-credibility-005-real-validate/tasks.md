# Tasks: change-credibility-005-real-validate

- [ ] Make `load_constitutions` and `check_constitution` pub in `forge-enricher/src/lib.rs`
- [ ] Export them from `forge-enricher` crate root if not already visible
- [ ] Rewrite `Commands::Validate` arm in `forge-cli/src/main.rs:210-219` to call real checker
- [ ] Handle missing constitution directory gracefully (print info, return 0)
- [ ] Exit code 1 when any `Error`-severity violation found
- [ ] Fix `forge_validate` MCP tool in `forge-mcp/src/lib.rs` to call `check_constitution`
- [ ] Run `cargo build --workspace` — clean
- [ ] Test: file with violation → non-zero exit; clean file → zero exit
