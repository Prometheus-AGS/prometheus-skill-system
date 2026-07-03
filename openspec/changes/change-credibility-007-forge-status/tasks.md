# Tasks: change-credibility-007-forge-status

- [ ] Add `Status` variant to `Commands` enum in `forge-cli/src/main.rs`
- [ ] Implement `Commands::Status` handler: print constitutions, skill count, drift reports, pk_mcp_url, active/gated features
- [ ] Update `Commands::Optimize`, `Commands::Generate`, `Commands::Evolve` to print [EXPERIMENTAL] prefix instead of fake success
- [ ] Update help text for stub commands to note they are experimental
- [ ] Run `cargo build --workspace` — clean
- [ ] Test: `forge status` prints expected sections; `forge optimize` prints [EXPERIMENTAL] warning
