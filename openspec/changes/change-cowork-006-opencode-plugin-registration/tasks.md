# Tasks: change-cowork-006-opencode-plugin-registration

- [x] Create `cli/src/commands/opencode_config.rs` with register_opencode_plugin, ensure_opencode_package_json, configure_opencode
- [x] Register `pub mod opencode_config` in `cli/src/commands/mod.rs`
- [x] Call `configure_opencode` in install.rs after opencode agent install
- [x] Add unit tests: idempotent plugin[] append; de-dup existing entry; package.json ensure
- [x] Run `cargo build --release` from `cli/` — must exit 0
- [x] Run `cargo test` from `cli/` — all tests must pass
- [x] Commit to cowork-skills fork
