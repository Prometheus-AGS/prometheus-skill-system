# Goals — phase-dsg-cli-foundation

## Context

`dsg` (disk-space-guardian) is a Rust CLI at
`/Users/gqadonis/Projects/prometheus/disk-space-guardian`. It is substantially
more complete than the cowork-integration reflection indicated — it has 1,635
lines of Rust across 5 modules (`main.rs`, `scanner.rs`, `ecosystems.rs`,
`safety.rs`, `config.rs`), builds cleanly, and has `status/scan/clean/caches`
all operational.

The gaps that remain are **integration** (install to PATH, wire into
`install-binaries.sh`) and **hardening** (CI, `--json` output, tests), not
initial scaffolding. The `cowork disk` stub in cowork v0.2.0 already delegates
to `dsg` — once `dsg` is on PATH, that delegation becomes real.

## Goals

- G-01: Install `dsg` binary to `~/.local/bin/dsg` and verify `dsg --version`
  returns `0.1.0` from PATH.
- G-02: Wire `dsg` build into `scripts/install-binaries.sh` (Path A: build from
  `tools/disk-space-guardian/dsg/`; Path B: GitHub Releases download fallback).
- G-03: Add `--json` output flag to `dsg status` and `dsg scan` so `cowork disk`
  and the skill layer can consume structured output.
- G-04: Add GitHub Actions CI workflow to `disk-space-guardian` (fmt + clippy +
  test + release binary builds).
- G-05: Wire `tools/disk-space-guardian` submodule pointer in prometheus-skill-pack
  to the tagged release commit and confirm `git submodule status` shows clean.
