---
id: change-hardening-005-daemon-health-detect-toolchain
title: Sovereign-sync daemon health detection
phase: phase-sovereign-sync-hardening
priority: MEDIUM
effort: S
agent: codex
status: planned
scope:
  - substrate/sovereign-sync
  - scripts
  - skills/process
---

# change-hardening-005 — Sovereign-sync daemon health detection

## Context

The sovereign-sync daemon/server is expected to use localhost port `7892`, but downstream installers and operators need a deterministic way to tell whether the service is healthy, missing, or blocked by a different process.

## Scope

- Add a minimal health endpoint or status command if one does not already exist.
- Wire health detection into the relevant detect-toolchain or installer diagnostic path.
- Detect port `7892` conflicts without killing user processes.
- Add a lightweight test or fixture for healthy, missing, and occupied-port states.

## Non-Goals

- No launchd service overhaul.
- No production monitoring stack.
- No authentication integration.

## Validation

- Local health command or endpoint returns deterministic output.
- Detection distinguishes healthy sovereign-sync from unrelated process on port `7892`.
