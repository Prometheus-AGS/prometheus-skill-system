# Tasks: change-dsg-004-scanner-core

- [x] Add `jwalk = "0.8"`, `humansize = "2"`, `serde_json = "1"` to `dsg/Cargo.toml`
- [x] Create `dsg/src/scanner.rs` with ScanResult, ScanOptions, EcosystemDetector trait, scan_directory, report_human, report_json
- [x] Wire `cmd_scan` in `dsg/src/main.rs` to use scanner (table + JSON modes)
- [x] Wire `cmd_status` in `dsg/src/main.rs` to show disk usage summary
- [x] Add unit tests for scanner (scan a temp dir, verify ScanResult fields)
- [x] Run `cargo build --release` from dsg repo root — must exit 0
- [x] Run `cargo test` from dsg repo root — all tests must pass (25/25)
- [x] Commit to disk-space-guardian repo
