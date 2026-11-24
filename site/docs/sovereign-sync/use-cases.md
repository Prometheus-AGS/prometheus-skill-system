---
id: use-cases
title: Real-World Use Cases
sidebar_label: Real-World Use Cases
---

# Cross-device continuity

A developer can keep local signed KBD work independent of networking, then
optionally synchronize an authorized project through Companion. Choose one
socket host, enroll each project/replica and configure authorized network peers.
Before a push, record the source frontier and exact signed intent; after it,
inspect the original host's receipt and destination application evidence.

If the response is lost, recover its durable intent and query the original ID
rather than creating another operation. If peers are unavailable, retain local
state, outbox and receipt evidence; transport recovery does not by itself settle
application outcomes.

This is a supported source workflow, not an observed cross-device acceptance
result. It does not replicate local-only memory, move credentials or activate an
agent team on the destination. See [installation](/docs/sovereign-sync/installation)
and [push recovery](/docs/sovereign-sync/signed-pushes-and-receipts).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
