# change-extval-001-quick-start-guide

**Phase:** phase-external-validation  
**Type:** documentation  
**Status:** proposed  
**Priority:** P0 (enables G1 and G2)

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

- [ ] Write prerequisites section (Node ≥ 18, Git, Rust — three lines)
- [ ] Write clone step (with `--recurse-submodules` flag)
- [ ] Write install step (`bash scripts/install-skills-flat.sh`)
- [ ] Write smoke test step (`bash shared/scripts/detect-toolchain.sh`)
- [ ] Write first invocation step (`/learn-goal "explain recursion to a 10-year-old"`)
- [ ] Add link from README.md "Quick Start" section to QUICK_START.md
