---
id: installation
title: Installation
sidebar_label: Installation
---

# Installing Sovereign Sync

## Automatic installation

Install the compiled tools, then render and start the managed services:

```bash
bash scripts/check-prerequisites.sh --install --build-tools
bash scripts/install-mcp-services.sh
```

What the installer does for sovereign-sync:

1. ensures `$HOME/.config/sovereign-sync/config.toml` has a non-empty
   `node.operator_id`;
2. runs `sovereign-sync --mode init` to create a permission-protected Ed25519
   device key;
3. sets the config and key file to mode `0600`;
4. renders `ai.prometheus.sovereign-sync` for launchd or systemd;
5. starts the daemon on loopback port `7892`;
6. reuses an already-running service instead of double-starting it.

`operator_id`, the device key, and the KBD control bearer token are three
different values. See [Tokens and authentication](/docs/kbd/tokens-and-authentication).

Each independent installation generates its own `operator_id`. That is the
safe single-machine default, but two machines will not share a gossip topic
until you deliberately choose one operator namespace and place that same value
in both configs. Do not copy the device key or bearer token with it. Follow
[Pair two machines](./pair-two-machines) after both installations are healthy.

## Verify installation

```bash
# Binary present
sovereign-sync --help

# Daemon running (macOS launchd)
launchctl print "gui/$(id -u)/ai.prometheus.sovereign-sync"

# REST health check
curl -s http://127.0.0.1:7892/health | jq .

# Full CLI diagnosis
prometheus doctor --json | jq .
```

Expected health response:

```json
{
  "status": "ok",
  "service": "sovereign-sync",
  "version": "0.1.0"
}
```

## Manual startup

If you prefer not to use launchd:

```bash
# Optional explicit token path
export PROMETHEUS_CONTROL_TOKEN_FILE="$HOME/.config/sovereign-sync/kbd-control-token"

# Foreground server (verbose)
RUST_LOG=sovereign_sync=debug sovereign-sync --mode daemon
```

`--mode daemon` starts P2P in the background and keeps local REST/KBD control
available when zero peers exist or gossip startup fails. `--mode server`
starts the HTTP server without the daemon-mode P2P setup.

## Configuration file

Default path:

```text
$HOME/.config/sovereign-sync/config.toml
```

Example:

```toml
[node]
skills_dir = "/path/to/installed/skills"
operator_id = "recommended-64-random-hex-characters"

[peers]
bootstrap = [
  "iroh-endpoint-id-from-an-already-running-trusted-peer"
]

[server]
port = 7892

[kbd]
node_id = 1

[[kbd.voters]]
id = 1
endpoint = "local"
witness = false
```

The installer creates `operator_id`; do not use it as an HTTP bearer token.
`peers.bootstrap` accepts iroh endpoint IDs, not IP addresses, HTTP URLs,
device-key IDs, project IDs, or relay tickets. In `0.1.0` the endpoint ID is
logged at daemon startup and changes after that daemon restarts.

The config parser accepts any non-empty operator ID, but a value generated with
`openssl rand -hex 32` is the recommended format. Preserve each machine’s own
`skills_dir` and copy only the shared operator value during pairing.

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
- Port 7892 available (daemon/server modes)
- `config.toml` with non-empty `node.operator_id` in daemon mode
- mode-`0600` Ed25519 device key when `PROMETHEUS_HEADLESS_VOTER=1`
- Outbound access to the N0 discovery/relay infrastructure for the current P2P
  preset, even when the two peers are on the same LAN

The current binary does not expose custom relay or discovery configuration.
See [Network configuration](./p2p-network) before using a corporate,
restricted, offline, or production network.
