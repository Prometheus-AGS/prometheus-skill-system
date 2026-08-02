---
id: rest-api
title: REST API Reference
sidebar_label: REST API
---

# REST API Reference

Sovereign Sync binds to `127.0.0.1:7892` in daemon/server mode. Read routes are
available to local processes. Every remote KBD command POST requires a
schema-v2 `SignedCommandEnvelope` signed by an active enrolled device.

```bash
PROJECT_ROOT="/path/to/project"
PROJECT_ID="$(jq -r '.projectId' "$PROJECT_ROOT/.prometheus/project.json")"
```

## Health

### `GET /health`

```bash
curl --fail-with-body http://127.0.0.1:7892/health | jq .
```

```json
{
  "status": "ok",
  "service": "sovereign-sync",
  "version": "0.1.0"
}
```

### `GET /ready`

Asynchronously replays the journal and returns `503` when the authority is not
reachable or valid. `/health` remains static and store-independent.

## Sync and discovery

### `GET /api/v1/sync/status`

```bash
curl --fail-with-body \
  http://127.0.0.1:7892/api/v1/sync/status | jq .
```

The current response reports the local scaffold state, peer list, and privacy
classification for `kbd-orchestrator`, `open-spec`, `surreal-memory`, and
`learner-model`.

### `GET /api/v1/sync/peers`

```bash
curl --fail-with-body \
  http://127.0.0.1:7892/api/v1/sync/peers | jq .
```

### `GET /api/v1/skills/search?q=<query>&limit=<n>`

```bash
curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/skills/search?q=feynman&limit=5" | jq .
```

### `POST /api/v1/sync/push`

```bash
curl --fail-with-body \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"domain":"learner-model"}' \
  http://127.0.0.1:7892/api/v1/sync/push | jq .
```

The handler validates the domain, exports its real owner state, prepares Loro
updates, and returns either `broadcast` or `applied-locally-only`. A broadcast
response is not proof that a peer applied the update.

## KBD read endpoints

### `GET /api/v1/kbd/projects`

Returns the machine registry, including all project and replica identities plus
any project that could not be opened by the router.

### `POST /api/v1/kbd/projects/register`

```bash
curl --fail-with-body \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"path":"/path/to/project"}' \
  http://127.0.0.1:7892/api/v1/kbd/projects/register | jq .
```

The path must already contain `.prometheus/project.json`. Registration assigns
a replica UUID but never invents or changes the project UUID.

### `GET /api/v1/kbd/projects/{projectId}/replicas`

Returns every registered replica path for the declared project UUID.

### `GET /api/v1/kbd/projects/{projectId}/status`

```bash
curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" | jq .
```

Returns canonical `KbdStateV2`.

### `GET /api/v1/kbd/projects/{projectId}/events`

```bash
curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/events" | jq .
```

Returns committed immutable events from revision 1.

### `GET /api/v1/kbd/projects/{projectId}/diagnostics`

```bash
curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/diagnostics" | jq .
```

Diagnostics include:

- quorum writable state and reason;
- single-writer node and lock path;
- replica journal path, byte size, event count, Lamport, and ingestion state;
- Loro snapshot path/hash, authority event count, derived revision, frontier, and conflict count;
- runtime derived revision, frontier, lifecycle, and plan revision;
- compatibility projection revision/match;
- signature-chain validity and event count;
- active and revoked device counts.

## KBD event stream

### `GET /api/v1/kbd/projects/{projectId}/events/stream`

```bash
curl --no-buffer \
  -H 'Last-Event-ID: 0' \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/events/stream"
```

The server emits `kbd.events` once per second when new events exist and a
keepalive every 15 seconds. The SSE event ID is a deterministic authority
cursor (not a branch-local scalar revision); send it back as `Last-Event-ID`
when reconnecting.

## KBD command endpoint

### `POST /api/v1/kbd/projects/{projectId}/commands`

The path project ID must equal `signed.command.projectId`. The outer object
contains `command`, `signerKeyId`, and an Ed25519 `signature` over canonical
command bytes plus the signer key ID. Every normal command supplies a fresh
`commandId` and the current causal `frontier`. Unsigned, schema-v1, unknown,
revoked, or incorrectly signed remote commands fail closed.

Use `prometheus kbd` or `sovereign-client` to construct signatures; shell
examples below show the inner command only and are not directly POSTable.

Example cancellation command:

```bash
RUN_ID="$(curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" |
  jq -r '.runId')"
FRONTIER="$(curl --fail-with-body \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" |
  jq -c '.frontier')"
COMMAND_ID="$(uuidgen | tr '[:upper:]' '[:lower:]')"

jq -n \
  --arg project "$PROJECT_ID" \
  --arg run "$RUN_ID" \
  --arg command "$COMMAND_ID" \
  --argjson frontier "$FRONTIER" \
  '{
    schemaVersion: "2",
    projectId: $project,
    runId: $run,
    commandId: $command,
    frontier: $frontier,
    actor: {
      kind: "harness",
      id: "operator",
      device: "workstation",
      harness: "claude-code",
      session: "manual-rest-example"
    },
    command: {
      type: "cancel",
      payload: {reason: "Operator abandoned this run"}
    }
  }'
```

Prefer `prometheus kbd` or MCP for routine operations; they construct the
envelope safely.

Schema-v2 concurrency is decided exclusively by `frontier`; scalar revision
is a derived compatibility projection.

## KBD conflicts and resolution

`GET /api/v1/kbd/projects/{projectId}/conflicts` returns every deterministic
conflict record, including all candidates, the provisional or adjudicated
winner, and resolution provenance.

`POST /api/v1/kbd/projects/{projectId}/conflicts/{conflictId}/resolve` accepts
the same schema-v2 command envelope, with a matching `conflict_resolve` command.
The actor must be an operator and `winnerEventId` must name one of the recorded
candidates. Resolution appends a signed event; it never rewrites history.

## KBD claims

- `GET /api/v1/kbd/projects/{projectId}/claims`
- `POST /api/v1/kbd/projects/{projectId}/claims/acquire`
- `POST /api/v1/kbd/projects/{projectId}/claims/renew`
- `POST /api/v1/kbd/projects/{projectId}/claims/release`

Mutation bodies are signed command envelopes whose inner command is
`claim_acquire`, `claim_renew`, or `claim_release`. Claims carry project and
replica identity, work scope, shared/exclusive mode, holder, expiry, and a
monotonic token.

## AG-UI routes

- `POST /api/v1/stream`
- `GET /api/v1/stream/ping`
- `GET /api/v1/events`

The continuous event route emits `event_appended`, `claim_acquired`,
`claim_conflict`, and `singleton_violation`. See [AG-UI SSE Reference](./ag-ui-sse).

## Error responses

| Status | Meaning |
|---|---|
| `400` | Path project ID differs from command envelope |
| `401` | Unknown, revoked, unsigned, or invalid device signature |
| `404` | Unknown registered project or uninitialized KBD runtime |
| `409` | Replay, revision, signature, or command conflict |
| `503` | Quorum is not writable |
