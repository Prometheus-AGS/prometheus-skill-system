---
title: Signed pushes and receipts
description: End-to-end design for authenticated sync, exact replay, SSE resume, and reconciliation.
---

# Signed push intent, receipts and recovery

MCP `sync-push` accepts an explicit syncable domain, such as
`{ "domain": "kbd-control:<project-id>" }`. The real enrolled runtime signs the
request; do not fabricate signatures or transfer session credentials as authority.

Before POST, the bridge fsyncs the exact request and socket in a private file:
`<data-root>/prometheus/companion/mcp-push-outbox/<request-id>.json`.
It requires an existing absolute data root and absolute selected socket and
returns `intentPath`. Recording failure returns `not-submitted` without send.
Conflicting or malformed intent is preserved for recovery. This is durable
intent, not a canonical event or acceptance receipt. SDK callers must persist
their own request before `sync_push_signed`.

A connection, timeout or response-decoding failure is uncertain. Recover the
original request/endpoint from the outbox, then call `sync-push-receipt` with
`{ "request_id": "<original-id>" }` on the original host. A 404 does not prove
an outstanding concurrent request was never received. Resolve outstanding work
before electing an authorized replay.

For replay, retain exactly the original ID, domain, targets, expected frontier,
time, signer and signature; supply the unchanged `signed_request`. The REST
schema `1.7` uses `schemaVersion`, `requestId`, `domain`, optional
`targetEndpointIds`/`expectedFrontier`, `issuedAtMs`, `signerKeyId`, `signature`.
Same ID/canonical payload returns the persisted receipt; changed content
conflicts. Signer, freshness and frontier checks remain enforced.

Receipt `localState` may be `accepted`, `prepared`, `applied_locally`,
`broadcast` or `failed`. Broadcast is a local send, not peer application. Local
preparation may precede failed broadcast. Replaying a failed receipt does not
restart it; inspect the receipt/frontier before authorizing new work. Preserve
intent plus receipts through upgrade/restart. Resume receipt events with `after`
or `Last-Event-ID` using the last real sequence.

## Confirm that a particular peer applied an update

For two local Companion authorities, enroll both replicas for the same project and pair their network identities. Address the signed push to the receiving endpoint. From the receiver, read the signed project audit and compare its ordered event IDs and integrity hashes with the sender. The sender's receipt alone cannot tell you what a restarted receiver retained.

In the Full 1.18 local exercise the receiver restarted during a 40-command backlog. It retained 11 events including three baseline events, applied the remaining 32, and matched the sender's 43 unique signed events in order. Two independent runs used the same binaries. This exercise covered one Mac. [Interrupted-transfer receipts](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/30e63d55/.kbd-orchestrator/phases/vlbv-r4-portability/children/intersession-communication/boundary-results.json)

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
