---
id: p2p-network
title: P2P Network
sidebar_label: P2P Network
---

# P2P Network

Sovereign Sync uses [iroh 1.0](https://docs.rs/iroh) + [iroh-gossip 0.101](https://docs.rs/iroh-gossip)
for peer-to-peer connectivity.

## Discovery

iroh uses `presets::N0` which provides:

- **DNS discovery** — peers can be found via DNS without knowing each other's IP addresses
- **QUIC relay** — falls back to relay-assisted connectivity when direct QUIC is blocked by NAT

No STUN server or manual port-forwarding is required. Devices behind NAT can connect.

## Topic-based grouping

Devices that share the same `operator_id` are assigned to the same gossip topic:

```
Topic = BLAKE3(operator_id || "sovereign-sync-v1")
```

Two devices with different operator IDs will NEVER receive each other's sync payloads —
they are on different gossip topics.

Set the operator ID at startup:

```bash
sovereign-sync --mode daemon --operator-id <64-hex-chars>
```

Generate a random operator ID:

```bash
openssl rand -hex 32
```

## Gossip broadcast

When a push is triggered, the gossip sender broadcasts the CRDT delta bytes to all
neighbors on the topic:

```rust
sender.broadcast(payload).await?
```

Peers that receive the gossip message apply the delta to their local Loro doc:

```rust
doc.import(delta)?
```

## Adding bootstrap peers

When two devices are on the same LAN but haven't discovered each other via DNS yet, add
a bootstrap peer directly:

```bash
# Get your node ID
curl -s http://127.0.0.1:7892/health | jq .node_id

# On the other machine, add the peer
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H 'Content-Type: application/json' \
  -d '{"domain": "skill-index"}'
```

The gossip layer will automatically exchange peer information once the first connection
is made.

## Mesh topology

iroh-gossip uses a partial mesh: each node maintains a small set of active neighbors
(typically 5–8 peers) and routes messages through the mesh. You don't need full-mesh
connectivity — as long as the graph is connected, all peers receive all messages.
