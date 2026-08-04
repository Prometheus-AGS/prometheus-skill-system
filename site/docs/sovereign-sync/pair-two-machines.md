---
id: pair-two-machines
title: Pair Two Machines
sidebar_label: Pair Two Machines
---

# Pair Two Machines

Pairing transfers one opaque ticket from an existing peer to a joining peer.
The ticket contains protocol version, the random 256-bit group secret, the
exporting endpoint ID, and its signing-key fingerprint. Treat the complete ticket
as a secret: do not paste it into logs, issue trackers, shell history, or release
evidence.

## Identity map

| Identity | Shared? | Persistence | Purpose |
|---|---:|---:|---|
| 256-bit group secret | Yes, only within the paired group | Stored in each mode-`0600` P2P identity file | Derives the private gossip topic |
| iroh endpoint secret/key | No | Atomically persisted across restarts | Gives one peer a stable endpoint ID |
| Endpoint allow-list binding | Each peer enrolls the other | Persisted with P2P identity | Binds endpoint ID to signing-key fingerprint |
| KBD project ID | Only for replicas of one project | Repository manifest + runtime registry | Names one KBD authority |
| KBD device key | No | Credential store or mode-`0600` file | Signs KBD commands/events |

The removed `operator_id` is not a pairing credential. New groups use a random
secret and explicit enrollment, so a guessable name cannot join a topic.

## 1. Initialize stable identities

Run on both machines with their own config:

```bash
sovereign-sync --mode init
```

The output reports the P2P identity path and endpoint ID, never the group secret.
The identity file must be regular, atomically written, and mode `0600`.

## 2. Export on the existing peer

On Machine A:

```bash
PAIR_TICKET="$(sovereign-sync --mode pair-export)"
```

Transfer `PAIR_TICKET` through an authenticated confidential channel. Do not use
`set -x`, command tracing, CI variables, or chat transcripts.

## 3. Import on the joining peer

On Machine B:

```bash
sovereign-sync --mode pair-import --ticket "$PAIR_TICKET"
```

Import validates the ticket protocol, 32-byte group secret, endpoint ID, and
fingerprint binding before persisting the group and allow-list entry. The command
prints only the paired endpoint and fingerprint.

Export Machine B’s ticket and import it on Machine A so both sides have explicit
allow-list bindings:

```bash
# Machine B
PAIR_TICKET_B="$(sovereign-sync --mode pair-export)"

# Machine A, after confidential transfer
sovereign-sync --mode pair-import --ticket "$PAIR_TICKET_B"
```

## 4. Start isolated peers and verify

The default local API is a Unix socket; it does not open port 7892:

```bash
SOCKET_PATH="$HOME/.prometheus/run/sovereign-sync.sock"
sovereign-sync --mode daemon --socket "$SOCKET_PATH"
curl --unix-socket "$SOCKET_PATH" \
  http://localhost/health
```

Use the actual socket path reported by your installation on macOS. Loopback TCP
is opt-in with `--tcp` and requires a bearer token loaded from a mode-`0600` file.

Create a signed v2 push, then poll its durable receipt or resume the event stream.
A local `broadcast` state alone is not peer application evidence. Certification
requires a per-peer `received`, `applied`, or `rejected` receipt and proves the
same terminal receipt survives restart.

## Rejection behavior

Inbound frames fail closed when the endpoint is unknown, the group secret/topic
does not match, the endpoint and signing key disagree, the request is stale, or a
request ID is replayed. The full ticket and group secret are never logged.

To remove a peer, delete its endpoint-to-signing-key binding through the supported
identity-management path and restart the isolated peer. Creating a new group
secret is a group rotation, not an `operator_id` rename.
