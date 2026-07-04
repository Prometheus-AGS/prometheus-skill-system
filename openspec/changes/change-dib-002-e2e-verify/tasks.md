# Tasks — change-dib-002-e2e-verify

- [x] Check `gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml --limit 5` for v0.1.1 success
- [x] List release assets: `gh release view v0.1.2 --repo GQAdonis/disk-space-guardian --json assets` (3/4 uploaded; macOS 13 queued)
- [x] Run `install_dsg()` directly — Path A source build succeeded: dsg → ~/.local/bin/dsg
- [x] Verify `dsg --version` returns the expected version from PATH
