---
id: change-dsg-001-capability-specs
title: Establish dsg capability specs + bind 4 open design decisions
phase: cowork-integration
priority: P0
effort: S
wave: 1
agent: general-purpose
status: done
gap_id: dsg-pre-cond
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/disk-space-guardian/openspec/specs/
  - /Users/gqadonis/Projects/prometheus/disk-space-guardian/docs/decisions.md
---

# change-dsg-001 — Establish dsg Capability Specs + Bind Design Decisions

## Context

`disk-space-guardian` is a separate Rust CLI project managed under the cowork-integration KBD phase. The repo was spec-only with extensive research docs but no formal OpenSpec capability specs and no binding records for 4 open design decisions identified in `docs/README.md`.

This change establishes the spec foundation that gates all implementation changes (dsg change-002 through change-005). It operates entirely in the dsg repo at `/Users/gqadonis/Projects/prometheus/disk-space-guardian`.

## What Was Done

### Capability Specs Created

All four files created in `/Users/gqadonis/Projects/prometheus/disk-space-guardian/openspec/specs/`:

1. **`cli.md`** — Complete Phase 1 command surface:
   - `dsg scan`, `dsg scan --deep`, `dsg scan --ecosystem <name>`, `dsg scan --stale <duration>`, `dsg scan --json`
   - `dsg clean`, `dsg clean --dry-run` (default), `dsg clean --force`, `dsg clean --target <path>`, `dsg clean --ecosystem <name>`
   - `dsg caches`, `dsg caches --list`, `dsg caches --clean <ecosystem>`
   - `dsg --version`, `dsg --help`
   - Exit codes: 0 = success, 1 = error, 2 = dry-run candidates found (nothing deleted)
   - Human table output format and JSON output schema for each command

2. **`config.md`** — TOML config schema at `~/.config/dsg/config.toml`:
   - `exclude_paths = []` — glob exclusion list
   - `min_age_days = 1` — staleness floor (24h default)
   - `min_size_mb = 10` — noise floor for scan results
   - `dry_run_default = true` — behavioral default
   - `log_level = "info"` — stderr log verbosity
   - Config loading: absent file = use defaults (no error); malformed = exit 1
   - Environment variable overrides (`DSG_CONFIG`, `DSG_LOG_LEVEL`, `DSG_DRY_RUN`)

3. **`safety.md`** — 7 safety rules with ordered pipeline:
   - RULE-01: Dry-run is default (requires `--force` to execute)
   - RULE-02: Trash via `trash` crate, never `std::fs::remove_*`
   - RULE-03: `lsof +D <path>` open-file check with 5s timeout
   - RULE-04: `git status --porcelain` uncommitted-change check
   - RULE-05: mtime minimum age guard (config `min_age_days`)
   - RULE-06: Exclusion list (built-in + user config)
   - RULE-07: Never delete dsg binary or config dir
   - Trash failure semantics (D-03 binding): per-item error → log + continue
   - Audit log format (JSONL at `~/.config/dsg/audit.log`)

4. **`scanner.md`** — Scanner algorithm spec:
   - `jwalk` as primary parallel walker; `walkdir` as verification fallback
   - `ScanResult` struct: `path`, `size_bytes`, `last_modified`, `entry_type`, `ecosystem`, `description`
   - `EntryType` enum: File, Dir, Symlink
   - `Ecosystem` enum: Rust, Node, Python, Go, Docker, Xcode, Homebrew
   - Sorting: `size_bytes` descending
   - Output formats: human table + JSON schema
   - Performance target: < 30s for 10 GB on NVMe SSD
   - `EcosystemDetector` trait: `name()`, `detect(root: &Path) -> Vec<PathBuf>`, `describe(path: &Path) -> String`
   - Phase 1 detectors: Rust, Node, Python, Go, Homebrew
   - Symlink handling (D-02 binding): follow for size, delete symlink not target
   - Scan depth limits: 8 levels (CWD scan), 20 levels (--deep)
   - Non-fatal per-entry errors (permission denied → skip, continue)

### Design Decisions Bound

`docs/decisions.md` created in the dsg repo with entries for:
- **D-01: lsof TOCTOU** — snapshot at operation start, warn on race, proceed
- **D-02: Symlink handling** — follow for size, delete symlink itself
- **D-03: Trash failure semantics** — per-item error → log + continue batch
- **D-04: mtime vs atime** — use mtime; atime unreliable under noatime/relatime mounts

## Verification

- All 4 spec files exist in `disk-space-guardian/openspec/specs/`
- `docs/decisions.md` has entries D-01 through D-04 with rationale
- Each spec cross-references the decision it binds
- Committed to dsg repo: `chore(specs): establish Phase 1 capability specs + bind design decisions`
