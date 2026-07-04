# Plan — phase-dsg-install-binaries-fix

_Generated: 2026-07-04_

## Summary

2 sequential changes. G-01 and G-02 were pre-satisfied (Path A is correct).
The real work is fixing the Path B archive naming mismatch and then verifying
the full install pipeline end-to-end.

## Root Cause

`install-binaries.sh` Path B downloads `dsg-${version}-${target}.tar.gz` but
`release.yml` uploads bare binaries (`dsg-aarch64-apple-darwin`, no version
prefix, no tar.gz wrapper). These never intersect — Path B will always fail
on a machine without Rust until the naming is aligned.

Fix direction: update `release.yml` to tar the binary into
`dsg-${GITHUB_REF_NAME#v}-${target}.tar.gz` before uploading. This preserves
the existing install-script logic intact (download → extract → copy).

## Changes

| # | ID | Wave | Effort | Goals |
|---|---|---|---|---|
| 1 | `change-dib-001-release-archive-format` | 1 | S | G-03 |
| 2 | `change-dib-002-e2e-verify` | 2 | XS | G-03, G-04 |

## Ordering

Wave 1 must be committed and tagged before Wave 2 can confirm CI green. The
new tag will be `v0.1.1` to re-trigger the fixed `release.yml`. (v0.1.0 CI is
still queued/stalled — a new tag is cleaner than investigating the stalled run.)

## Goal Mapping

| Goal | Change | Status |
|------|--------|--------|
| G-01: Path A target path | (pre-done) | MET |
| G-02: Fix Path A | (not needed) | MET |
| G-03: Release CI green + 4 artifacts | change-dib-001 + change-dib-002 | PENDING |
| G-04: End-to-end install verification | change-dib-002 | PENDING |
