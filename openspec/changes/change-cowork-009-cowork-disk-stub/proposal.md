---
id: change-cowork-009-cowork-disk-stub
title: cowork disk stub subcommand — delegates to dsg CLI
phase: cowork-integration
priority: P0
effort: S
wave: 3
agent: general-purpose
status: done
gap_id: G-03
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/disk.rs (NEW)
  - cli/src/commands/mod.rs (add disk module)
  - cli/src/main.rs (add Disk variant + dispatch)
---

# change-cowork-009 — cowork disk stub subcommand

## Context

The `cowork disk` subcommand is a thin proxy to the `dsg` (disk-space-guardian)
CLI. When `dsg` is on PATH it delegates all work there. When `dsg` is absent,
it emits an actionable install message with the releases URL. Satisfies G-03
(prometheus-pack awareness) and G-05 (integration pipeline).

## Scope

1. Create `cli/src/commands/disk.rs` with:
   - `is_dsg_available() -> bool` — checks `which dsg` or PATH lookup
   - `execute_status()` — runs `dsg status --json` if present; else error
   - `execute_scan(deep: bool, ecosystem: Option<&str>)` — delegates `dsg scan`
   - `execute_clean(dry_run: bool, ecosystem: Option<&str>)` — delegates `dsg clean`
   - Graceful degradation: non-zero exit with install instructions when dsg absent
   - Install URL: `https://github.com/GQAdonis/disk-space-guardian/releases/latest`
2. Wire `Disk` variant into `Commands` enum in `main.rs`
3. Register `pub mod disk` in `commands/mod.rs`

## Sub-command surface

```
cowork disk status                       # dsg status --json
cowork disk scan [--deep] [--ecosystem rust]   # dsg scan [flags]
cowork disk clean [--dry-run] [--ecosystem rust]  # dsg clean [flags]
```

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass
- When dsg absent: `cowork disk status` exits non-zero with install URL in stderr
