# Tasks: change-cowork-008-cowork-toolchain-subcommand

- [x] Create `cli/src/commands/toolchain.rs` with execute_status/execute_check/execute_install
- [x] Register `pub mod toolchain` in `cli/src/commands/mod.rs`
- [x] Add `Toolchain` variant to `Commands` enum in `cli/src/main.rs`
- [x] Add dispatch arm in `main()` for `Commands::Toolchain`
- [x] Add unit tests: script resolution; JSON parsing; install instructions; graceful degradation
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit to cowork-skills fork
