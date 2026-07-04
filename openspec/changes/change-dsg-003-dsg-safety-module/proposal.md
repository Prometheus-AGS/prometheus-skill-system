---
id: change-dsg-003-dsg-safety-module
title: dsg safety module — dry-run/execute split, trash, lsof, exclusions, age guard
phase: cowork-integration
priority: P0
effort: M
wave: 3
agent: general-purpose
status: done
gap_id: G-01-dsg
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/disk-space-guardian (dsg repo)
  - dsg/Cargo.toml (add trash crate)
  - dsg/src/safety.rs (NEW)
  - dsg/src/main.rs (integrate SafetyEngine into cmd_clean; add status subcommand)
  - dsg/src/config.rs (add exclude_paths field)
---

# change-dsg-003 — dsg safety module

## Context

The safety module is the core safety guarantee of dsg: nothing is deleted
without going through trash, activity checks, and age guards. The dry-run
default means users cannot accidentally delete by running dsg clean without
--force. Satisfies the safety requirement from dsg goals.

## Design Decisions (from docs/decisions.md)

- D-01: lsof TOCTOU — snapshot at scan time; warn if race detected post-snapshot
- D-02: symlink handling — check path's mtime (not symlink source)
- D-03: trash failure semantics — abort item, log warning, continue batch
- D-04: mtime anchoring — mtime is the staleness anchor

## Scope

1. Add `trash = "0.3"` to `dsg/Cargo.toml`
2. Create `dsg/src/safety.rs`:
   - `ActivityCheck` enum: `Idle`, `ActiveProcesses(Vec<String>)`, `GitDirty`
   - `SafetyEngine { dry_run: bool, min_age_secs: u64, config: Arc<Config> }`
   - `verify_activity(path: &Path) -> Result<ActivityCheck>`: runs `lsof +D`; runs `git status --porcelain` if git repo
   - `move_to_trash(path: &Path) -> Result<()>`: uses `trash::delete()`
   - `should_exclude(path: &Path) -> bool`: checks config.exclude_paths patterns
   - `age_guard(path: &Path) -> Result<bool>`: checks mtime >= min_age_secs
3. Add `exclude_paths: Vec<String>` to `Config` in `config.rs`
4. Update `cmd_clean` in `main.rs` to construct `SafetyEngine` and run checks
5. Add `Status` subcommand stub to `main.rs`

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass
- `dsg clean --dry-run` prints preview, exits 2 (dry-run sentinel)
- `dsg clean --force` would trash (integration: no real files cleaned in CI)
