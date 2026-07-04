# Tasks: change-dsg-003-dsg-safety-module

- [x] Add `trash = "0.3"` to `dsg/Cargo.toml` dependencies
- [x] Add `exclude_paths: Vec<String>` field to `Config` in `dsg/src/config.rs`
- [x] Create `dsg/src/safety.rs` with SafetyEngine, ActivityCheck, verify_activity, move_to_trash, should_exclude, age_guard
- [x] Add `mod safety;` to `dsg/src/main.rs` and integrate SafetyEngine into cmd_clean
- [x] Add `Status` subcommand stub to `dsg/src/main.rs`
- [x] Add unit tests for activity check, age guard, exclusion matching
- [x] Run `cargo build --release` from dsg repo root — must exit 0
- [x] Run `cargo test` from dsg repo root — all tests must pass
- [x] Commit to disk-space-guardian repo
