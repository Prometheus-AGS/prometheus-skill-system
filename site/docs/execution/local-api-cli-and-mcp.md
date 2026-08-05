---
title: Local API, CLI, and MCP
description: Unix-socket REST routes, resumable events, command-line operations, and typed MCP tools.
---

# Local API, CLI, and MCP

The sidecar listens on a private Unix-domain socket created atomically with mode `0600`. Peer credentials must match the daemon user. The placeholder HTTP authority is `http://localhost`; no loopback TCP listener is required.

## Start a disposable local service

```bash
prometheus-exec init --identity ./exec-identity.json
prometheus-exec daemon \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack"
```

`/health` is available before runtime initialization. `/ready` returns bounded per-subsystem state and uses `503` while a required local subsystem is initializing or failed.

## REST lifecycle

| Method | Route | Purpose |
| --- | --- | --- |
| `POST` | `/api/v2/exec/runs` | Durably accept a `SignedExecRequest` |
| `GET` | `/api/v2/exec/runs/{run_id}` | Read current state and any terminal receipt |
| `GET` | `/api/v2/exec/runs/{run_id}/events?after=N` | Resume persisted SSE events after exclusive sequence `N` |
| `GET` | `/api/v2/exec/receipts/{run_id}` | Read the terminal signed receipt |
| `GET` | `/api/v2/exec/artifacts/{digest}` | Read exact content-addressed bytes |

First acceptance returns `202`; exact replay returns `200` with `replayed: true`. A request-ID/hash conflict returns `409`. Malformed JSON returns `422`; contract-invalid requests return `400`. Missing runs, receipts, or artifacts return `404`. Unavailable durable state returns `503`.

```bash
curl --unix-socket ./runtime/exec.sock \
  http://localhost/health

curl --no-buffer --unix-socket ./runtime/exec.sock \
  'http://localhost/api/v2/exec/runs/00000000-0000-0000-0000-000000000000/events?after=4'
```

Each SSE event uses the durable sequence as `id`. Reconnect with `after=<last-id>` to replay only later events, then follow live events until terminal completion.

The checked [OpenAPI 3.1 specification](/openapi/prometheus-exec.openapi.json) is generated from the Rust contracts. The [generated runtime reference](/docs/operations/generated-reference) lists every request/receipt field and route.

## CLI

`run` places code and named inputs in the CAS, signs the request with the host identity, submits it over the socket, and waits for terminal state.

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --runtime python3 \
  --code ./job.py \
  --input records=./records.json \
  --format json

prometheus-exec status \
  --socket ./runtime/exec.sock \
  --run-id '<run-uuid>' \
  --format json
```

For Tier W, use `--runtime wasm-component`, point `--code` at the authorized component, and provide `--plugin-root` in estate mode. There is no native fallback when component authorization or Wasmtime readiness fails.

## MCP stdio

`prometheus-exec mcp` starts a private same-process runner and exposes six typed tools:

- `exec-run`
- `exec-status`
- `exec-events`
- `exec-receipt`
- `exec-artifact`
- `exec-verify`

The spawning process authenticates the stdio boundary. Tool arguments never accept a private key; receipts are signed by the configured host identity. MCP calls share the same facade as REST, including request replay and event cursors. Inline artifacts are capped at 1 MiB; larger results return metadata for retrieval instead of expanding an unbounded tool response.

Next: [Remote dispatch and reconciliation](./remote-dispatch-and-reconciliation.md).
