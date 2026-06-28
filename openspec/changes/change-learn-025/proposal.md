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
