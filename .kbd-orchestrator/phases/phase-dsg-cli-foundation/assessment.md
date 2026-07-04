# Assessment — phase-dsg-cli-foundation

_Generated: 2026-07-04_

## Reality Check vs Goals

The cowork-integration reflection described `dsg` as "spec-only" — that was
stale. The actual state of
`/Users/gqadonis/Projects/prometheus/disk-space-guardian` is:

| Dimension | State |
|-----------|-------|
| Rust workspace | `Cargo.toml` + `dsg/` crate present |
| Source | 1,635 lines across 5 modules (main, scanner, ecosystems, safety, config) |
| Build | `cargo build --release` — Finished in 3.55s |
| Commands | `status`, `scan`, `clean`, `caches` all implemented |
| `--json` flag | Already present on `status` and `scan` |
| Tests | 40 passing; 0 failures |
| CI | `.github/workflows/ci.yml` — check, clippy, fmt (49 lines; no release job) |
| PATH install | NOT installed — binary is only in `target/release/`, not `~/.local/bin/` |
| `install-binaries.sh` | `install_dsg()` defined AND called at line 285 — **already wired** |
| Remote commits | 5 commits ahead of `origin/main` — UNPUSHED |
| Submodule pointer | `852ab4c` (heads/main, pre-feature commits) — stale |

## Gap Analysis per Goal

### G-01: Install dsg to ~/.local/bin/dsg
**GAP — NOT DONE.**
`~/.local/bin/dsg` does not exist. `which dsg` returns nothing.
The binary exists at
`/Users/gqadonis/Projects/prometheus/disk-space-guardian/target/release/dsg`
but has never been copied to PATH.

Fix: run `bash scripts/install-binaries.sh` OR copy manually — either works
since `install_dsg()` is already coded correctly in `install-binaries.sh`.

### G-02: Wire dsg build into scripts/install-binaries.sh
**ALREADY DONE — no work needed.**
`install_dsg()` at line 216 of `scripts/install-binaries.sh` implements
Path A (source build from `tools/disk-space-guardian`) and Path B (GitHub
Releases download fallback). It is called unconditionally at line 285.
The only remaining issue is that the submodule pointer is stale (see G-05).

### G-03: --json output flag on dsg status and dsg scan
**ALREADY DONE — no work needed.**
`dsg status --json` and `dsg scan --json` are implemented in `main.rs`
(lines 42 and 53 of the `Commands` enum; `scanner::report_status_json` and
`scanner::report_json` are the output handlers). Verified working.

### G-04: GitHub Actions CI workflow
**PARTIAL — basic CI exists, release workflow missing.**
`.github/workflows/ci.yml` covers check + clippy + fmt on `ubuntu-latest`.
There is NO `release.yml` for cross-platform binary builds. The install
script's Path B download URL points to
`github.com/GQAdonis/disk-space-guardian/releases/latest` — if no release
artifacts exist there, Path B silently fails and callers fall back to source
build. A release workflow is needed to publish binaries for fresh-machine
installs (no Rust toolchain present).

### G-05: Advance tools/disk-space-guardian submodule pointer
**GAP — stale.**
The gitlink points to `852ab4c` (heads/main, pre-feature state). The local
dsg repo has 5 feature commits (ecosystem detectors, scanner, safety, Cargo
scaffold, capability specs) that are UNPUSHED to `origin/main`. The pointer
cannot be advanced until those commits are pushed and tagged.

## Revised Phase Scope

Given the actual state, the phase has three real work items, not five:

| Real Gap | Effort |
|----------|--------|
| Push 5 dsg commits to origin main + tag v0.1.0 | XS |
| Add release.yml for cross-platform binary builds | S |
| Advance tools/disk-space-guardian submodule pointer + install dsg to PATH | XS |

G-02 and G-03 are already complete — they should be noted as MET on delivery
rather than re-implemented.

## Open Questions

**OQ-01**: Does `github.com/GQAdonis/disk-space-guardian` exist as a public
repo? The remote is `git@github.com:GQAdonis/disk-space-guardian.git` — if
the repo is private or doesn't exist, the push will fail. Must confirm before
executing the push change.

**OQ-02**: The `dsg caches` subcommand has `[stub]` bodies with
`change-dsg-005` references. Should this phase fully implement `caches`, or
defer it? Assessment recommendation: defer — the stub emits a clear message
and doesn't block any integration goal.
