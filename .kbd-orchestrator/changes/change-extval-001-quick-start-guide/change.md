---
id: change-extval-001-quick-start-guide
title: QUICK_START.md — zero-to-first-command guide
phase: phase-external-validation
priority: P0 (enables G1 and G2)
agent: claude-code
status: done
scope:
  - docs/QUICK_START.md
  - docs/guide/19-installation.md
---

# change-extval-001-quick-start-guide — QUICK_START.md — zero-to-first-command guide

## Summary

Write `docs/QUICK_START.md` — a single-page guide that gets an external user from
zero to their first `/learn-goal` invocation in under 10 minutes, with no assumed
knowledge of MCP servers or the Prometheus stack.

## Motivation

The existing `docs/guide/19-installation.md` is a 24-page technical reference. An
external user encountering this project for the first time has no obvious path to
a first working command. This change creates that path.

## Deliverables

- `docs/QUICK_START.md` — one page, five steps, copy-paste commands

## Tasks

- Write prerequisites section (Node ≥ 18, Git, Rust — three lines)
- Write clone step (with `--recurse-submodules` flag)
- Write install step (`bash scripts/install-skills-flat.sh`)
- Write smoke test step (`bash shared/scripts/detect-toolchain.sh`)
- Write first invocation step (`/learn-goal "explain recursion to a 10-year-old"`)
- Add link from README.md "Quick Start" section to QUICK_START.md

## Tasks

- [x] 1. Write prerequisites section (Node ≥ 18, Git, Rust — three lines)
- [x] 2. Write clone step (with `--recurse-submodules` flag)
- [x] 3. Write install step (`bash scripts/install-skills-flat.sh`)
- [x] 4. Write smoke test step (`bash shared/scripts/detect-toolchain.sh`)
- [x] 5. Write first invocation step (`/learn-goal "explain recursion to a 10-year-old"`)
- [x] 6. Add link from README.md "Quick Start" section to QUICK_START.md
