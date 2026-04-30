---
id: change-001-forge-package-librefang
title: "forge package-librefang subcommand"
phase: phase-librefang-wasm-onramp
gaps: [G1, G4]
priority: P0
effort: S
agent: native-tool
evolver_item_id: null
status: proposed
---

# change-001 — forge package-librefang subcommand

## Context

The `forge` CLI (`tools/forge-rs/crates/forge-cli/`) has 9 subcommands. None produces a LibreFang `.lf-skill.zip`. This change adds `forge package-librefang <agent-dir>` which reads `skill.toml`, optionally builds the WASM binary, and writes a zip compatible with `librefang skill install`.

The `zip` crate is not yet in the workspace — must be added.

LibreFang's `install_from_bytes` expects a standard zip with `skill.toml` + `<name>.wasm` at archive root (verified from `crates/librefang-skills/src/clawhub.rs`).

## Files to Change

| File | Action |
|------|--------|
| `tools/forge-rs/Cargo.toml` | Add `zip = "2"` to `[workspace.dependencies]` |
| `tools/forge-rs/crates/forge-cli/Cargo.toml` | Add `zip = { workspace = true }` to `[dependencies]` |
| `tools/forge-rs/crates/forge-cli/src/main.rs` | Add `PackageLibrefang` to `Commands` enum + match arm |

## Tasks

- [ ] Add `zip = "2"` to workspace `[workspace.dependencies]` in `tools/forge-rs/Cargo.toml`
- [ ] Add `zip = { workspace = true }` to `[dependencies]` in `tools/forge-rs/crates/forge-cli/Cargo.toml`
- [ ] Add `PackageLibrefang` variant to `Commands` enum with `agent_dir: PathBuf`, `--no-build`, `--output` flags
- [ ] Implement match arm: read `skill.toml` → optional `cargo build` → zip `skill.toml` + `.wasm` → write file
- [ ] Update doc comment at top of `main.rs` to list new subcommand
- [ ] `cargo build -p forge-cli --release` succeeds with 0 new warnings
- [ ] `cargo clippy -p forge-cli -- -D warnings` clean
- [ ] Manual spot check: `forge package-librefang skills/rust/librefang-wasm-skill --no-build` + `unzip -l`

## Acceptance Criteria

- [ ] `forge package-librefang --help` shows subcommand with flags
- [ ] Produces `<name>-<version>.lf-skill.zip` at cwd (or `--output` path)
- [ ] Zip contains `skill.toml` and `<name>.wasm` at root
- [ ] Missing `skill.toml` → clear error, non-zero exit
- [ ] Missing `.wasm` with `--no-build` → clear error, non-zero exit
