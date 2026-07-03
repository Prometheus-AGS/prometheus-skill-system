---
name: sync-status
description: Show the current P2P sync status of the sovereign-sync node — connection state, peer count, and domain health.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [sovereign-sync, p2p, sync, status, learn, iroh]
---

# /sync-status

Show the live status of the local sovereign-sync node: connection state, number
of connected peers, and a summary of which sync domains are up to date.

## When to use

Trigger on:
- "sync status", "check sync", "is sync running", "how many peers"
- Before a `/sync-push` to confirm the node is `Connected`
- After install to verify the node started correctly

## Prerequisites

A `sovereign-sync` daemon or server must be running on `127.0.0.1:7892`.
Start one with:

```bash
sovereign-sync --mode daemon
```

or check if the launchd service is loaded:

```bash
launchctl list | grep sovereign-sync
```

## Instructions

1. Query the REST health endpoint to confirm the node is up:

   ```bash
   curl -s http://127.0.0.1:7892/health | jq .
   ```

2. Fetch detailed sync status:

   ```bash
   curl -s http://127.0.0.1:7892/api/v1/sync/status | jq .
   ```

3. Interpret the `node_state` field:

   | State | Meaning |
   |-------|---------|
   | `Disconnected` | Node started but not yet joined the P2P network |
   | `Bootstrapping` | Connecting to known bootstrap peers |
   | `Connected` | At least one peer reachable; gossip active |
   | `Syncing` | Actively exchanging CRDT deltas with peers |
   | `Idle` | Connected and in sync; no active exchange |

4. Report to the user:
   - Node state
   - Peer count (`peers` array length)
   - Domains listed in the status response

## Output format

```
sovereign-sync status
─────────────────────
State     : Connected
Peers     : 2
Domains   : skill-index, learner-model
```

## Troubleshooting

**Node not reachable** — start the daemon:
```bash
sovereign-sync --mode daemon &
```

**State stays `Bootstrapping`** — no peers are reachable. Add a peer address:
```bash
/sync-peers add <node-id>@<address>
```

**State is `Disconnected`** — check that port 7892 is not blocked by a firewall.

## Privacy guarantee

The status endpoint returns only network metadata (state, peer IDs, domain
names). No KB content is exposed. `PrivacyClass::LocalOnly` domains are visible
by name in the domain list but their payloads never leave the device.
