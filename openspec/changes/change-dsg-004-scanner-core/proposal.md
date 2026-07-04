---
id: change-dsg-004-scanner-core
title: dsg scanner core — parallel filesystem walk, ScanResult, EcosystemDetector trait, reporting
phase: cowork-integration
priority: P0
effort: L
wave: 3
agent: general-purpose
status: done
gap_id: G-01-dsg
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/disk-space-guardian (dsg repo)
  - dsg/Cargo.toml (add jwalk, humansize, serde_json)
  - dsg/src/scanner.rs (NEW)
  - dsg/src/main.rs (wire cmd_scan, cmd_status)
---

# change-dsg-004 — dsg scanner core

## Context

The scanner is the engine that powers `dsg scan` and provides the data for
`dsg clean`. It must be fast (parallel walk via jwalk), safe (use SafetyEngine
from change-dsg-003), and extensible (EcosystemDetector trait for per-ecosystem
scanning in change-dsg-005).

## Scope

1. Add `jwalk = "0.8"`, `humansize = "2"`, `serde_json = "1"` to `dsg/Cargo.toml`
2. Create `dsg/src/scanner.rs`:
   - `EntryType` enum: `File`, `Directory`, `Symlink`
   - `ScanResult` struct: `{ path: PathBuf, size_bytes: u64, last_modified: SystemTime, entry_type: EntryType, ecosystem: Option<String> }`
   - `ScanOptions` struct: `{ deep: bool, ecosystem_filter: Option<String>, stale_secs: Option<u64>, min_size_bytes: u64 }`
   - `EcosystemDetector` trait: `fn name(&self) -> &str`, `fn detect_roots(&self, deep: bool) -> Vec<PathBuf>`, `fn matches(&self, path: &Path) -> bool`
   - `scan_directory(root: &Path, options: &ScanOptions, detectors: &[Box<dyn EcosystemDetector>]) -> Vec<ScanResult>`
   - `report_human(results: &[ScanResult])` — aligned table output
   - `report_json(results: &[ScanResult])` — serde_json serialized array
3. Wire `cmd_scan` in `main.rs` to use scanner
4. Wire `cmd_status` in `main.rs` to show disk usage summary

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass (14+ tests)
- `dsg scan` outputs table to stdout
- `dsg scan --json` outputs valid JSON array
- `dsg status` shows disk usage
- `dsg status --json` outputs valid JSON
