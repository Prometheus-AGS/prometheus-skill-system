---
id: overview
title: Sovereign Sync Overview
sidebar_label: Overview
slug: /sovereign-sync/overview
---

# Sovereign Sync

Sovereign Sync is the P2P CRDT synchronization layer for the Prometheus Skill Pack. It lets multiple
devices (developer laptops, AI workstations, CI runners) share skill indexes, learner models, and
orchestrator state without any central server.

## What it does

- **Synchronizes skill indexes** across devices so the same set of skills is available everywhere
- **Syncs learner models** (mastery levels, FSRS cards, gap records) so a Feynman loop started on
  one machine continues seamlessly on another
- **Enforces the KB content privacy invariant** — `surreal-memory` and other `LocalOnly` domains
  never leave the device, regardless of sync configuration
- **Runs as an MCP server** so any harness (Claude Code, Kimi, Codex, OpenCode) can query sync
  status and trigger pushes via tool calls
- **Exposes a REST API + AG-UI SSE endpoint** for Tauri desktop apps and web clients

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│  sovereign-sync binary                                      │
│                                                            │
│  ┌──────────┐  ┌─────────────┐  ┌────────────────────┐   │
│  │  MCP     │  │  REST API   │  │  AG-UI SSE         │   │
│  │  stdio   │  │  :7892      │  │  /api/v1/stream    │   │
│  └──────────┘  └─────────────┘  └────────────────────┘   │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  P2P layer: iroh 1.0 + iroh-gossip 0.101             │ │
│  │  CRDT layer: Loro 1.13                               │ │
│  │  Persistence: redb 2.x                               │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

## Modes

| Mode | When to use | Port |
|------|-------------|------|
| `--mode mcp` | Configured in `mcp-servers.json` — harnesses call tools | stdio |
| `--mode daemon` | Background service — also serves REST API | 7892 |
| `--mode server` | Foreground HTTP server for debugging | 7892 |

## Quick start

```bash
# Install (builds Rust binary + registers launchd service on macOS)
bash scripts/install-skills-flat.sh

# Verify the daemon is up
curl -s http://127.0.0.1:7892/health | jq .

# Check sync status via skill
/sync-status

# Push skill index to peers
/sync-push skill-index
```

## Privacy guarantee

`surreal-memory` is assigned `PrivacyClass::LocalOnly` in the `SyncManifest`. This is enforced
structurally in Rust — any attempt to export or apply a delta for a `LocalOnly` domain returns a
`PrivacyViolation` error at the CRDT layer, not just a policy check. KB content (palace RAG, Dify
knowledge bases, local markdown) never enters a sync payload.

---
*Canonical source: [`substrate/sovereign-sync`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/sovereign-sync) — the crate and its doc comments are the source of truth.*
