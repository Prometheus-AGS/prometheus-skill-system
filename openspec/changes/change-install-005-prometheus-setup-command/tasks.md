# Tasks: change-install-005-prometheus-setup-command

- [ ] Read existing prometheus-cli command structure (main.rs, commands/mod.rs, commands/doctor.rs for pattern)
- [ ] Design component registry: list of all components with type (docker/launchd/binary/port) and detection logic
- [ ] Create `commands/setup.rs` with `SetupArgs` struct (non_interactive, dry_run, check flags)
- [ ] Implement `detect_component_status()` — probe Docker, launchd, PATH, port for each component
- [ ] Implement `print_status_table()` — colored output showing ✅/⚠️/❌ per component
- [ ] Implement `prompt_and_install()` — interactive loop with y/N/s per gap
- [ ] Implement `install_component()` — dispatch to correct installer per component type
- [ ] Implement `write_setup_state()` — serialize state to `~/.prometheus/setup-state.json`
- [ ] Register `Setup` in `commands/mod.rs`
- [ ] Add `Setup` variant and dispatch in `main.rs`
- [ ] `cargo build --release -p prometheus-cli` succeeds
- [ ] `cargo test -p prometheus-cli` passes
- [ ] Run `prometheus setup --check` and verify output
