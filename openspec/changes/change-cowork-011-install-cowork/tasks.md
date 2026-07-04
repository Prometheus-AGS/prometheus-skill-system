# Tasks: change-cowork-011-install-cowork

- [ ] Add `tools/cowork-skills` submodule entry to `.gitmodules`
- [ ] Add `install_cowork()` function to `scripts/install-binaries.sh` (source-build + GitHub Release fallback)
- [ ] Call `install_cowork` at end of install-binaries.sh
- [ ] Smoke-test: bash -n scripts/install-binaries.sh passes
- [ ] Commit to skill-pack worktree
- [ ] Update KBD orchestrator (progress.json, waypoint, position-reminder)
