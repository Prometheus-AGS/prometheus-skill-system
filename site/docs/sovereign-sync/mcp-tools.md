---
id: mcp-tools
title: MCP Tools
sidebar_label: MCP Tools
---

# MCP Tools

When running as `--mode mcp` (default when configured via `mcp-servers.json`), sovereign-sync
exposes 4 tools to any MCP-compatible harness.

## Configuration

Add to `~/.claude/mcp-servers.json`:

```json
{
  "sovereign-sync": {
    "command": "/Users/you/.local/bin/sovereign-sync",
    "args": ["--mode", "mcp"],
    "env": { "RUST_LOG": "sovereign_sync=warn" }
  }
}
```

To avoid tool name collisions in UAR or multi-server environments:

```json
{
  "sovereign-sync": {
    "command": "/Users/you/.local/bin/sovereign-sync",
    "args": ["--mode", "mcp", "--prefix-tools"]
  }
}
```

When `--prefix-tools` is set, all tool names are prefixed with `sovereign:` (e.g.,
`sovereign:search-skills`).

## Tools

### `search-skills`

Search the local skill index by keyword.

**Input schema:**

```json
{
  "query": "feynman",
  "limit": 5
}
```

**Output:**

```json
{
  "results": [
    { "name": "feynman-loop", "description": "Core Feynman PMPO loop" }
  ],
  "count": 1
}
```

### `sync-status`

Returns current node state and connected peer count.

**Input schema:** `{}` (no parameters)

**Output:**

```json
{
  "node_state": "Connected",
  "peer_count": 2,
  "domains_active": ["kbd-orchestrator", "learner-model"]
}
```

### `sync-push`

Push a sync domain to all connected peers.

**Input schema:**

```json
{
  "domain": "learner-model"
}
```

Valid domains: `skill-index`, `learner-model`, `kbd-orchestrator`, `open-spec`

**Output:**

```json
{
  "status": "queued",
  "domain": "learner-model"
}
```

**Privacy:** `surreal-memory` is rejected with a `PrivacyViolation` error.

### `sync-peers`

List connected peer node IDs.

**Input schema:** `{}` (no parameters)

**Output:**

```json
{
  "peers": [
    { "node_id": "a1b2c3...", "addr": "192.168.1.42:7892" }
  ]
}
```
