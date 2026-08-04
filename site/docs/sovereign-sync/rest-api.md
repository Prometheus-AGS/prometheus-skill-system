---
id: rest-api
title: REST API Reference
sidebar_label: REST API
---

# REST API Reference

The local API uses a same-user Unix-domain socket by default. The socket is
created atomically with mode `0600`; the server validates peer credentials on
macOS and Linux. Loopback TCP exists only with `--tcp` and requires a bearer
token read from a mode-`0600` file.

The generated OpenAPI 3.1 contract is available at `/openapi.json`, from
`sovereign-sync --mode openapi`, and as a checked-in release artifact under
`site/static/openapi/`.

## Liveness and readiness

- `GET /health` proves the dedicated HTTP runtime is bound and responsive. It
  does not read KBD, Loro, the receipt store, or P2P state.
- `GET /ready` performs bounded authority checks and returns `503` until the
  application state is installed and readable.

This separation keeps liveness available while slow P2P startup or an invalid
authority is being diagnosed.

## Create a signed push

`POST /api/v2/sync/pushes`

```json
{
  "schemaVersion": "1.7",
  "requestId": "f2d84eaf-5474-4e28-971d-4bc4334c1fb7",
  "domain": "learner-model",
  "targetEndpointIds": [],
  "expectedFrontier": null,
  "issuedAtMs": 1785772800000,
  "signerKeyId": "ed25519:device-key-id",
  "signature": "base64-signature"
}
```

The signature covers RFC 8785/JCS canonical JSON of every request field except
`signature`. `requestId` must be a UUID. `issuedAtMs` must be within the accepted
five-minute window (with 30 seconds of future clock tolerance). Target endpoint
IDs must be valid and unique. `expectedFrontier` is accepted only for
`kbd-control:<project-id>` domains.

The signer must be an active enrolled KBD device for the selected scope.

## Receipt semantics

A successful new submission returns `201` and a durable `PushReceipt`:

```json
{
  "schemaVersion": "1.7",
  "pushId": "f2d84eaf-5474-4e28-971d-4bc4334c1fb7",
  "canonicalPayloadHash": "blake3-hex",
  "domain": "learner-model",
  "targets": [],
  "localState": "broadcast",
  "perPeer": {},
  "createdAtMs": 1785772800001,
  "updatedAtMs": 1785772800003,
  "events": []
}
```

Local states are `accepted`, `prepared`, `applied_locally`, `broadcast`, and
`failed`. Per-peer receipts separately record `received`, `applied`, or
`rejected` state and failure detail. A local broadcast is not proof of peer
application.

### Exact replay and conflict

- Same `requestId` and same canonical payload hash returns the exact persisted
  receipt with `200`; execution is not repeated.
- Same `requestId` and a different canonical payload hash returns `409`.

This is the response-loss recovery contract: after a client times out, resend
the identical signed request or retrieve the receipt by ID.

## Retrieve and resume

- `GET /api/v2/sync/pushes/{push_id}` returns the durable current receipt.
- `GET /api/v2/sync/pushes/{push_id}/events?after=<sequence>` returns SSE events
  strictly after the supplied sequence.

`Last-Event-ID` is also accepted. Event sequences are monotonic and persisted
with the receipt, so a reconnect does not require replaying already observed
events.

```bash
curl --unix-socket "$SOCKET_PATH" --no-buffer \
  -H 'Last-Event-ID: 2' \
  "http://localhost/api/v2/sync/pushes/$PUSH_ID/events"
```

## Error envelope

```json
{
  "error": {
    "code": "request_id_conflict",
    "message": "requestId already exists with a different canonical payload hash",
    "pushId": "f2d84eaf-5474-4e28-971d-4bc4334c1fb7"
  }
}
```

| Status | Typical meaning |
|---:|---|
| `400` | Unsupported schema, malformed request, canonicalization failure |
| `401` | Canonical signature is invalid |
| `403` | Signer is unknown or revoked, or the domain is prohibited |
| `404` | Push, domain, or KBD project scope does not exist |
| `409` | Request-ID/hash conflict or expected-frontier mismatch |
| `422` | Invalid targets, stale request, or frontier on an unsupported domain |
| `503` | Receipt store, authority, envelope preparation, or transport unavailable |

## Deprecated v1 compatibility

`POST /api/v1/sync/push` is unsigned and available only over the same-user Unix
socket during the 1.7 transition. Responses carry `Deprecation`, `Sunset`, and
successor-version headers. It is rejected over TCP and P2P. New callers use v2.

The v1 status, peers, search, KBD read/control, claims, conflicts, audit, and
AG-UI routes remain available. KBD command routes continue to require their own
signed schema-v2 command envelopes. Scalar revision is derived compatibility
data; concurrency is decided by the causal frontier.
