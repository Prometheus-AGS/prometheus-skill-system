# Tasks: change-credibility-009-rust-ci

- [ ] Read current `.github/workflows/validate.yml` to understand existing job structure
- [ ] Add `forge-rs-test` job (fmt + clippy + test) targeting `tools/forge-rs/`
- [ ] Use `dtolnay/rust-toolchain@stable` with rustfmt and clippy components
- [ ] Add `actions/cache` for cargo registry and tools/forge-rs/target/
- [ ] Verify YAML is valid (no tabs, correct indentation)
- [ ] Add `forge-rs-test` to the PR required checks list (manual — note in PR description)
- [ ] Test: confirm job appears in GitHub Actions on push to a test branch
