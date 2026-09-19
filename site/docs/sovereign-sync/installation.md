---
id: installation
title: Installation
sidebar_label: Installation
---

# Install the Companion control plane

Sovereign Sync is owned by
[`prometheus-companion`](https://github.com/Prometheus-AGS/prometheus-companion).
The skill pack remains complete without it and does not build, install, start,
stop, or repair the Companion service.

From a `prometheus-companion` checkout, use the Companion-owned installers:

```bash
bash scripts/install-companion-service.sh
bash scripts/install-skill-package.sh
```

The first command installs the Companion process and its control endpoint. The
second publishes `/sync-status`, `/sync-peers`, and `/sync-push` through the
Companion's skill package. Re-run the same commands after upgrading the
Companion.

The pack discovers the optional endpoint through integration-contract seam 1.
Absence is valid and does not produce a warning:

```bash
prometheus contract show --json
prometheus doctor --json --check control
```

Contract v1 retains the compatibility identifiers `SOVEREIGN_SYNC_SOCKET`,
`sovereign-sync.sock`, and the existing device-key path. They identify the open
transport contract; they do not make the daemon a skill-pack component.

For process and pairing details, continue with
[Pair two machines](./pair-two-machines.md) and
[P2P network](./p2p-network.md) from the Companion checkout.
