---
name: flint-sdk-ts
description: >
  Install and use the Flint Realtime Fabric TypeScript/JavaScript SDK (@prometheusags/frf-sdk).
  Covers SpineClient setup, channel subscriptions, event publishing, ack handling, and Connect-RPC
  transport configuration for browser and Node.js environments.
version: '1.0.0'
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: flint
  tags: [flint, realtime, typescript, javascript, sdk, websocket, grpc, connect-rpc]
---

# flint-sdk-ts

Use the **Flint Realtime Fabric** TypeScript SDK to subscribe to channels, publish events, and handle acknowledgements in browser or Node.js applications.

## When to use

- Adding realtime event streaming to a React, Next.js, Vite, or Node.js app.
- Consuming Flint Realtime Fabric (FRF) channels via the TypeScript client.
- Generating types from the FRF proto schema.

## Installation

```bash
# npm
npm install @prometheusags/frf-sdk

# pnpm
pnpm add @prometheusags/frf-sdk

# yarn
yarn add @prometheusags/frf-sdk
```

**Peer dependencies** (Connect-RPC transport):
```bash
npm install @connectrpc/connect @connectrpc/connect-web
```

## Core exports

| Export | Purpose |
|--------|---------|
| `SpineClient` | Main client for subscribe / publish / ack |
| `SpineService` | Connect-RPC service descriptor |
| `EventKind` | Enum: `DATA`, `SYSTEM`, `HEARTBEAT` |
| `EventEnvelope` | Protobuf message type for events |
| `SubscribeRequest` | Request type for subscribe calls |
| `PublishRequest` / `PublishResponse` | Request/response for publish |
| `AckRequest` / `AckResponse` | Request/response for ack |
| `Channel` | Channel descriptor type |
| `Cursor` / `Offset` | Stream position types |

## Minimal example

```typescript
import { SpineClient } from '@prometheusags/frf-sdk'
import { createConnectTransport } from '@connectrpc/connect-web'

const transport = createConnectTransport({ baseUrl: 'https://your-frf-gateway' })
const client = new SpineClient(transport)

// Subscribe to a channel
const stream = client.subscribe({ channel: { name: 'my-channel' } })
for await (const event of stream) {
  console.log(event.kind, event.payload)
  await client.ack({ cursor: event.cursor })
}
```

## Publish events

```typescript
await client.publish({
  channel: { name: 'my-channel' },
  payload: new TextEncoder().encode(JSON.stringify({ msg: 'hello' })),
})
```

## Environment variables

| Variable | Purpose |
|----------|---------|
| `FRF_GATEWAY_URL` | Base URL of the FRF gateway (e.g. `https://frf.example.com`) |
| `FRF_AUTH_TOKEN` | Bearer token for authenticated channels (optional) |

## SDK source

Source code lives at: `<flint-realtime-fabric>/sdks/ts/`. Resolve the repository root from the current workspace or `FLINT_REPO_ROOT`; never assume a machine-specific path.

Generated proto types are in `src/gen/flint/v1/` — regenerate with `pnpm gen:proto` in the flint repo.
