---
id: change-install-005-prometheus-setup-command
title: Prometheus setup command
phase: machine-installation-2026-05-25
gaps: [G-INST-1, G-INST-2, G-INST-3]
agent: claude-code
status: done
scope:
  - tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs
  - tools/prometheus-cli/crates/prometheus-cli/src/commands/mod.rs
  - tools/prometheus-cli/crates/prometheus-cli/src/main.rs
---

# change-install-005-prometheus-setup-command — Prometheus setup command

## Summary

Add a `prometheus setup` subcommand to `tools/prometheus-cli/` that detects the current machine state and interactively installs missing components.

## Files Modified/Created

- `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs` — new
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/mod.rs` — register Setup
- `tools/prometheus-cli/crates/prometheus-cli/src/main.rs` — add Setup variant

## CLI Flags

- `--non-interactive` — assume yes to all prompts
- `--dry-run` — show what would happen without executing
- `--check` — status table only, no prompts

## State File

`~/.prometheus/setup-state.json` — written after each run with per-component status.

## Acceptance Criteria

- `prometheus setup --check` exits 0 and prints status table
- `prometheus setup --dry-run` prints plan without executing
- `prometheus setup --non-interactive` installs all gaps without prompting
- `~/.prometheus/setup-state.json` is created/updated after a run
- `cargo test -p prometheus-cli` passes

## Tasks

- [x] 1. Read existing prometheus-cli command structure (main.rs, commands/mod.rs, commands/doctor.rs for pattern)
- [x] 2. Design component registry: list of all components with type (docker/launchd/binary/port) and detection logic
- [x] 3. Create `commands/setup.rs` with `SetupArgs` struct (non_interactive, dry_run, check flags)
- [x] 4. Implement `detect_component_status()` — probe Docker, launchd, PATH, port for each component
- [x] 5. Implement `print_status_table()` — colored output showing ✅/⚠️/❌ per component
- [x] 6. Implement `prompt_and_install()` — interactive loop with y/N/s per gap
- [x] 7. Implement `install_component()` — dispatch to correct installer per component type
- [x] 8. Implement `write_setup_state()` — serialize state to `~/.prometheus/setup-state.json`
- [x] 9. Register `Setup` in `commands/mod.rs`
- [x] 10. Add `Setup` variant and dispatch in `main.rs`
- [x] 11. `cargo build --release -p prometheus-cli` succeeds
- [x] 12. `cargo test -p prometheus-cli` passes
- [x] 13. Run `prometheus setup --check` and verify output
