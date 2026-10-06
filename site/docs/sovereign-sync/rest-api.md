---
id: rest-api
title: REST API Reference
sidebar_label: REST API
---

# Selected control API

The default API is HTTP over a same-user Unix socket, not a public `:7892`
listener. Explicit loopback `--tcp` daemon/server operation requires a private
bearer-token file. Connected MCP sync does not use that alternate transport.

| Route | Meaning |
|---|---|
| `GET /health` | Listener/process liveness |
| `GET /ready` | Local authority startup readiness |
| `GET /openapi.json` | Current signed sync schema/examples |
| `GET /api/v1/sync/status` | Selected host transport/domain metadata |
| `GET /api/v1/sync/peers` | Selected host peer snapshot |
| `GET /api/v1/services` | Manifest-supervisor reports, or an empty list |
| `GET /api/v1/kbd/projects` | Registered project routes |
| `/api/v1/kbd/projects/{project_id}/...` | Signed project authority/control routes |
| `POST /api/v2/sync/pushes` | Signed receipt-backed push |
| `GET /api/v2/sync/pushes/{request_id}` | Durable receipt on original host |
| `GET /api/v2/sync/pushes/{request_id}/events` | Receipt event sequence/resume |

Project identity, authorized replica and expected frontier remain canonical
checks. The router and OpenAPI define the complete inventory; this table is
not authority to invoke an arbitrary mutation. Deprecated unsigned v1 push
is same-user Unix-only and lacks the v2 receipt contract. TCP does not revive
unsigned mutation. See [push recovery](/docs/sovereign-sync/signed-pushes-and-receipts).

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
