# Certified PGlite and Loro sync path

This reference covers the stable headless path owned jointly by:

- `@prometheus-ags/entity-graph-core` for graph snapshots and PGlite persistence; and
- `@prometheus-ags/entity-graph-sync` for provider registries, graph bridging, Loro reconciliation, and WebSocket/loopback channels.

It does not certify a rendered example, mobile device, remote production relay, ElectricSQL server, or the sibling `prometheus-entity-sync` repository.

## Mental model

PGlite is durable storage for one client's graph snapshot. Loro reconciles edits produced by different clients. The graph bridge projects the reconciled result into canonical entity reads. The channel carries snapshots and must recover work missed while disconnected. Passing one layer does not prove the other three.

## Public runtime surface

The machine ledger is `sync-library-exports.json`. Important entry points are:

| Export | Contract |
| --- | --- |
| `createSyncProviderRegistry()` | Creates client-owned provider registration state; use one per isolated graph client. |
| `startSyncBridge({ store, registry, pushDebounceMs })` | Connects an injected `GraphStore` and registry. Inbound peer projections are suppressed from outbound echo. |
| `applyPeerChanges(changes, store?)` | Applies peer fields and sync metadata to the selected store. |
| `createLoroProvider({ channel, peerId, loadLoro, onError, registerMergeStrategies })` | Creates one Loro document per managed type. `loadLoro` gives browser bundlers a visible optional-peer import; deterministic numeric peer IDs make same-field conflict tests reproducible. |
| `createLoroLoopbackNetwork({ autoFlush })` | Deterministic offline/reconnect reference fabric with FIFO/reverse delivery control. |
| `createWebSocketLoroChannel(url, options)` | Queues the latest disconnected writes, reconnects, requests missed peer snapshots, and exposes transport status. |
| `encodeLoroWebSocketMessage` / `decodeLoroWebSocketMessage` | Stable opaque relay frame helpers. A one-byte zero control frame is reserved for sync requests. |

Public types include `SyncProviderRegistry`, `SyncBridgeOptions`, `LoroChannel`, `LoroChannelStatus`, `LoroProviderOptions`, `LoroLoopbackNetwork`, and `LoroWebSocketChannelOptions`.

## Required architecture

```text
component → hook → store/adapter → persistence or relay
                    │
                    ├─ GraphStore (canonical entities; ID-only lists)
                    ├─ SyncProviderRegistry (client-owned provider mapping)
                    └─ LoroProvider (CRDT state; never a second UI store)
```

- Inject both `store` and `registry` when hosting multiple clients, workers, or SSR/request-isolated graphs.
- Keep local UI patches separate from canonical entity data. Persistence intentionally retains patches in the current version; peer synchronization sends canonical entity fields only.
- Use a distinct numeric Loro peer ID for each live replica. For concurrent same-field writes at equal logical counters, Loro's LWW map deterministically selects the higher peer ID.
- Do not re-publish inbound peer graph writes as local changes. The bridge's origin-suppression boundary owns this invariant.
- A WebSocket relay only broadcasts opaque binary frames to other clients. It need not understand Loro, but it must preserve binary payloads and allow the sync-request frame to reach peers.

## Dependency contract

- Mandatory release tests pin `@electric-sql/pglite` `0.5.4` and `loro-crdt` `1.13.9`.
- Sync consumers selecting Loro must install `loro-crdt` in the supported range `>=1.13.9 <2`.
- Vite/browser consumers should pass `loadLoro: () => import("loro-crdt")`; Node consumers may omit it. The provider passes the same loader to core merge-strategy registration.
- Yjs peers remain optional and secondary to this certified Loro path.
- The sibling `prometheus-entity-sync` integration stays explicit opt-in; never add an absolute `link:` or developer-local path.

## Verification

```bash
pnpm run test:sync-persistence
pnpm run verify:sync-persistence
pnpm run bdd:sync-persistence
pnpm --filter @prometheus-ags/entity-graph-sync test:websocket-integration
pnpm --filter @prometheus-ags/entity-graph-sync run verify:skills
```

These commands prove real PGlite close/reopen, two isolated Loro clients, both delivery orders, different-field preservation, same-field deterministic conflicts, echo suppression, real socket termination/reconnect, packed ESM/CommonJS runtime, and NodeNext declarations. Mandatory receipts contain no conditional skip.

The machine coverage source is `examples/coverage.json`, capability `graph.offline-persistence-sync`, quality gate `release.sync.persistence-convergence`. Overall coverage remains `in-progress`: React/Vite has browser PGlite/Loro evidence; the implemented Flutter showcase has in-memory offline/reconnect plus Android/iOS smoke evidence but makes no durable-persistence claim; Tauri has browser queue/reload/reconnect evidence plus a packaged macOS native SQLite process-restart receipt, while mobile durable restart is not claimed; Docusaurus remains planned.
