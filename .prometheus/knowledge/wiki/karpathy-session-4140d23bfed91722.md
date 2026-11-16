---
type: SessionRecord
id: karpathy-session-4140d23bfed91722
title: Karpathy session 4140d23bfed9
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T10:44:48.989275+00:00
created_at: 2026-11-16T10:44:48.989275+00:00
updated_at: 2026-11-16T10:44:48.989275+00:00
revision: 0
---

## Delta

Both sections of [install-binaries.sh](scripts/install-binaries.sh) now honor `--dry-run`, and a dry run confirmed it changes nothing.

- **`dsg`:** a dry run prints what it would build (or download, when the submodule is absent) and returns without doing either.
- **`prometheus-research`:** a dry run prints the build, the binary install, the static-asset copy into `docs/deep-research/static/` and the launchd re-registration, and performs none of them.
- **Dry-run check:** it exited 0 and compiled nothing. The research service kept the same process id, and its plist, both installed binaries and the repo tree were unchanged.

One behaviour change in the real (non-dry-run) path: a failed `prometheus-research` build used to abort the script on the spot. It is now reported as a failed component, and the run still ends non-zero. The real path has not been exercised yet; the full installer pass still queued in the background will run it.

The `cowork` section has the same gap on its download path: when the `tools/cowork-skills` submodule is absent, a dry run still downloads the release. I left it alone since you asked about the other two.

Nothing is committed or pushed yet. The exec integration test is still in its real-build stage, and the full installer pass follows it; I'll commit and push once both pass.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T10:44:46.974400Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/rust-toolchain.toml
- dist/plugins/claude/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- dist/plugins/codex/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/build-prometheus-exec.sh
- scripts/install-binaries.sh
- scripts/install-prometheus-exec.sh
- scripts/install-skills-flat.sh
- scripts/tests/install-prometheus-exec.test.sh
- skills/process/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/cargo-release-dir.sh
- scripts/prometheus-exec-pin.sh
