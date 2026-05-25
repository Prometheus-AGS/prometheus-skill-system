# change-install-005-prometheus-setup-command

**Phase**: machine-installation-2026-05-25  
**Status**: PENDING  
**Gaps closed**: G-INST-1, G-INST-2, G-INST-3

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
