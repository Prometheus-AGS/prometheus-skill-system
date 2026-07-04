# Tasks: change-cowork-005-codex-toml-config

- [x] Create `cli/src/commands/codex_config.rs` with merge_codex_toml, set_goals_enabled, copy_goal_templates, configure_codex
- [x] Register `pub mod codex_config` in `cli/src/commands/mod.rs`
- [x] Call `configure_codex` in install.rs after codex agent install loop completes
- [x] Add unit tests: idempotent TOML merge; goals.enabled idempotent; template copy
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit to cowork-skills fork
- [x] Mark proposal status: done
