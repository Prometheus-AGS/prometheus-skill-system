---
id: change-int-005-install-dsg
title: install_dsg() in install-binaries.sh + CLAUDE.md + detect-toolchain.sh
phase: cowork-integration
priority: P1
effort: M
wave: 5
agent: devops-engineer
status: done
gap_id: G-05-dsg
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - scripts/install-binaries.sh (add install_dsg function)
  - CLAUDE.md (add dsg commands to Essential Commands)
  - shared/scripts/detect-toolchain.sh (add dsg binary check)
---

# change-int-005 — install_dsg() in install-binaries.sh + CLAUDE.md + detect-toolchain.sh

## Context

The dsg CLI submodule is wired in (change-int-001) and has plugin artifacts
(change-int-002 through change-int-004). This change adds the install plumbing
so that `bash scripts/install-binaries.sh` builds/downloads the `dsg` binary,
and surfaces it in CLAUDE.md and the toolchain health check.

## Strategy

1. Add `install_dsg()` to scripts/install-binaries.sh following the same
   two-path pattern as `install_cowork()`: source build preferred, GitHub
   Releases download as fallback.
2. Add dsg scan/clean/status commands to CLAUDE.md Essential Commands.
3. Add `dsg` binary check to shared/scripts/detect-toolchain.sh (Prometheus
   Binaries section).

## Scope

1. Add install_dsg() to scripts/install-binaries.sh and call it at the end
2. Update CLAUDE.md: add dsg commands to the cowork-management section
3. Update shared/scripts/detect-toolchain.sh: add dsg check
4. Update KBD orchestrator
5. Commit
