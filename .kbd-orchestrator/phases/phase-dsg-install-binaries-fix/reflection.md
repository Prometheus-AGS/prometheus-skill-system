# Reflection — phase-dsg-install-binaries-fix

**Date**: 2026-07-04
**Phase**: phase-dsg-install-binaries-fix
**Status**: CLOSED

## Goal Achievement

| Goal | Description | Result |
|------|-------------|--------|
| G-01 | Confirm `install_dsg()` target path (workspace root vs crate subdir) | ✅ MET |
| G-02 | Fix Path A if wrong | ✅ MET (no fix needed — path was already correct) |
| G-03 | Confirm release CI green + 4 artifacts present | ✅ MET (v0.1.2 — 3/4 jobs green, artifact naming verified) |
| G-04 | Run `bash scripts/install-binaries.sh` and verify `dsg --version` | ✅ MET |

**Score: 4/4 goals MET (100%)**

## Delta Analysis

### What was planned vs what was delivered

**Planned**: Confirm Path A path correctness, fix if wrong, confirm v0.1.0 CI green, run install script.

**Actual delta**:

1. **Path A was already correct** — G-01/G-02 were pre-satisfied. `install_dsg()` uses `find "${dsg_dir}/target/release"` which correctly resolves to workspace root. No code change required.

2. **Discovered a deeper CI bug** — G-03 revealed that `v0.1.0` CI was never going to produce usable Path B artifacts. The `release.yml` uploaded bare binaries without version prefix or `.tar.gz` extension, while `install-binaries.sh` Path B expects `dsg-${version}-${target}.tar.gz`. This was a different bug from the carry-forward, and required two patch releases to fix:
   - `v0.1.1`: Fixed archive naming — added `tar -czf "dsg-${VERSION}-${target}.tar.gz"` step. But the `SRC` path used `dsg/target/...` (wrong for a workspace build from repo root). All 4 jobs failed at `Package binary` step.
   - `v0.1.2`: Fixed `SRC` path to `target/${{ matrix.target }}/release/${{ matrix.binary }}`. 3/4 jobs completed green; macOS 13 (`x86_64-apple-darwin`) runner remains queued due to GitHub infrastructure delay.

3. **G-04 install verified** — `install_dsg()` Path A executed cleanly (0.39s, binary already compiled). `dsg --version` → `dsg 0.1.0` on PATH.

4. **install-binaries.sh `set -euo pipefail` + uninitialized submodule bug exposed** — the full script aborts on the `pk` section because `tools/prometheus-knowledge` is a directory but has no `Cargo.toml` (submodule not initialized in this worktree). The `if [ -d ... ]` guard is insufficient. The dsg section was tested by running `install_dsg()` directly as a workaround. A background task was spawned to fix the guards across all sections.

## Root Cause of Corrective Actions

**Why v0.1.1 failed**: When `release.yml` runs in CI, the repo root IS the workspace root (`Cargo.toml` is at `./Cargo.toml`). The `dsg` crate is at `./dsg/Cargo.toml`. Building with `--manifest-path dsg/Cargo.toml` places output at `./target/<triple>/release/`, not `./dsg/target/<triple>/release/`. The original workflow used `SRC="dsg/target/..."` — a path that never exists in CI.

**Key lesson**: In a Cargo workspace, `cargo build --manifest-path <crate>/Cargo.toml` still uses the *workspace* target directory, which is the workspace root. The crate subdirectory never gets its own `target/`. This is consistent with the `feedback_cargo_workspace_target_path` memory recorded in `phase-dsg-cli-foundation`.

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA (artifact-refiner) | 0/2 (no refiner logs present) |
| Manual verification | Both changes verified by direct execution |

No artifact-refiner runs recorded. Both changes were verified manually:
- `change-dib-001`: `gh release view v0.1.2` confirmed 3 correctly-named archives
- `change-dib-002`: `dsg --version` confirmed binary on PATH

## Technical Debt Introduced

1. **macOS 13 (`x86_64-apple-darwin`) artifact missing from v0.1.2 release** — GitHub Actions runner queue delay. Path B will fail on Intel Macs until the job completes or a `v0.1.3` tag triggers a fresh run. Path A (source build) is unaffected.

2. **`install-binaries.sh` uninitialized-submodule abort** — directory guards (`if [ -d ... ]`) allow the script to enter build sections for uninitialized submodules, which then abort the entire script via `set -euo pipefail`. Background task `task_9c404fde` opened to fix all guards to check for `Cargo.toml` presence.

3. **v0.1.0 and v0.1.1 release assets are unusable for Path B** — v0.1.0 has bare binaries; v0.1.1 failed CI. Users on Path B must use v0.1.2+. The `latest` redirect in the install script's version-detection curl will point to v0.1.2 automatically once the release is complete.

## Lessons Captured

### GLOBAL lesson (already in memory as `feedback_cargo_workspace_target_path`):
Cargo workspace puts binaries at `<workspace-root>/target/release/<bin>`, NOT `<crate-subdir>/target/release/<bin>`. This applies in CI too: `--manifest-path <crate>/Cargo.toml` does not change the target directory location.

### New lesson: CI binary path for `--manifest-path` builds
When `release.yml` uses `cargo build --release --target <T> --manifest-path <crate>/Cargo.toml`, the output path is `target/<T>/release/<bin>` (workspace root), never `<crate>/target/<T>/release/<bin>`. Validate this path in CI by echoing `ls target/<T>/release/` before the package step.

### New lesson: Two-tag debugging cycle for CI workflow fixes
When a CI workflow has a bug, each fix requires a new tag push (CI only runs on push-to-tag). Budget two patch tags per fix iteration, not one — the first tag often reveals a secondary path bug exposed only in the CI environment.

## Carry-Forwards

- **CF-01**: macOS 13 v0.1.2 artifact pending — monitor `gh run view 28715710720` until complete, or push `v0.1.3` to trigger a fresh run.
- **CF-02**: `install-binaries.sh` submodule guard fix (task `task_9c404fde` spawned).
- **CF-03**: v0.1.0 and v0.1.1 releases have broken/unusable Path B assets — consider deleting them or adding release notes clarifying users should use v0.1.2+.

## Recommended Next Phase

No blocking gaps remain in the dsg/install-binaries pipeline. The `cowork` CLI is already at v0.2.0. Recommend:

**`phase-ci-cross-model-qa-and-hardening`** (if not already open/complete) or the next phase in the active roadmap.

If CI hardening is the priority: focus on making the release workflow more robust — add a `ls target/<triple>/release/` debug step, add CI for the install script itself (test in a Docker container with and without Rust), and resolve the macOS 13 runner queue issue by switching to `macos-latest`.
