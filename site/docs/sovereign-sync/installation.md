---
id: installation
title: Installation
sidebar_label: Installation
---

# Installing Sovereign Sync

## Automatic installation

Run the unified installer — it builds the Rust binary, installs it to `~/.local/bin/`, and on
macOS registers a launchd service that starts automatically at login:

```bash
bash scripts/install-skills-flat.sh
```

What the installer does for sovereign-sync:

1. `cargo build --release --manifest-path substrate/sovereign-sync/Cargo.toml`
2. `cp target/release/sovereign-sync ~/.local/bin/sovereign-sync`
3. On macOS: installs `com.prometheusags.sovereign-sync.plist` into `~/Library/LaunchAgents/`
4. Adds `sovereign-sync` to `~/.claude/mcp-servers.json` (MCP stdio mode)

## Verify installation

```bash
# Binary present
sovereign-sync --help

# Daemon running (macOS launchd)
launchctl list | grep sovereign-sync

# REST health check
curl -s http://127.0.0.1:7892/health | jq .
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
# Background daemon
sovereign-sync --mode daemon &

# Foreground server (verbose)
RUST_LOG=sovereign_sync=debug sovereign-sync --mode server
```

## MCP configuration

For Claude Code, the installer writes to `~/.claude/mcp-servers.json`:

```json
{
  "sovereign-sync": {
    "command": "/Users/you/.local/bin/sovereign-sync",
    "args": ["--mode", "mcp"],
    "env": { "RUST_LOG": "sovereign_sync=warn" }
  }
}
```

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
- No additional cloud infrastructure required
