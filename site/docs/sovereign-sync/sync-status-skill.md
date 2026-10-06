---
id: sync-status-skill
title: /sync-status Skill
sidebar_label: /sync-status
---

# Sync status skill

The `/sync-status` instruction belongs to Companion's separate connected-skill
package. It uses MCP `sync-status` against the selected host and preserves its
raw transport/domain state. Optional `domain` annotates the request; it does not
prove convergence. Unreachable host is unavailable, not an empty ready network.

Read transport attempt, last error, next retry and peer IDs. Listener health,
local authority readiness, network readiness and destination application are
separate observations. Installing either skill pack or registering MCP does not
start or certify the Companion host. See [registration](/docs/sovereign-sync/installation).

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
