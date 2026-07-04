# Tasks: change-dsg-005-ecosystem-detectors

- [x] Create `dsg/src/ecosystems.rs` with 7 detector structs (rust, node, python, go, docker, xcode, homebrew) + all_detectors() factory
- [x] Add `collect_scan_roots` helper to `dsg/src/scanner.rs`
- [x] Wire `all_detectors()` into `cmd_scan` in `dsg/src/main.rs`
- [x] Wire per-item SafetyEngine loop into `cmd_clean` in `dsg/src/main.rs`
- [x] Add unit tests for detector matches() and detect_roots() (15 new tests)
- [x] Run `cargo build --release` from dsg repo root — must exit 0
- [x] Run `cargo test` from dsg repo root — all tests must pass (40/40)
- [x] Commit to disk-space-guardian repo (7716f87)
- [x] Update KBD orchestrator (progress.json, waypoint, position-reminder)
