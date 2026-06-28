---
id: change-learn-002
title: "Spike: surface-bridge detect-surface-tier probe"
type: design
status: DONE
phase: phase-learn-feynman
depends_on: []
---

# change-learn-002: Spike — surface-bridge detect-surface-tier probe

## Problem

Skills that render interactive content need to know what surface they are running
on so they can degrade gracefully. There is no reliable way to detect the harness
tier at runtime.

## Proposal

Document the detection signals available in each supported harness (Claude Code,
OpenCode, Codex, Kimi, Zed) and ship a `detect-surface-tier.sh` script that
probes those signals and emits a `SURFACE_TIER` environment variable (0, 1, or 2).

## Outcome

A probe script and a convention doc that `change-learn-006` (ui-surface skill)
can invoke to select the correct rendering path.
