---
type: SessionRecord
id: karpathy-session-81579975019fc232
title: Karpathy session 81579975019f
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T10:49:07.364064+00:00
created_at: 2026-11-16T10:49:07.364064+00:00
updated_at: 2026-11-16T10:49:07.364064+00:00
revision: 0
---

## Delta

The cowork download path now honors `--dry-run`, and the installer fixes are committed and pushed. The exec pin is not committed, because the integration test failed.

**Installer fixes: committed as `a16dac18` and pushed to `origin/main`**

- **cowork:** with the `tools/cowork-skills` submodule absent, a dry run now prints that it would download the release and returns. I checked this with `curl` replaced by a tripwire: the new script made no download call, and the previous version made two.
- **Full real pass:** exited 0 with all binaries installed, including `prometheus-exec`. This also exercised the reworked `prometheus-research` section.
- **Full dry run:** exited 0, compiled nothing, and left the research service, the installed binaries and the repo tree unchanged.

**Exec pin: held back**

The integration test's real-build case failed. A fresh compile from a second checkout path on this machine produced `eb6128fd…`, not the `fe2f2c5e…` I put in the Intel entry. So `fe2f2c5e…` is only what the cached Oct 5 artifact hashes to; it is not what this machine builds today. The full installer pass accepted it only because the main checkout's build was a cache hit on that old artifact.

I don't yet know which of two causes it is:

- **The build depends on the checkout path.** Then no single Intel hash can work across checkouts or worktrees.
- **Something in the build environment changed since Oct 5.** Then every fresh Intel build now gives `eb6128fd…`, and that is the right pin.

To tell them apart I've queued a forced fresh build in the main checkout. If it gives `eb6128fd…` the cause is the environment; if it gives `fe2f2c5e…` the cause is the path. It is waiting because another session on this machine is running a cargo build, and only one may run at a time. I'll report when it finishes.

The per-target pin work is all still uncommitted in the working tree.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T10:49:04.801041Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/rust-toolchain.toml
- dist/plugins/claude/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- dist/plugins/codex/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/build-prometheus-exec.sh
- scripts/install-prometheus-exec.sh
- scripts/tests/install-prometheus-exec.test.sh
- skills/process/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/prometheus-exec-pin.sh
