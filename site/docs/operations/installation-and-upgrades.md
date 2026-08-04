---
title: Installation and upgrades
description: Local-first build, signing, service installation, skill discovery, and safe upgrade order.
---

# Installation and upgrades

Prometheus 1.7.0 is installed and certified locally before release branches are
pushed. Use an internal SSD for Rust caches:

```bash
export CARGO_HOME=/path/to/cargo-home
export CARGO_TARGET_DIR=/path/on/internal-ssd/prometheus-target
```

Build and test the server first, then knowledge/worker tools, then the root CLI. Install and sign these binaries:

- `surreal-memory-server`
- `pk`
- `pk-cherry`
- `prometheus-learning-worker`
- `prometheus`

The root installer is strict by default: any requested build, copy, service, or
post-install verification failure makes the command fail. Use `--skills-only`
to install only skill payloads, or `--best-effort` for an explicitly
non-certifying development run:

```bash
bash scripts/install-skills-flat.sh
bash scripts/install-skills-flat.sh --skills-only
bash scripts/install-skills-flat.sh --best-effort
```

Install the immutable plugin generation after binaries, followed by the
learning-worker and hook-log-rotation user services. The service installer
accepts repeatable exclusions:

```bash
bash scripts/install-mcp-services.sh --dry-run --exclude sovereign-sync
bash scripts/install-mcp-services.sh --exclude sovereign-sync
```

Always inspect the dry-run plan first. Excluded services are not rendered, installed, restarted, or rewritten.

Success means every requested artifact was byte-verified (and executability was
verified for binaries), the active signed plugin generation passed trust and
receipt verification, and each requested service passed its post-install check.
An empty or skipped check is not success.

## Skills discovery

Repository skills live under `.agents/skills/`; Codex discovery uses `.codex/skills/` and the user catalog. Use the repository sync utility to add missing project skills without replacing installed system skills. Confirm discovery from the target harness after syncing.

## Upgrade order

1. Back up active/previous generation pointers, receipts, snapshots, and queue state.
2. Build and certify the Memory server.
3. Build and certify knowledge tools and worker.
4. Build the root CLI and run local doctors with exclusions.
5. Activate and verify the plugin generation.
6. Reload allowed user services.
7. Certify receipts, queues, snapshots, logs, rollback, and stale-path absence.
8. Push server, knowledge, then root branches.

Push only after local certification. Hosted automation may synchronize
deterministic documentation and package/deploy Pages; it never confirms runtime,
installer, doctor, test, lint, or certification state.
