# Goals — phase-dsg-install-binaries-fix

## Context

`phase-dsg-cli-foundation` closed with a carry-forward: the `install_dsg()`
function in `scripts/install-binaries.sh` may reference the wrong Path A
target path. The Cargo workspace (`tools/disk-space-guardian/`) puts its
build output at `tools/disk-space-guardian/target/release/dsg` (workspace
root), NOT `tools/disk-space-guardian/dsg/target/release/dsg` (crate subdir).
If the script uses the crate-subdir path, Path A silently fails and falls
through to Path B on machines with Rust installed but no release artifact — or
fails entirely on machines without Rust.

Additionally, the release CI matrix from `v0.1.0` should be confirmed green
so that Path B (GitHub Releases download) is actually functional.

## Goals

- G-01: Read `scripts/install-binaries.sh` `install_dsg()` and confirm which
  target path it references (workspace root vs crate subdir).
- G-02: If incorrect, fix `install_dsg()` Path A to use
  `tools/disk-space-guardian/target/release/dsg`.
- G-03: Confirm the `v0.1.0` GitHub Actions release matrix completed green
  and all 4 binary artifacts are present in the GitHub Release.
- G-04: Run `bash scripts/install-binaries.sh` on this machine to verify
  end-to-end install works (Path A build + copy).
