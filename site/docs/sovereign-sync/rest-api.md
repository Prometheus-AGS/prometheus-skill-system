---
id: rest-api
title: REST API Reference
sidebar_label: REST API
---

# REST API Reference

Sovereign Sync binds to `127.0.0.1:7892` in daemon/server mode. `/health` is
public on loopback. Every other route requires the focused project’s KBD
control bearer token.

## Authentication helper

```bash
PROJECT_ROOT="/path/to/project"
PROJECT_ID="$(jq -r '.projectId' "$PROJECT_ROOT/.prometheus/project.json")"

case "$(uname -s)" in
  Darwin) DATA_ROOT="$HOME/Library/Application Support" ;;
  *) DATA_ROOT="${XDG_DATA_HOME:-$HOME/.local/share}" ;;
esac

TOKEN_FILE="${PROMETHEUS_CONTROL_TOKEN_FILE:-$DATA_ROOT/prometheus/kbd/projects/$PROJECT_ID/control-token}"
TOKEN="$(tr -d '\r\n' < "$TOKEN_FILE")"
AUTH_HEADER="Authorization: Bearer $TOKEN"
```

Never commit, log, or paste the token.

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

## Sync and discovery

### `GET /api/v1/sync/status`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  http://127.0.0.1:7892/api/v1/sync/status | jq .
```

The current response reports the local scaffold state, peer list, and privacy
classification for `kbd-orchestrator`, `open-spec`, `surreal-memory`, and
`learner-model`.

### `GET /api/v1/sync/peers`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  http://127.0.0.1:7892/api/v1/sync/peers | jq .
```

### `GET /api/v1/skills/search?q=<query>&limit=<n>`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/skills/search?q=feynman&limit=5" | jq .
```

### `POST /api/v1/sync/push`

```bash
curl --fail-with-body \
  -X POST \
  -H "$AUTH_HEADER" \
  -H 'Content-Type: application/json' \
  -d '{"domain":"learner-model"}' \
  http://127.0.0.1:7892/api/v1/sync/push | jq .
```

The current handler acknowledges that the domain is queued. Treat this as
request acceptance, not proof that a peer received or applied a delta.

## KBD read endpoints

### `GET /api/v1/kbd/projects/{projectId}/status`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" | jq .
```

Returns canonical `KbdStateV2`.

### `GET /api/v1/kbd/projects/{projectId}/events`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/events" | jq .
```

Returns committed immutable events from revision 1.

### `GET /api/v1/kbd/projects/{projectId}/diagnostics`

```bash
curl --fail-with-body \
  -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/diagnostics" | jq .
```

Diagnostics include:

- quorum writable state and reason;
- Raft node, term, leader, log/apply lag, snapshot, and transport label;
- runtime revision, lifecycle, plan revision, lease, and fence;
- compatibility projection revision/match;
- signature-chain validity and event count;
- active and revoked device counts.

## KBD event stream

### `GET /api/v1/kbd/projects/{projectId}/events/stream`

```bash
curl --no-buffer \
  -H "$AUTH_HEADER" \
  -H 'Last-Event-ID: 0' \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/events/stream"
```

The server emits `kbd.events` once per second when new revisions exist and a
keepalive every 15 seconds. The SSE event ID is the latest emitted revision;
send it back as `Last-Event-ID` when reconnecting.

## KBD command endpoint

### `POST /api/v1/kbd/projects/{projectId}/commands`

The path project ID must equal `envelope.projectId`. Every command supplies a
fresh `commandId` and current `expectedRevision`. Lease-protected commands also
supply the current `leaseId` and `fencingToken`.

Example lease claim:

```bash
RUN_ID="$(curl --fail-with-body -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" |
  jq -r '.runId')"
REVISION="$(curl --fail-with-body -H "$AUTH_HEADER" \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/status" |
  jq -r '.revision')"
COMMAND_ID="$(uuidgen | tr '[:upper:]' '[:lower:]')"

jq -n \
  --arg project "$PROJECT_ID" \
  --arg run "$RUN_ID" \
  --arg command "$COMMAND_ID" \
  --argjson revision "$REVISION" \
  '{
    schemaVersion: "1",
    projectId: $project,
    runId: $run,
    commandId: $command,
    expectedRevision: $revision,
    actor: {
      kind: "harness",
      id: "operator",
      device: "workstation",
      harness: "claude-code",
      session: "manual-rest-example"
    },
    leaseId: null,
    fencingToken: null,
    command: {
      type: "claim",
      payload: {scope: "project/phase", force: false}
    }
  }' |
curl --fail-with-body \
  -X POST \
  -H "$AUTH_HEADER" \
  -H 'Content-Type: application/json' \
  --data-binary @- \
  "http://127.0.0.1:7892/api/v1/kbd/projects/$PROJECT_ID/commands" | jq .
```

Prefer `prometheus kbd` or MCP for routine operations; they construct the
envelope and current lease context safely.

## AG-UI routes

- `POST /api/v1/stream`
- `GET /api/v1/stream/ping`

Both require the bearer token. See [AG-UI SSE Reference](./ag-ui-sse).

## Error responses

| Status | Meaning |
|---|---|
| `400` | Path project ID differs from command envelope |
| `401` | Missing or invalid bearer token |
| `404` | Unknown focused project or uninitialized KBD runtime |
| `409` | Replay, revision, lease, fencing, signature, or command conflict |
| `503` | Quorum is not writable |
