---
id: installation
title: Installation
sidebar_label: Installation
---

# Installing Sovereign Sync

## Automatic installation

Sovereign Sync is optional. Install and start it only when cross-machine
sharing is required:

```bash
bash scripts/check-prerequisites.sh --install --build-tools
bash scripts/install-binaries.sh --sharing
bash scripts/install-mcp-services.sh --sharing
```

What the installer does for sovereign-sync:

1. runs `sovereign-sync --mode init` to create permission-protected KBD and
   durable P2P identities;
2. atomically persists secrets with mode `0600`;
3. renders `ai.prometheus.sovereign-sync` for launchd or systemd;
4. starts the daemon on a same-user Unix-domain socket;
5. reuses an already-running service instead of double-starting it.

Without `--sharing`, the managed-service installer stops and disables any
existing sovereign-sync service. Ordinary KBD commands continue against the
signed local runtime without a daemon.

The P2P identity/group secret, project/replica identities, and each KBD device
key are different values. Pair machines with export/import tickets; never copy
an identity file. See [Pair two machines](./pair-two-machines).

## Verify installation

```bash
# Binary present
sovereign-sync --help

# Daemon running (macOS launchd)
launchctl print "gui/$(id -u)/ai.prometheus.sovereign-sync"

# Local health check through the default Unix socket
sovereign-sync --mode status --format json | jq .

# Full CLI diagnosis
prometheus doctor --json | jq .
```

Expected health response:

```json
{
  "status": "ok",
  "service": "sovereign-sync",
  "version": "1.7.0"
}
```

## Manual startup

If you prefer not to use launchd:

```bash
# Foreground server (verbose)
RUST_LOG=sovereign_sync=debug sovereign-sync --mode daemon
```

`--mode daemon` starts P2P in the background and keeps local REST/KBD control
available when zero peers exist or gossip startup fails. `--mode server`
starts the HTTP server without the daemon-mode P2P setup.

Both modes use the platform data-directory Unix socket by default. To expose
loopback TCP, pass `--tcp`; the process then requires a bearer token from the
configured mode-`0600` token file.

## Configuration file

Default path:

```text
$HOME/.config/sovereign-sync/config.toml
```

Example:

```toml
[node]
skills_dir = "/path/to/installed/skills"
p2p_identity_file = "/private/path/p2p-identity.json"

[peers]
bootstrap = [
  "iroh-endpoint-id-from-an-already-running-trusted-peer"
]

[server]
port = 7892 # used only with explicit --tcp
```

`peers.bootstrap` accepts iroh endpoint IDs, not IP addresses, HTTP URLs,
device-key IDs, project IDs, or relay tickets. In `1.7.0` the endpoint ID is
persisted and stable across daemon restarts. Group membership comes from the
random secret and endpoint allow-list stored in the P2P identity, not config.

## Register projects with a managed daemon

The daemon routes every project in the machine registry; it is not focused by
its working directory or an environment variable. A checkout is eligible only
when it already declares its UUID in `.prometheus/project.json`:

```bash
prometheus kbd register /path/to/project
prometheus kbd projects --json
prometheus kbd replicas --project-id <project-uuid> --json
```

Registration never creates or infers project identity from a path, Git origin,
or commit. Matching origin/HEAD evidence is reported only as a possible
duplicate. Restarting is not required for REST registration; the router reloads
after a successful `POST /api/v1/kbd/projects/register`.

After changing any other plist setting, fully reload it. `kickstart` alone does
not reload the definition:

```bash
LABEL="ai.prometheus.sovereign-sync"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DOMAIN="gui/$(id -u)"

launchctl bootout "$DOMAIN/$LABEL"
launchctl bootstrap "$DOMAIN" "$PLIST"
launchctl enable "$DOMAIN/$LABEL"
launchctl kickstart -k "$DOMAIN/$LABEL"
```

The managed installer performs the same bootout/bootstrap sequence when run
with `--restart`:

```bash
bash scripts/install-mcp-services.sh --restart
```

Be aware that a managed reinstall regenerates the plist from the repository
template. Persist deployment-specific overrides in your service-management
layer rather than relying on an untracked manual edit.

For systemd unit changes, reload and restart the user service:

```bash
systemctl --user daemon-reload
systemctl --user restart ai.prometheus.sovereign-sync
```

## MCP configuration

An MCP client launches stdio mode and reads the same platform registry:

```json
{
  "sovereign-sync": {
    "command": "/path/to/sovereign-sync",
    "args": ["--mode", "mcp"],
    "env": {
      "RUST_LOG": "sovereign_sync=warn"
    }
  }
}
```

When exactly one project is registered, KBD tool calls may omit `project_id`.
With multiple projects, pass the UUID explicitly; omission is rejected rather
than guessed.

For Kimi Code, add an equivalent entry to `~/.kimi-code/config.toml`.

## UAR passthrough mode

When `sovereign-sync` is embedded inside a Universal Agent Runtime (UAR), set:

```bash
export UAR_SKILL_SERVICE_URL=http://uar-host:port
```

The daemon detects this env var at startup and logs:

```
UAR_SKILL_SERVICE_URL detected — enabling passthrough mode (sync tools only)
```

In passthrough mode, sync tools forward to the UAR service for cross-node operations instead
of operating on the local P2P network directly.

## Prerequisites

- Rust stable (for building from source)
- macOS or Linux
- a writable private location for the Unix socket and P2P identity
- Port 7892 available only when using explicit `--tcp`
- a protected Ed25519 device key for signed KBD mutations
- Outbound access to the N0 discovery/relay infrastructure for the current P2P
  preset, even when the two peers are on the same LAN

The current binary does not expose custom relay or discovery configuration.
See [Network configuration](./p2p-network) before using a corporate,
restricted, offline, or production network.
