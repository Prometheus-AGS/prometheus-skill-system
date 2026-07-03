---
id: change-credibility-009-rust-ci
title: Add forge-rs-test CI job to validate.yml
phase: phase-credibility-closure
priority: P2
effort: S
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-B
verdict: BUILD
scope:
  - .github/workflows/validate.yml
---

# change-credibility-009 — Add forge-rs-test CI job to validate.yml

## Context

The existing `.github/workflows/validate.yml` has no job that builds or tests the `tools/forge-rs` Rust workspace. The sovereign-sync substrate has a dedicated workflow (`sovereign-sync.yml`), but forge-rs is completely untested in CI. Any regression in forge-rs would ship silently.

## Scope

Add a `forge-rs-test` job to `.github/workflows/validate.yml` that:
1. Checks out the repo
2. Installs stable Rust via `dtolnay/rust-toolchain@stable`
3. Caches `~/.cargo` with `actions/cache`
4. Runs `cargo fmt --check --manifest-path tools/forge-rs/Cargo.toml`
5. Runs `cargo clippy --manifest-path tools/forge-rs/Cargo.toml --workspace -- -D warnings`
6. Runs `cargo test --manifest-path tools/forge-rs/Cargo.toml --workspace`

## Implementation Notes

```yaml
  forge-rs-test:
    name: forge-rs (fmt + clippy + test)
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: tools/forge-rs
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            tools/forge-rs/target/
          key: forge-rs-${{ runner.os }}-cargo-${{ hashFiles('tools/forge-rs/Cargo.lock') }}
          restore-keys: forge-rs-${{ runner.os }}-cargo-

      - name: fmt
        run: cargo fmt --check --all

      - name: clippy
        run: cargo clippy --all --all-features -- -D warnings

      - name: test
        run: cargo test --all
```

Add `forge-rs-test` to the required status checks in the repository branch protection rules (manual step, documented in tasks.md).

## Verification

- PR with the new job opens and the `forge-rs-test` job runs and passes
- Deliberately introduce a fmt violation → job fails as expected
