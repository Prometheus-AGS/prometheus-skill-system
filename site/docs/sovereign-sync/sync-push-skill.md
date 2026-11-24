---
id: sync-push-skill
title: /sync-push Skill
sidebar_label: /sync-push
---

# Sync push skill

Companion's `/sync-push` instruction uses an explicit authorized syncable domain
and MCP `sync-push`. A new request uses the enrolled project signer; ambiguous
generic domains need an explicit project choice. A sync handoff supplies neither
another repository's write authority nor native session credentials.

Before POST, the MCP bridge records exact signed intent and endpoint durably.
On uncertainty, preserve `intentPath`, request ID/body and host, then resolve
`sync-push-receipt` before any replay. Do not issue a new ID to settle response
loss. Local accepted/broadcast state is not remote peer application. Follow
[the complete recovery contract](/docs/sovereign-sync/signed-pushes-and-receipts).

## Companion references

The Companion project owns this interface. [Companion control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md) · [Installation](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) · [Local KBD without Companion](/docs/kbd/control-plane)
