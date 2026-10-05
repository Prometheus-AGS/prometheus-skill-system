---
id: privacy-model
title: Privacy Model
sidebar_label: Privacy Model
---

# Replication privacy

Transport membership, project authorization and content classification are
separate decisions. Local-only surreal-memory has a never-synced adapter;
trusted/public domain labels are not permission to transmit arbitrary data.
Keep raw memory, credentials, session material and private keys local.

Default connected control uses a same-user Unix socket. Explicit loopback TCP
requires a private bearer-token file and does not enable the deprecated unsigned
mutation route. The MCP bridge has no TCP fallback. Pairing secrets require
private transfer, while KBD replica signing authority remains independently checked.

Optional DHT publication and iroh relay traffic are distinct configuration
choices. Disabling DHT alone is not a no-network/no-relay guarantee. See
[peer settings](/docs/sovereign-sync/p2p-network) and
[data scopes](/docs/sovereign-sync/data-scope).

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
