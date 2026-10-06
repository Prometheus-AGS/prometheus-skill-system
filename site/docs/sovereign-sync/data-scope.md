---
id: data-scope
title: Exactly What Syncs
sidebar_label: Exactly What Syncs
---

# Replication domains and ownership

The current host metadata labels `skill-index` public, `learner-model` and
`kbd-control` trusted, and `surreal-memory` local-only with a never-synced adapter.
These describe policy and wiring, not completed replication. Never push local-only
memory payloads or treat a project ID as permission to share its data.

Full native memory uses `memory/mcp`; mini Compose uses `memory/main_local_384`.
Companion service adoption or peer configuration does not merge or migrate them.
Git carries reviewed source/specification documents. Raw memory, enrolled private
keys, session credentials and transcripts are not portable authority.

Choose an implemented syncable domain, authorized project/replica and target,
then preserve its exact request, frontier and durable receipt. Destination
application needs its own actual evidence.

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
