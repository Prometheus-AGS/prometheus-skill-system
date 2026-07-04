# Goals — phase-cowork-push-and-release

## Context

The cowork-integration phase delivered 10 Rust commits to the local cowork-skills
worktree at `/Users/gqadonis/Projects/prometheus/cowork-skills` but those commits
were never pushed to `git@github.com:GQAdonis/cowork-skills.git`. The
`tools/cowork-skills` submodule in prometheus-skill-pack still points to the
upstream v0.1.5 tag (53e6b31), meaning `install_cowork()` in install-binaries.sh
builds the unextended binary without Zed/Kimi/MiniMax/pack/toolchain/disk support.

This phase closes that gap by pushing the commits, tagging a release, and advancing
the submodule pointer.

## Goals

- G-01: Push the 10 cowork-integration commits from the local worktree to
  `git@github.com:GQAdonis/cowork-skills.git` on `main`.
- G-02: Tag a semver release (v0.2.0) on the cowork-skills remote and confirm
  the CI workflow passes.
- G-03: Advance the `tools/cowork-skills` submodule pointer in prometheus-skill-pack
  to the new HEAD, commit, and verify `git submodule status` shows clean.
- G-04: Confirm `cowork pack status` and `cowork toolchain status` work end-to-end
  from the installed binary on this machine.
