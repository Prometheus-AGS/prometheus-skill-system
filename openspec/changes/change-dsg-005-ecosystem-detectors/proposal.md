---
id: change-dsg-005-ecosystem-detectors
title: dsg ecosystem detectors — Rust, Node, Python, Go, Docker, Xcode, Homebrew + clean integration
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
  - dsg/src/ecosystems.rs (NEW — 7 detector structs)
  - dsg/src/scanner.rs (add collect_scan_roots helper)
  - dsg/src/main.rs (wire detectors into cmd_scan + cmd_clean per-item loop)
---

# change-dsg-005 — ecosystem detectors + clean integration

## Context

The EcosystemDetector trait landed in change-dsg-004. This change fills in the
concrete detector implementations for the seven ecosystems dsg targets, and
wires them into both `dsg scan` (tagging + root discovery) and `dsg clean`
(per-item safety-check loop via SafetyEngine).

## Ecosystem Targets

| Ecosystem | Key Directories | What gets cleaned |
|---|---|---|
| rust | ~/.cargo/registry, ~/.cargo/git, target/ dirs | Registry cache, old artifacts |
| node | ~/.npm/_cacache, node_modules | npm cache, orphaned node_modules |
| python | ~/.cache/pip, ~/.venv, __pycache__ | pip cache, venvs, pyc cache |
| go | ~/go/pkg/mod/cache | module download cache |
| docker | dangling images/volumes (docker CLI) | Prune output |
| xcode | ~/Library/Developer/Xcode/DerivedData | Build artifacts |
| homebrew | ~/Library/Caches/Homebrew | Formula download cache |

## Scope

1. Create `dsg/src/ecosystems.rs`:
   - `RustDetector`, `NodeDetector`, `PythonDetector`, `GoDetector`,
     `DockerDetector`, `XcodeDetector`, `HomebrewDetector`
   - Each implements `EcosystemDetector`: `name()`, `detect_roots(deep)`, `matches(path)`
   - `all_detectors() -> Vec<Box<dyn EcosystemDetector>>` factory
2. Update `dsg/src/scanner.rs`:
   - `collect_scan_roots(opts: &ScanOptions, detectors: &[Box<dyn EcosystemDetector>]) -> Vec<PathBuf>`
3. Update `dsg/src/main.rs`:
   - `cmd_scan`: pass `all_detectors()` to scanner; use `collect_scan_roots` for deep scan
   - `cmd_clean`: per-item loop — `verify_activity` → `age_guard` → `should_exclude` → `move_to_trash`

## Verification

- `cargo build --release` exits 0
- `cargo test` all tests pass (30+ tests)
- `dsg scan` lists reclaimable items with ecosystem tags
- `dsg scan --ecosystem rust` limits output to Rust items
- `dsg clean --dry-run` shows preview with safety checks
- `dsg clean --force` would trash (CI: no real files cleaned)
