# Tasks: change-cowork-007-cowork-pack-subcommand

- [x] Create `cli/src/commands/pack.rs` with execute_status/execute_update/execute_repair
- [x] Register `pub mod pack` in `cli/src/commands/mod.rs`
- [x] Add `Pack` variant to `Commands` enum in `cli/src/main.rs`
- [x] Add dispatch arm in `main()` for `Commands::Pack`
- [x] Add unit tests: pack_root resolution; skill counting; broken symlink detection
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit to cowork-skills fork
