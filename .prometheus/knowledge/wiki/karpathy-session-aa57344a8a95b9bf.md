---
type: SessionRecord
id: karpathy-session-aa57344a8a95b9bf
title: Karpathy session aa57344a8a95
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T08:40:12.037952+00:00
created_at: 2026-08-06T08:40:12.037952+00:00
updated_at: 2026-08-06T08:40:12.037952+00:00
revision: 0
---

## Delta

Done. `prometheus-exec` now auto-starts, and I found two latent bugs while getting there.

## The core gap

`install-prometheus-exec-service.sh` had **zero callers** — only four docs mentioned it. The binary installer was wired into `install-binaries.sh:39`, but nothing ever installed the *service*, which is why `ai.prometheus.exec.plist` was absent from `~/Library/LaunchAgents` and the binary wasn't on this machine at all.

**Wiring added** (delegating to its own installer, per your choice, so identity/version/hash/signature checks are preserved):

| Surface | Change |
|---|---|
| `install-mcp-services.sh` | `install_exec_service()` after the daemon loop; added to `macos_unload`; honors `--dry-run` and `--exclude exec`; a failure warns instead of aborting the other 7 daemons |
| `check-mcp-health.sh` | New `unix:` probe mode (`test -S`) — a real liveness check, not a `stdio` placeholder |
| `prometheus-services.sh` | Added to `DOCTOR_LABELS`, the binary list, and `status` |

I deliberately did **not** add it to `DAEMON_LABELS`, `TEMPLATES`, or `--render-only`: those paths use a shared `render_template` whose placeholder map lacks the `__PROMETHEUS_EXEC_*__` tokens, and port-probing can't check a socket daemon.

## Two bugs found en route

**1. The hash pin was unreproducible.** `install-prometheus-exec.sh` gates on a SHA256, but the repo pinned no toolchain — the committed hash (`ae70b4d2…`) came from a different `rustc` than this machine's default nightly, so the gate failed on a correctly-built binary. Fixed by pinning `crates/prometheus-exec/rust-toolchain.toml` to **stable** (matching the `tools/forge-rs` precedent; the crate declares `rust-version = "1.94"` and uses no nightly-only features) and repinning to `c5ee008f…`, verified reproducible across a forced relink.

**2. `--manifest-path` silently bypassed the pin.** `install-prometheus-exec.sh:72` ran `cargo build --manifest-path …` from the repo root. **`rust-toolchain.toml` resolves from the current directory and is not honored via `--manifest-path`** — so even after adding the file, the build still used nightly. Changed to build from inside the crate directory. `site/docs/execution/…` documented the same broken command; corrected.

## Verification

- `install-prometheus-exec.sh` — installed and hash-verified; its test passes (atomic install, rollback, false-green contracts)
- `install-policy.test.sh`, `test-learning-service-install.sh` — pass
- `doctor` — **PASS on all 11 checks**, including before *and* after restart
- `RunAtLoad=true`, zero unsubstituted placeholders; SIGKILL auto-recovered (45530 → 55834)
- **Real execution**: run `ad5e5f54…` Succeeded with a signed receipt
- Re-running the full installer left the other 7 daemons' PIDs untouched

Two things worth flagging: re-running the installer **does** restart the exec daemon (PID changed 55834 → 85246) because the delegated installer unconditionally `bootout`s and `kickstart -k`s — unlike the other daemons, which reuse a healthy instance. It came back PASS both times, but if you want reuse-if-healthy semantics there, that's a small change to `install-prometheus-exec-service.sh`. Separately, the pre-existing `sovereign-sync UNREACHABLE` and `forge-rs 422` are unrelated to this work and were failing before I started.

Nothing is committed — the changes are staged in the working tree for your review.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T08:40:09.536904Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- config/prometheus-exec-binary.json
- docs/guide/16-cli-and-scripts.md
- docs/guide/19-installation.md
- scripts/check-mcp-health.sh
- scripts/install-mcp-services.sh
- scripts/install-prometheus-exec.sh
- scripts/prometheus-services.sh
- site/docs/execution/installation-doctor-and-recovery.md
- site/docs/operations/installation-and-upgrades.md
- .prometheus/knowledge/wiki/karpathy-session-044cc4d6c70eb7d9.md
- .prometheus/knowledge/wiki/karpathy-session-51c79b1ee9ca4893.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- crates/prometheus-exec/rust-toolchain.toml
