# Decision: use same-user Unix sockets and durable explicit P2P pairing

**Status:** accepted · 2026-08-03 · release 1.7.0

## Context

An always-open loopback port broadens local access, ephemeral iroh keys break
restart identity, and a human-readable topic namespace is not group
authorization.

## Decision

The local API defaults to an atomically created mode-`0600` Unix socket and
enforces same-user peer credentials on macOS and Linux. Loopback TCP is explicit
and requires a mode-`0600` bearer-token file. The iroh secret key is atomically
persisted, producing a stable endpoint ID. Pairing tickets carry protocol
version, a random 256-bit group secret, endpoint ID, and signing-key fingerprint.
An allow-list binds endpoint IDs to enrolled signing keys; stale and replayed
requests fail closed.

## Alternatives considered

- Loopback TCP by default was rejected because every local process could reach
  it without an application credential.
- Ephemeral endpoints were rejected because restart invalidated bootstrap data.
- Topic derivation from an operator name was rejected as guessable and
  unauthenticated.

## Consequences

Pairing requires confidential ticket exchange and peer enrollment. Operators
gain stable restart behavior and explicit local/P2P identity boundaries.

## Verification

Disposable two-peer tests assert socket mode/credentials, token permissions,
stable endpoint restart, pairing, allow-list rejection, wrong-group rejection,
staleness, replay rejection, and secret-safe logs.
