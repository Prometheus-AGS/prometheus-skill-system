# Tasks: change-cowork-009-cowork-disk-stub

- [x] Create `cli/src/commands/disk.rs` with execute_status/execute_scan/execute_clean
- [x] Register `pub mod disk` in `cli/src/commands/mod.rs`
- [x] Add `Disk` variant to `Commands` enum in `cli/src/main.rs`
- [x] Add dispatch arm in `main()` for `Commands::Disk`
- [x] Add unit tests: dsg presence detection; graceful degradation; arg passthrough
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit to cowork-skills fork
