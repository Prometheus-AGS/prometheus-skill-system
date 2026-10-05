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

## Ownership and evidence

These routes remain useful after relocation. The current recovered Companion
source (`docs/installation.md`, `docs/control-api.md`) has no public remote or
certified release yet. Source inspection is not installed or peer acceptance;
final source/artifact identities and publication links remain release-owned.

Use [local KBD](/docs/kbd/control-plane) without the optional extension, and the
[service operations guide](/docs/guide/service-operations#optional-companion)
for pack/Companion ownership. The [integration contract](/docs/kbd/integration-contract)
is a one-way seam. [Relocation history](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/decisions/sovereign-sync-relocated-to-companion.md)
is a decision record, not a Companion repository URL.
