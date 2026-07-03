---
name: sync-peers
description: List, add, or remove peers in the sovereign-sync P2P network. Manage which nodes participate in CRDT state synchronization.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [sovereign-sync, p2p, peers, iroh, learn, network]
---

# /sync-peers

Manage peers in the sovereign-sync P2P gossip network. Peers exchange CRDT
deltas for enabled sync domains (skill indexes, learner models, etc.).

## When to use

Trigger on:
- "add peer", "show peers", "list peers", "remove peer", "sync with <device>"
- When setting up sync across multiple devices for the first time
- When a peer drops off and you want to re-add it

## Prerequisites

A `sovereign-sync` daemon or server must be running:

```bash
sovereign-sync --mode daemon
```

## Instructions

### List current peers

```bash
curl -s http://127.0.0.1:7892/api/v1/sync/peers | jq .
```

Output:
```json
{
  "peers": [
    { "node_id": "a1b2c3...", "addr": "192.168.1.42:7892" },
    { "node_id": "d4e5f6...", "addr": null }
  ]
}
```

### Find your own node ID

```bash
curl -s http://127.0.0.1:7892/health | jq .node_id
```

Share this with other devices so they can add you as a peer.

### Add a peer

Use the `/sync-push` skill after adding a peer — it will attempt to connect
via iroh gossip automatically once the peer address is known to the QUIC
transport layer.

For direct peer address addition (when on the same LAN):

```bash
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "skill-index"}'
```

The daemon automatically discovers peers via the iroh relay and DNS discovery
(`presets::N0`). On a LAN, peers bootstrap via known addresses embedded in
sync-manifests.

### Bootstrap via operator key

Nodes that share the same `operator_id` (a 32-byte key) are automatically
placed on the same gossip topic:

```
Topic = blake3(operator_id || "sovereign-sync-v1")
```

Set the operator key at daemon startup:

```bash
sovereign-sync --mode daemon --operator-id <hex-32-bytes>
```

All devices with the same `--operator-id` join the same P2P group automatically.

## Privacy guarantee

Peer metadata (node IDs, addresses) is network infrastructure — not KB content.
No learner data, skill content, or KB payloads are exposed via the peers endpoint.
