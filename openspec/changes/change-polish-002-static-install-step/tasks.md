# Tasks — change-polish-002-static-install-step

- [ ] Run git rm docs/deep-research/static to remove the symlink from git index
- [ ] Copy substrate/prometheus-research/src/static/ to docs/deep-research/static/ as real files
- [ ] git add docs/deep-research/static/ and verify mode 100644 entries
- [ ] Add cp -r static assets step to scripts/install-binaries.sh after binary install block
- [ ] Verify install-binaries.sh change runs without error
- [ ] Commit both the static files addition and the install script update
