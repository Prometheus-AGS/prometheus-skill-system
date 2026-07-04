# Tasks — change-dib-002-e2e-verify

- [ ] Check `gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml --limit 5` for v0.1.1 success
- [ ] List release assets: `gh release view v0.1.1 --repo GQAdonis/disk-space-guardian --json assets`
- [ ] Run `bash scripts/install-binaries.sh` and capture output
- [ ] Verify `dsg --version` returns the expected version from PATH
