# change-prui-004-ci-workflow

## Summary

Add `.github/workflows/prometheus-research.yml` — a GitHub Actions CI workflow for
`substrate/prometheus-research`. The crate currently has no CI coverage. Template:
`.github/workflows/sovereign-sync.yml` (3-job matrix: fmt / clippy / test).

## Goal

G-04: Add GitHub Actions CI job for `prometheus-research`

## Files Changed

- `.github/workflows/prometheus-research.yml` — new CI workflow

## Acceptance Criteria

- [ ] Workflow triggers on push/PR affecting `substrate/prometheus-research/**` and the workflow file itself
- [ ] 3-job matrix: `fmt` (cargo fmt --check), `clippy` (deny warnings), `test` (cargo test)
- [ ] Uses `dtolnay/rust-toolchain@stable`
- [ ] Uses `actions/cache@v4` with key hashing `substrate/prometheus-research/Cargo.lock`
- [ ] YAML parses cleanly (`python3 -c "import yaml; yaml.safe_load(...)"`)
- [ ] `--manifest-path substrate/prometheus-research/Cargo.toml` used on all cargo commands

## Risk

Low. New file, no existing code modified.
