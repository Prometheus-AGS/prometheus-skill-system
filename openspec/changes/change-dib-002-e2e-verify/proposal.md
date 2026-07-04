# change-dib-002-e2e-verify

**Status**: pending

## Summary

Confirm the `v0.1.1` release CI matrix completed green and all 4 binary
artifacts are present. Then run `bash scripts/install-binaries.sh` end-to-end
to verify Path A works on this machine.

## Acceptance Criteria

- `gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml` shows
  `v0.1.1` run completed with status `success`
- All 4 artifacts present: `dsg-<ver>-aarch64-apple-darwin.tar.gz`,
  `dsg-<ver>-x86_64-apple-darwin.tar.gz`,
  `dsg-<ver>-x86_64-unknown-linux-musl.tar.gz`,
  `dsg-<ver>-x86_64-pc-windows-msvc.exe.tar.gz` (or `.zip`)
- `bash scripts/install-binaries.sh` completes without error and
  `dsg --version` returns the expected version
