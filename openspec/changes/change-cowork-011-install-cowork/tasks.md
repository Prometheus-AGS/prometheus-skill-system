# Tasks: change-cowork-011-install-cowork

- [x] Add `tools/cowork-skills` submodule entry to `.gitmodules`
- [x] Add `install_cowork()` function to `scripts/install-binaries.sh` (source-build + GitHub Release fallback)
- [x] Call `install_cowork` at end of install-binaries.sh
- [x] Smoke-test: bash -n scripts/install-binaries.sh passes
- [x] Commit to skill-pack worktree (afb41ea)
- [x] Update KBD orchestrator (progress.json, waypoint, position-reminder)
