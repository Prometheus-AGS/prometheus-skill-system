---
stage: assess
from: assess
to: plan
phase: phase-dsg-install-binaries-fix
timestamp: 2026-07-04T01:10:00Z
artifacts:
  - .kbd-orchestrator/phases/phase-dsg-install-binaries-fix/assessment.md
---

# Handoff: Assess → Plan

Path A in install_dsg() is already correct (workspace-root target path) — G-01
and G-02 are pre-satisfied. The real gap is Path B archive naming: install-binaries.sh
expects versioned tar.gz files (dsg-0.1.0-aarch64-apple-darwin.tar.gz) but
release.yml uploads bare binaries (dsg-aarch64-apple-darwin). Plan needs 2
changes: (1) fix release.yml to tar.gz + versioned naming, (2) verify CI green
+ run install script end-to-end. OQ-01: release run 28714222694 still queued
after 23+ min — may need investigation before or during execute.
