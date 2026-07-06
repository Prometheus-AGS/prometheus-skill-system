# change-hard-001-submodule-guards

**Status**: done

## Summary

Replace three `[ -d ]` guards with `[ -f Cargo.toml ]` guards in
`scripts/install-binaries.sh` for the `prometheus-knowledge`, `liter-llm`,
and `surreal-memory-server` tool sections.

## Problem

`install-binaries.sh` runs with `set -euo pipefail`. Three tool sections
(lines 47, 58, 77) use `if [ -d "${REPO_ROOT}/tools/<name>" ]` to guard
optional Rust builds. When these git submodules are uninitialized, the
directory exists (empty mount) so the guard passes, but there is no
`Cargo.toml` inside. `cargo build` fails with "could not find Cargo.toml",
and `set -euo pipefail` aborts the entire script — including the `dsg` section
that follows. Users get a broken install with no useful error message.

## Fix

Replace each guard with `[ -f "${REPO_ROOT}/tools/<name>/Cargo.toml" ]`.
This is already the correct pattern used by the `dsg` section (line 219):
`[ -f "${dsg_dir}/Cargo.toml" ]`.

## Files changed

- `scripts/install-binaries.sh` — lines 47, 58, 77 only

## Acceptance criteria

- [ ] Line 47 uses `[ -f "${REPO_ROOT}/tools/prometheus-knowledge/Cargo.toml" ]`
- [ ] Line 58 uses `[ -f "${REPO_ROOT}/tools/liter-llm/Cargo.toml" ]`
- [ ] Line 77 uses `[ -f "${REPO_ROOT}/tools/surreal-memory-server/Cargo.toml" ]`
- [ ] `bash scripts/install-binaries.sh` completes end-to-end (no abort) on this machine
- [ ] `dsg --version` returns expected version after the script completes
