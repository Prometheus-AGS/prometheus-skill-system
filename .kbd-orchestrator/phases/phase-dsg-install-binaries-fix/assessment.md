# Assessment — phase-dsg-install-binaries-fix

_Generated: 2026-07-04_

## G-01: Confirm install_dsg() Path A target path

**FINDING: Path A is CORRECT — no fix needed.**

`install-binaries.sh` line 224:
```bash
dsg_bin="$(find "${dsg_dir}/target/release" -maxdepth 1 -name "dsg" -type f 2>/dev/null | head -1)"
```

`dsg_dir = "${REPO_ROOT}/tools/disk-space-guardian"` so the full search path is
`tools/disk-space-guardian/target/release/dsg` — exactly the Cargo workspace
root output path. The concern from the carry-forward was unfounded; Path A is
safe.

## G-02: Fix Path A if incorrect

**NOT NEEDED.** Path A is already correct (see G-01).

## G-03: Confirm release CI matrix green + 4 binary artifacts

**GAP — BLOCKED.**

`gh run list` at assess time shows run `28714222694` still `queued` after 23
minutes. This is anomalous — a queued run this long suggests either a runner
pool bottleneck or the run never advanced past the queue step.

**Critical secondary finding: Path B archive naming mismatch.**

`install-binaries.sh` expects archive name format:
```
dsg-${version}-${target}.tar.gz
# e.g.: dsg-0.1.0-aarch64-apple-darwin.tar.gz
```

`release.yml` uploads files named:
```
dsg-aarch64-apple-darwin        (bare binary, no version, no extension)
dsg-x86_64-apple-darwin
dsg-x86_64-unknown-linux-musl
dsg-x86_64-pc-windows-msvc.exe
```

The download URL constructed by `install-binaries.sh` would be:
```
https://github.com/GQAdonis/disk-space-guardian/releases/latest/download/dsg-0.1.0-aarch64-apple-darwin.tar.gz
```

But the actual artifact URL will be:
```
https://github.com/GQAdonis/disk-space-guardian/releases/latest/download/dsg-aarch64-apple-darwin
```

**Path B will fail on every platform** until either:
- `release.yml` is updated to produce versioned tar.gz archives, OR
- `install-binaries.sh` Path B is updated to download bare binaries

This is a blocking defect for fresh-machine installs without Rust.

## G-04: End-to-end bash scripts/install-binaries.sh verification

**NOT YET RUN** — blocked by G-03 (Path B broken; Path A works locally but
install script should be verified end-to-end after any fix).

## Gap Summary

| Goal | Status | Work Required |
|------|--------|--------------|
| G-01: Path A target path | **MET** (pre-done) | None |
| G-02: Fix Path A | **MET** (no fix needed) | None |
| G-03: Release CI + Path B naming | **GAP** | Fix release.yml OR fix install script Path B |
| G-04: End-to-end install verification | **BLOCKED** on G-03 | Run after G-03 fixed |

## Revised Scope: 2 real changes

| Change | What | Effort |
|--------|------|--------|
| change-dib-001-release-archive-format | Fix naming mismatch: update `release.yml` to produce versioned tar.gz archives matching Path B expectations | S |
| change-dib-002-e2e-verify | Verify CI matrix completed green; run `bash scripts/install-binaries.sh` end-to-end | XS |

## Open Questions

**OQ-01**: The v0.1.0 release CI has been queued for 23+ minutes. Is the
run actually executing (just slow build matrix) or is it stuck? Check
`gh run view 28714222694` for detailed status. If stuck, may need to re-tag
`v0.1.0` after fixing `release.yml`.

**OQ-02**: Should the archive format be `tar.gz` wrappers (more portable,
consistent with install-script expectations) or should Path B in
`install-binaries.sh` be simplified to download bare binaries (simpler CI)?
Recommendation: fix `release.yml` to produce tar.gz archives — it keeps the
install script's existing download/extract logic intact.
