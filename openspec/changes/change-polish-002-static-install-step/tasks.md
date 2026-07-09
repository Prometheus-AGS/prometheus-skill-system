# Tasks — change-polish-002-static-install-step

- [x] Run git rm docs/deep-research/static to remove the symlink from git index
- [x] Copy substrate/prometheus-research/src/static/ to docs/deep-research/static/ as real files
- [x] git add docs/deep-research/static/ and verify mode 100644 entries
- [x] Add cp -r static assets step to scripts/install-binaries.sh after binary install block
- [x] Verify install-binaries.sh change runs without error
- [x] Commit both the static files addition and the install script update
