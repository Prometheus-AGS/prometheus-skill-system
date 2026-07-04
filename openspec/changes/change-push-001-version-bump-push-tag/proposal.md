---
id: change-push-001-version-bump-push-tag
title: Bump cowork version 0.1.5 → 0.2.0, push 10 commits, tag v0.2.0
phase: phase-cowork-push-and-release
priority: P0
effort: S
wave: 1
agent: general-purpose
status: pending
gap_id: G-01 G-02
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (local worktree)
  - cli/Cargo.toml (version field)
  - cli/Cargo.lock (lock file update)
  - git push origin main
  - git tag v0.2.0 + git push origin v0.2.0
---

# change-push-001 — Version bump + push + tag v0.2.0

## Context

10 Rust commits implementing Zed/Kimi/MiniMax platform support,
pack/toolchain/disk subcommands, and Claude Code/Codex/OpenCode plugin
management are complete in the local cowork-skills worktree but not pushed
to the remote. The Cargo.toml version is still 0.1.5.

## Strategy

1. Bump `cli/Cargo.toml` version to `0.2.0`
2. Run `cargo update --workspace` (or edit Cargo.lock directly for the cowork crate)
3. Commit the version bump
4. `git push origin main` — pushes all 11 commits (10 feature + 1 version bump)
5. `git tag -a v0.2.0 -m "release: v0.2.0 — new platform support + subcommands"`
6. `git push origin v0.2.0` — triggers release.yml binary builds
7. Verify CI at github.com/GQAdonis/cowork-skills/actions

## Scope

1. Bump Cargo.toml version 0.1.5 → 0.2.0
2. Update Cargo.lock
3. Commit version bump
4. Push origin main
5. Tag v0.2.0 and push tag
