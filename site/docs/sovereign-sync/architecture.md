---
id: architecture
title: Architecture
sidebar_label: Architecture
---

# Connected architecture

Headless and windowed Companion share the same-user Unix socket and local signed
KBD authority. `sovereign-sync --mode daemon` offers the same production
supervisor/consumer path as an alternative host. Choose one socket owner.
`--mode server` disables P2P; stdio `--mode mcp` connects tools to an existing host.

Local control is installed before optional P2P startup. Disabled or failed peer
transport does not manufacture a replacement signer or require losing local
control. An explicit pack manifest may attach service-supervisor reports;
its absence leaves local authority independent. Neither service reports nor
transport readiness proves a peer applied signed project state.

The MCP Unix HTTP client disables proxies and redirects, bounds its requests
and has no TCP fallback. It reports an unreachable host as unavailable rather
than reporting a private empty transport as the selected host's state.

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
