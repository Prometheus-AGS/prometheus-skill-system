---
id: overview
title: Sovereign Sync Overview
sidebar_label: Overview
slug: /sovereign-sync/overview
---

# Sovereign Sync

Sovereign Sync is the Prometheus service intended to let trusted machines carry
the same operational context without turning one machine into a public server.
It combines:

- an iroh peer-to-peer connection layer;
- Loro conflict-free replicated data types (CRDTs) for mergeable data;
- a privacy manifest that decides which named data domains may leave a device;
- an authenticated loopback API and MCP tool surface; and
- the local KBD command authority used by Claude Code, Codex, OpenCode, Kimi,
  and CLI operators.

The design goal is continuity: a user should be able to stop work on one
trusted machine and resume on another with the relevant project position,
learning state, and approved knowledge already present.

:::warning Current release boundary

Version `0.1.0` contains the network, CRDT, privacy, storage, API, and
single-node KBD building blocks, but the daemon does **not yet connect them into
an end-to-end project data replication pipeline**.

- `POST /api/v1/sync/push` returns a queue acknowledgement but does not export
  project data or broadcast a delta.
- `GET /api/v1/sync/status` and `/sync/peers` return bounded scaffold data, not
  live P2P state.
- the daemon drops the P2P incoming-message receiver;
- the installed learner model, Karpathy wiki, loop directories, OpenSpec
  files, and KBD authority are not automatically ingested into sync domains;
- authoritative KBD replication is explicitly disabled between processes.

Today, pairing two machines proves network-topic membership only. It does not
prove that project or global data moved. The pages in this section distinguish
the intended contract from the behavior operators can verify now.

:::

## Why sync exists

Prometheus produces useful state outside the source tree:

- KBD knows the active phase, next command, progress, decisions, leases, and
  handoffs;
- the Feynman system records mastery, knowledge gaps, observations, sessions,
  and FSRS review schedules;
- the Karpathy learning loop produces project and shared wiki entries,
  reflection events, learning logs, and skill-improvement candidates;
- the outer loop, evolver, ZeeSpec, Forge, and skill creator each maintain
  resumable state.

Git remains the right way to exchange source code and reviewed documentation.
It is not a live state transport for every runtime database, local learning
record, or in-progress loop. Sovereign Sync is intended to fill that gap with
explicitly classified, peer-to-peer replication.

## The intended operating model

1. Install Sovereign Sync on each trusted machine.
2. Give the machines the same **operator ID** so they derive the same private
   gossip topic.
3. Exchange their distinct **iroh endpoint IDs** so at least one machine can
   bootstrap a connection.
4. Keep the same immutable **project ID** for clones of the same repository.
5. Register an explicit sync domain with `Public`, `Trusted`, or `Local`
   privacy.
6. Convert the domain owner’s state into a CRDT snapshot or delta.
7. Send the encrypted payload to authorized peers.
8. Merge it into the matching domain and advance its version vector.

Steps 5–8 are implemented as libraries and tests but are not wired to the
daemon’s project/global data producers in `0.1.0`.

## Three planes, three different jobs

| Plane | Purpose | Scope today |
|---|---|---|
| Local control | REST, SSE, MCP, KBD commands, search | Operational on one focused project |
| P2P connectivity | Endpoint discovery, NAT traversal, relay fallback, gossip topic | Starts in daemon mode; pairing is log-driven |
| Domain replication | Manifest gate, CRDT export/import, per-domain versions | Library-tested; not connected to daemon data sources |

The KBD control plane is intentionally separate from CRDT merge. A learner
model or public skill index can merge concurrent updates. KBD command authority
cannot: it needs one ordered, signed, fenced event chain.

## What sync is not

Sovereign Sync is not:

- a replacement for Git, package managers, or the skill installer;
- a backup system or historical archive;
- remote access to the loopback REST API;
- a folder mirroring tool that copies everything under `.prometheus/`;
- permission to transmit a domain merely because a file exists;
- a way to merge two independent offline KBD writers;
- a reason to copy device keys, bearer tokens, or other credentials.

## Architecture

```mermaid
flowchart TB
  Client["CLI, MCP harness, or desktop backend"] --> Local["Loopback REST, SSE, and MCP"]
  Local --> KBD["Local KBD authority<br/>flocked journal + signed events"]
  Producer["Explicit domain adapter<br/>not yet daemon-wired"] --> Gate["SyncManifest privacy gate"]
  Gate --> CRDT["Loro snapshot or delta"]
  CRDT --> Gossip["iroh-gossip operator topic"]
  Gossip --> Peer["Trusted peer"]
  Peer --> Import["Manifest check + CRDT import"]
  KBD -. "authoritative cross-process transport not yet enabled" .-> Peer
```

See [Architecture](./architecture) for the component and trust boundaries.

## Command modes

| Mode | When to use | Port |
|---|---|---|
| `--mode init` | Create the local KBD device-signing key | none |
| `--mode mcp` | Let a harness call search, sync-summary, and KBD tools | stdio |
| `--mode daemon` | Start P2P setup and the local HTTP/KBD service | loopback `7892` |
| `--mode server` | Start only the local HTTP/KBD service for debugging | loopback `7892` |
| `--mode status` | Probe the local daemon health route | none |

## Quick start

```bash
# Build/install tools and managed services
bash scripts/check-prerequisites.sh --install --build-tools
bash scripts/install-mcp-services.sh

# Verify the daemon is up
curl --fail-with-body http://127.0.0.1:7892/health | jq .

# Read the current bounded sync summary
/sync-status
```

The health route is intentionally unauthenticated on loopback. All other REST
calls require the focused project’s KBD control token. See
[Tokens and authentication](/docs/kbd/tokens-and-authentication).

## Read next

- [Pair two machines](./pair-two-machines) explains every identifier and the
  current log-driven bootstrap procedure.
- [Network configuration](./p2p-network) covers same-LAN, different-network,
  VPN, firewall, relay, and air-gapped constraints.
- [Exactly what syncs](./data-scope) inventories project and global data,
  including every loop and Karpathy artifact family.
- [Real-world use cases](./use-cases) shows the operator value and the current
  workaround for each scenario.

---
*Canonical source: [`substrate/sovereign-sync`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/sovereign-sync) — the crate and its doc comments are the source of truth.*
