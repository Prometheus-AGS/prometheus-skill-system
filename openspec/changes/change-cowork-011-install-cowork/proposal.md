---
id: change-cowork-011-install-cowork
title: install_cowork() in install-binaries.sh — submodule + binary install
phase: cowork-integration
priority: P1
effort: M
wave: 4
agent: general-purpose
status: in_progress
gap_id: G-05-cowork
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - .gitmodules (add tools/cowork-skills submodule)
  - scripts/install-binaries.sh (add install_cowork function — section 8)
---

# change-cowork-011 — install_cowork() in install-binaries.sh

## Context

The cowork binary needs to be buildable from source (for developers without
a pre-built release) AND downloadable from GitHub Releases (for users who
want a quick install). install-binaries.sh already handles all other tools/
submodule binaries; cowork follows the same pattern.

## Strategy

Two-path install:
1. **Source build** (preferred when tools/cowork-skills submodule present):
   `cd tools/cowork-skills/cli && cargo build --release`
2. **GitHub Release download** (fallback when submodule absent or user passes --download):
   `curl` the latest `cowork-{version}-{os_arch}.tar.gz` from
   `https://github.com/GQAdonis/cowork-skills/releases/latest`

Both paths produce `~/.local/bin/cowork` and `~/.local/bin/co` (the short alias).

## Scope

1. Add `tools/cowork-skills` git submodule entry to `.gitmodules`
2. Add `install_cowork()` function to `scripts/install-binaries.sh` as section 8:
   - detects `tools/cowork-skills/cli/` → source build path
   - falls back to GitHub Release download if submodule absent
   - calls `install_bin` for both `cowork` and `co` binaries
3. Call `install_cowork` at the bottom of the install script

## Verification

- `.gitmodules` contains tools/cowork-skills entry
- `scripts/install-binaries.sh` has install_cowork() function
- Source build path calls `cargo build --release` from `cli/`
- Both `cowork` and `co` binaries installed to BIN_DIR
