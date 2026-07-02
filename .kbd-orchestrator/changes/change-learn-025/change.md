---
id: change-learn-025
title: install-skills-flat.sh update for learn domain
type: infrastructure
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-020
---

# change-learn-025 — install-skills-flat.sh learn domain support

## Summary

Extend `scripts/install-skills-flat.sh` to include `skills/learn/` in the
platform skill installation sweep, add `substrate/learner-model/` and
`substrate/storage-provider/` Rust crates to the build-and-install step, wire
`substrate/surface-bridge/` into the macOS launchd service install (with a
graceful skip warning on non-macOS), and extend `detect-toolchain.sh` to report
learner-model binary and surface-bridge service status.

## Motivation

Learn-domain skills and their substrate crates are invisible to operators until
the install script installs them. Without this change the domain is fully built
but unreachable.

## Scope

- `scripts/install-skills-flat.sh` — learn domain sweep, substrate builds, launchd install
- `shared/scripts/detect-toolchain.sh` — two new status checks

## Tasks

- [x] Add `skills/learn/` to the platform skill installation sweep in `scripts/install-skills-flat.sh`: include it in the directory glob alongside existing domain directories, ensure all detected platforms receive learn-domain skills
- [x] Add `substrate/learner-model/` Rust crate to the build-and-install step: run `cargo build --release -p learner-model` and copy the resulting binary to `~/.prometheus/bin/learner-model`
- [x] Add `substrate/storage-provider/` Rust crate to the build-and-install step: run `cargo build --release -p storage-provider` and copy binary to `~/.prometheus/bin/storage-provider`
- [x] Add `substrate/surface-bridge/` launchd service install to `install-skills-flat.sh` (macOS only): build `surface-bridge`, copy to `~/.prometheus/bin/surface-bridge`, copy the plist to `~/Library/LaunchAgents/`, run `launchctl load`; on non-macOS, print a warning (`[SKIP] surface-bridge launchd install requires macOS`) and continue with exit 0
- [x] Update `shared/scripts/detect-toolchain.sh` to report: (1) whether `~/.prometheus/bin/learner-model` exists and is executable; (2) whether `com.prometheus.surface-bridge` is loaded in launchctl (macOS) or `N/A` on other platforms; include both in the `--json` output
