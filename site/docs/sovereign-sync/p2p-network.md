---
id: p2p-network
title: P2P Network
sidebar_label: Network
---

# Optional peer supervision

The selected configuration can set `[node] p2p_enabled = false`. Local authority
starts before optional transport. Status reports `disabled`, `initializing`,
`bootstrapping`, `ready`, `degraded`, `failed` or `stopping`, plus retry attempt,
last error, next retry and peer IDs. Ready does not prove peer application.

Each supervisor retry uses the selected identity, bootstrap endpoint IDs and
discovery settings. mDNS defaults on and locates changing addresses for known
IDs; it does not enroll unknown peers. DHT defaults off. Enabling
`[discovery] dht = true` publishes a signed endpoint/relay/address record to the
public DHT. iroh relay behavior is separate; disabling DHT does not promise no
relay traffic. Endpoint authentication uses Ed25519 over iroh QUIC/TLS, not an
additional Noise handshake. Network pairing never grants project mutation.

Use [pairing](/docs/sovereign-sync/pair-two-machines) and inspect the
[receipt evidence](/docs/sovereign-sync/signed-pushes-and-receipts) separately.

On a single Mac, the transport remembers the UDP ports selected for a local identity and tries to reuse them after restart. Check the receiver’s signed history to confirm a backlog arrived. [Signed push recovery](/docs/sovereign-sync/signed-pushes-and-receipts)

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
