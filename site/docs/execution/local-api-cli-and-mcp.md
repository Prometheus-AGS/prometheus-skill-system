---
title: Local API, CLI, and MCP
description: Unix-socket REST routes, resumable events, command-line operations, and typed MCP tools.
---

# Local API, CLI, and MCP

The sidecar listens on a private Unix-domain socket created atomically with mode `0600`. Peer credentials must match the daemon user. The placeholder HTTP authority is `http://localhost`; no loopback TCP listener is required.

## Choose the caller surface

All surfaces reach the same durable facade, but they serve different callers:

| Surface | Choose it for | Identity and replay control |
| --- | --- | --- |
| CLI | Human-operated first runs, status, doctor, and offline verification | `run` creates and signs a new request; `status` reads by run ID |
| Unix-socket REST | Applications that already construct signed contracts or need SSE | Caller controls request ID, issue time, signature, and event cursor |
| MCP stdio | AI tools that need typed execution tools without private key arguments | Private runner signs requests; caller may preserve `requestId` and `issuedAt` for replay |
| Embedded Rust/FFI | Desktop or mobile hosts that need an in-process Tier W boundary | Trusted host installs one configured API and owns the identity |

The CLI is the shortest path to a successful operation. REST and MCP are the right surfaces for deliberate same-ID replay and response-loss reconciliation.

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

Run the checked Python example after readiness succeeds:

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --runtime python3 \
  --code ./examples/prometheus-exec/tier-p/transform.py \
  --input records=./examples/prometheus-exec/tier-p/records.json \
  --timeout-ms 5000 \
  --output-mb 2 \
  --format json
```

The runnable example and expected business output are checked in under `examples/prometheus-exec/README.md`.

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

Each CLI `run` creates a new request ID. Use `status` to revisit that run. Use REST or MCP when a caller must control and resubmit the same canonical request identity.

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

The spawning process authenticates the stdio boundary. Tool arguments never accept a private key; receipts are signed by the configured host identity. MCP calls share the same facade as REST, including request replay and event cursors. For response-loss reconciliation, preserve and resubmit the same `requestId`, `issuedAt`, and canonical `exec-run` arguments. The service returns the original `runId`, `requestHash`, and receipt with `replayed: true`; a reused ID with a different canonical payload fails as a conflict. If `requestId` and `issuedAt` are omitted, the server creates a one-shot identity and timestamp.

The MCP request shape is generated from the Rust contract. A typical Tier P call supplies `runtime`, base64url code, optional named base64url inputs, limits, and stable request identity. The tool returns durable run status, not an unstructured shell transcript. See the [generated runtime reference](/docs/operations/generated-reference) for every field.

`exec-events` returns at most 100 events and 8 MiB of serialized event data per page. Pass the returned exclusive `nextAfter` cursor as the next call's `after`; `hasMore` indicates that another page is available. Every successful nonempty page advances the cursor. If one event cannot fit the hard byte ceiling, the call fails explicitly instead of returning an empty page that could loop. The server validates the hash chain while reading only the bounded page into the response.

Inline artifacts are capped at 1 MiB. When an artifact exceeds the caller's effective ceiling, `exec-artifact` returns `inline: false`, the exact digest and size, and a `retrieval` object containing the `unix-domain-http` transport, `GET` method, private runner `socketPath`, and `/api/v2/exec/artifacts/{digest}` path. The caller can stream the complete bytes without expanding an unbounded MCP response while the MCP process remains active:

```bash
curl --fail --silent --show-error \
  --unix-socket "$STATE_DIR/.mcp-runner.sock" \
  "http://localhost/api/v2/exec/artifacts/$DIGEST" \
  --output artifact.bin
```

Next: [Remote dispatch and reconciliation](./remote-dispatch-and-reconciliation.md).
