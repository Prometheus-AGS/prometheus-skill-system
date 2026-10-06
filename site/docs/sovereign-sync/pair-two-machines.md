---
id: pair-two-machines
title: Pair Two Machines
sidebar_label: Pair Two Machines
---

# Pairing two selected hosts

Choose the configuration and identity on each device; retain separate private
signing keys. Network membership and KBD project/replica enrollment are separate.
From the selected Companion binary, A exports with
`sovereign-sync --config /absolute/config.toml --mode pair-export`.
Transfer its secret ticket privately to B; B imports with `--mode pair-import`
and `--ticket <private-ticket>`, then exports its updated ticket back to A for
import. This round trip keeps the group secret shared.

Import replaces the selected identity's group secret and updates its allowlist;
back up existing membership first. Ticket arguments may appear in shell history
or process listings. Never put tickets in shared logs, issues or transcripts.

Add each authorized endpoint ID to the other configuration's `[peers] bootstrap`
list, then restart the one selected host to read identity/config changes.
mDNS supplies addresses for known IDs. The source provides a pairing URI payload,
not a shipped QR screen, short-code flow or group UI. Neither pairing nor
connectivity proves an authorized signed push was applied at the destination.

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
