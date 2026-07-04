# Tasks — change-dib-001-release-archive-format

- [ ] Update `.github/workflows/release.yml` in dsg repo: replace bare-binary upload with versioned tar.gz packaging step
- [ ] Commit the release.yml fix in the dsg repo
- [ ] Push commit to `origin/main`
- [ ] Tag `v0.1.1` and push: `git tag -a v0.1.1 -m "dsg v0.1.1 — fix release archive naming for install-binaries.sh Path B"` + `git push origin v0.1.1`
- [ ] Advance `tools/disk-space-guardian` submodule pointer to `v0.1.1` and commit in skill-pack worktree
- [ ] Confirm `gh run list` shows a new run triggered for `v0.1.1`
