---
id: p2p-network
title: Network Configuration
sidebar_label: Network Configuration
---

# Network Configuration

Sovereign Sync uses
[iroh](https://docs.iroh.computer/what-is-iroh) with the `N0` preset and
`iroh-gossip` for peer connectivity. The local HTTP API and the P2P transport
are different network surfaces.

## Network surfaces

| Surface | Address/transport | Intended exposure |
|---|---|---|
| REST and SSE | `127.0.0.1:7892` TCP | Local machine only |
| MCP | stdio | Local harness process |
| iroh direct path | dynamically bound UDP for IPv4/IPv6 | Peer-to-peer, negotiated by iroh |
| iroh discovery | N0 DNS/Pkarr services | Outbound internet dependency |
| iroh relay | N0 public relay infrastructure | Encrypted fallback when direct paths fail |

Opening or forwarding TCP port `7892` does not help P2P connectivity. It only
risks exposing a loopback control API if the binding is changed. Keep it on
`127.0.0.1`.

## What the N0 preset does

The preset configures:

- publication and resolution of endpoint reachability through N0 Pkarr/DNS;
- a public relay map;
- direct encrypted QUIC attempts;
- NAT traversal and relay fallback.

An endpoint ID identifies the peer. Discovery resolves that stable identifier
to current direct and relay addresses. In this application the endpoint ID is
not stable across daemon restarts, because the endpoint key is not persisted.

Matching `operator_id` values select the same application gossip topic; they do
not resolve endpoint addresses or introduce peers. Read
[Pair two machines](./pair-two-machines) for the complete identity sequence.

## Two machines on the same network

For a normal home or office LAN with internet access:

1. use the same `operator_id`;
2. start one machine and capture its endpoint ID;
3. put that ID in the other machine’s `[peers].bootstrap` list;
4. allow normal outbound DNS/HTTPS and UDP traffic;
5. start the joining machine and inspect gossip debug logs.

No router port-forward is normally required. Once the peers know each other,
iroh can advertise local/direct addresses and prefer a direct QUIC path.

Same-LAN conditions that can still prevent a direct path include:

- guest Wi-Fi or access-point client isolation;
- host firewalls that block UDP;
- enterprise endpoint software that blocks peer traffic;
- separate VLANs without routing;
- IPv4/IPv6 policy mismatches.

When both devices can reach the public relay, iroh can fall back to an encrypted
relay path even though they are physically nearby.

### Same LAN without internet

The current binary does not enable a dedicated mDNS pairing flow and accepts
only endpoint IDs—not full direct endpoint addresses—in `peers.bootstrap`.
Because the N0 resolver/relay supplies the missing address information, an
offline or air-gapped LAN is not a supported configuration today.

Adding an endpoint ID alone does not tell iroh which local IP and UDP port to
dial when discovery is unavailable.

## Two machines on different networks

For a laptop at home and a workstation at an office, or two devices behind
separate NATs:

1. both machines need outbound access to N0 discovery;
2. both need outbound access to an N0 relay;
3. UDP should be allowed for the best chance of a direct path;
4. use the same operator topic and configure a bootstrap endpoint ID;
5. do not expose either machine’s port `7892`.

[Iroh NAT traversal](https://docs.iroh.computer/concepts/nat-traversal)
begins through a relay, exchanges observed/local addresses, and attempts a
simultaneous direct connection. If direct connectivity fails, encrypted
traffic stays on the relay.

Networks that permit only outbound TCP can force relay-only operation. Networks
that block the discovery service, relay destinations, TLS interception model,
or all non-approved egress can prevent pairing completely.

## VPNs and overlay networks

A VPN can help when it gives both machines mutually routable addresses and
permits UDP, but it does not replace the application pairing contract:

- `operator_id` must still match;
- one endpoint ID must still be bootstrapped;
- the daemon still uses N0 discovery/relay configuration;
- endpoint IDs still rotate on restart.

If the VPN blocks public egress and the machines rely only on private overlay
addresses, the current endpoint-ID-only bootstrap is insufficient.

## Corporate and production networks

The public N0 infrastructure is convenient for development and personal use,
but the [public relay service](https://docs.iroh.computer/iroh-services/relays/public)
is rate-limited and has no uptime guarantee. Relays cannot read encrypted
payloads, but can observe connection metadata such as addresses, timing, and
traffic volume.

Iroh supports
[dedicated or self-hosted relays](https://docs.iroh.computer/add-a-relay),
custom discovery, and endpoint connection hooks. Sovereign Sync `0.1.0` does
not expose those settings in `config.toml`; it hard-codes `presets::N0`.

Production deployments that require any of the following need implementation
work before rollout:

- a fixed egress allow-list;
- private discovery;
- dedicated/self-hosted relays;
- relay authentication or version locking;
- endpoint allow-list/deny-list hooks;
- proxy-specific transport configuration;
- no third-party connection metadata;
- durable endpoint identities.

## Firewall guidance

Use policy goals rather than hard-coded public relay IPs, which can change:

| Goal | Firewall posture |
|---|---|
| Best direct connectivity | Permit outbound UDP and return traffic for dynamically established flows |
| Relay fallback | Permit outbound connections to the configured N0 relay infrastructure |
| Endpoint resolution | Permit the N0 DNS/Pkarr lookup and publication path |
| Local API safety | Keep TCP `7892` bound to loopback; do not forward it |
| Air-gapped operation | Not supported by the current preset/config schema |

For tightly controlled networks, capture an iroh connectivity trace in a
staging environment and have the network team approve the actual discovery and
relay policy. Do not solve an egress restriction by binding the KBD REST API to
all interfaces.

## Current observability limitations

The daemon prints its endpoint ID and topic at startup. Neighbor-up/down
messages are emitted at debug level. However:

- `/api/v1/sync/peers` is not wired to `P2PNode`;
- `/sync-peers` does not return live gossip neighbors;
- `/api/v1/sync/status` always reports the scaffold `idle` state and zero
  peers;
- no route reports direct-versus-relay path selection;
- no route reports bytes sent, received, rejected, or applied.

Use logs only for connectivity development. Do not use the current REST
responses as production network monitoring.

## Troubleshooting by symptom

| Symptom | Likely cause | Check |
|---|---|---|
| Daemon exits immediately | Empty `node.operator_id` | `[node]` in `config.toml` |
| `Ignoring invalid bootstrap peer` | Value is not a valid iroh endpoint ID | Copy only the logged endpoint ID |
| Both daemons say `idle`, no neighbor | No bootstrap path, different topic, or blocked discovery | Compare operator IDs; inspect debug logs |
| Worked until anchor restarted | Ephemeral endpoint ID became stale | Capture new anchor ID and update the joiner |
| Works by relay, not direct | UDP/NAT/firewall/VLAN restrictions | Test UDP policy and LAN isolation |
| Same LAN, no internet, cannot connect | No local discovery/full-address bootstrap | Current configuration is unsupported |
| Health works, data absent on peer | Health is local; replication pipeline is unwired | Read [Exactly what syncs](./data-scope) |

## Security reminder

Endpoint encryption authenticates the endpoint key, but the current application
does not install endpoint hooks that enforce an explicit peer allow-list.
`operator_id` separates topics; it is not a complete authorization protocol.
Only share the operator namespace and endpoint IDs with devices you intend to
pair, and do not treat the current connectivity layer as production
multi-tenant isolation.
