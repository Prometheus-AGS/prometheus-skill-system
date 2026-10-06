---
id: sync-peers-skill
title: /sync-peers Skill
sidebar_label: /sync-peers
---

# Sync peers skill

Companion's `/sync-peers` instruction reads MCP `sync-peers` from the selected
host. It lists peers and transport state; it does not enroll, admit, restart or
authorize a peer. A listed peer is not evidence it applied a signed project push.

Use the separately authorized [pairing procedure](/docs/sovereign-sync/pair-two-machines)
for membership changes. Keep the same host socket and its configured identity,
and retain uncertainty rather than switching endpoints after a failed observation.

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
