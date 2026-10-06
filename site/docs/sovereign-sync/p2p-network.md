---
id: p2p-network
title: P2P Network
sidebar_label: Network
---

# Optional peer supervision

The selected configuration can set `[node] p2p_enabled = false`. Local authority
starts before optional transport. Status reports `disabled`, `initializing`,
`bootstrapping`, `ready`, `degraded`, `failed` or `stopping`, plus retry attempt,
last error, next retry and peer IDs. Ready does not prove peer application.

Each supervisor retry uses the selected identity, bootstrap endpoint IDs and
discovery settings. mDNS defaults on and locates changing addresses for known
IDs; it does not enroll unknown peers. DHT defaults off. Enabling
`[discovery] dht = true` publishes a signed endpoint/relay/address record to the
public DHT. iroh relay behavior is separate; disabling DHT does not promise no
relay traffic. Endpoint authentication uses Ed25519 over iroh QUIC/TLS, not an
additional Noise handshake. Network pairing never grants project mutation.

Use [pairing](/docs/sovereign-sync/pair-two-machines) and inspect the
[receipt evidence](/docs/sovereign-sync/signed-pushes-and-receipts) separately.

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
