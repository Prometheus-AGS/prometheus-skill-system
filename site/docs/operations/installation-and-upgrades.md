---
title: Installation and upgrades
description: Local-first build, signing, service installation, skill discovery, and safe upgrade order.
---

# Installation and upgrades

Prometheus 1.6.1 is installed and certified locally before release branches are pushed. Use an internal SSD for Rust caches:

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

Install the immutable plugin generation after binaries, followed by the learning-worker and hook-log-rotation user services. The service installer accepts repeatable exclusions:

```bash
bash scripts/install-mcp-services.sh --dry-run --exclude sovereign-sync
bash scripts/install-mcp-services.sh --exclude sovereign-sync
```

Always inspect the dry-run plan first. Excluded services are not rendered, installed, restarted, or rewritten.

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

GitHub checks confirm the final environment; they are not the edit/test loop.
