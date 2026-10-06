---
id: ag-ui-sse
title: AG-UI / A2UI SSE
sidebar_label: AG-UI SSE
---

# Connected event streams

`/api/v1/stream` is the AG-UI event surface. It is separate from the durable
signed-push event sequence at
`GET /api/v2/sync/pushes/{request_id}/events`. A progress event can report local
work without proving replication or a peer's application.

Resume push events with `after` or `Last-Event-ID` and the last actual sequence.
After response loss, look up the original receipt on the selected host before
new mutation. Do not substitute scaffold events, a fresh stream connection or
an empty event list for durable acceptance evidence. See
[request and receipt recovery](/docs/sovereign-sync/signed-pushes-and-receipts).

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
