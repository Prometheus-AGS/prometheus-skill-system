---
id: change-cowork-010-github-releases-cicd
title: cowork GitHub Releases CI/CD — cargo-dist cross-platform binary workflow
phase: cowork-integration
priority: P1
effort: M
wave: 4
agent: general-purpose
status: done
gap_id: G-05-cowork
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (cowork repo)
  - .github/workflows/release.yml (NEW)
  - .github/workflows/ci.yml (NEW)
  - cli/Cargo.toml (add cargo-dist metadata)
---

# change-cowork-010 — GitHub Releases CI/CD

## Context

The cowork binary needs to be distributable so that install-binaries.sh
(change-cowork-011) can pull a pre-built binary from GitHub Releases.
cargo-dist is the standard tool for cross-platform Rust binary releases.

## Scope

1. Create `.github/workflows/ci.yml` — runs on push/PR to main:
   - `cargo fmt --check`
   - `cargo clippy --all-targets`
   - `cargo test`
   All run from `cli/` directory.

2. Create `.github/workflows/release.yml` — triggers on `v*` tag push:
   - matrix: ubuntu-latest, macos-latest, windows-latest
   - cross-compile to: x86_64-unknown-linux-musl, aarch64-apple-darwin,
     x86_64-apple-darwin, x86_64-pc-windows-msvc
   - upload binaries to GitHub Release via softprops/action-gh-release
   - artifact naming: `cowork-{version}-{target}.tar.gz` (`.zip` for Windows)

3. Add `[package.metadata.dist]` to `cli/Cargo.toml`:
   - `targets = [...]`
   - `install-updater = false`

## Verification

- Workflow YAML files parse without error (yamllint / actionlint)
- CI workflow covers fmt + clippy + test
- Release workflow has matrix for all 4 targets
- Artifact naming follows `cowork-{version}-{target}` pattern
