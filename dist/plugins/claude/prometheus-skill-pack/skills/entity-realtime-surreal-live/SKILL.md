---
license: MIT
name: entity-realtime-surreal-live
version: '1.0.0'
description: >
  Wire SurrealDB LIVE SELECT subscriptions into a Prometheus entity graph
  via createSurrealLiveAdapter. Covers select-then-live seeding,
  CREATE/UPDATE/DELETE action mapping, reconnect with exponential backoff,
  and optional checkpoint-based replay on reconnect.
metadata:
  tags: [entity, realtime, surrealdb, adapter, prometheus]
---

# entity-realtime-surreal-live

Hook up SurrealDB's live-query stream to the Prometheus entity graph.

## When to use

- Your app stores entities in SurrealDB (or a SurrealDB-compatible backend
  reachable via `surrealdb.js` / `surrealdb`).
- You want the entity graph (`useEntity`, `useEntityList`,
  `useGQLEntity`) to update in real time as rows change server-side, no
  per-component subscription wiring.
- You want a documented reconnect + replay story so transient websocket
  outages don't lose updates.

If you're on ElectricSQL + PGlite, see `entity-realtime-channel` /
`entity-realtime-local-first` instead.

## What you get

- One `LIVE SELECT` per registered table, opened at app startup and
  re-opened automatically after reconnects.
- Initial bulk seed (`select-then-live` mode, default) so the graph is
  populated before the first render — or skip the seed entirely with
  `live-only` mode.
- Action → `EntityChange` mapping handled inside the adapter; each
  emitted `ChangeSet` carries `affectedListKeys` so derived lists refresh.
- Optional `checkpointResume` to replay missed updates after a
  disconnect.

## Setup

```ts
// app-init.ts
import {
  createSurrealLiveAdapter,
  registerAdapter,
  type SurrealTableConfig,
} from "@prometheus-ags/prometheus-entity-management";
import { db } from "./surreal-client"; // your already-connected Surreal client

const tables: SurrealTableConfig<Record<string, unknown>>[] = [
  { type: "client",  table: "client", where: "active = true" },
  { type: "deal",    table: "deal" },
  { type: "contact", table: "contact",
    normalize: (row) => ({
      ...row,
      // map server snake_case to camelCase for the graph:
      displayName: row["display_name"],
      createdAt:   row["created_at"],
    }),
  },
];

const adapter = createSurrealLiveAdapter({
  db,
  tables,
  initialQueryStrategy: "select-then-live",
  onSynced: () => console.log("[surreal-live] initial seed complete"),
});

registerAdapter("surreal-live", adapter);
```

That's it — every consumer of `useEntity`, `useEntityList`, etc. now
receives live updates as the underlying rows change.

## Patterns

### Tenant-scoped subscriptions

Put your tenant filter in `where`:

```ts
{ type: "deal", table: "deal", where: `company_id = "${tenantId}"` }
```

(For multi-tenant apps that must switch tenants at runtime, prefer a
re-register on tenant change.)

### Schema normalisation

Use `normalize` to map server columns to graph properties — useful when
the SurrealDB schema differs from your entity model.

### Checkpoint-based replay

For offline-tolerant apps, supply a `checkpointResume` block. The adapter
persists the latest `updated_at` it has seen (per change), and on reconnect
runs `SELECT * FROM <table> WHERE updated_at > <last>` for each table
before re-attaching the live stream:

```ts
const adapter = createSurrealLiveAdapter({
  db,
  tables,
  checkpointResume: {
    columnName: "updated_at",
    loadCheckpoint: () => localforage.getItem<string>("surreal:lastSync"),
    saveCheckpoint: (offset) => localforage.setItem("surreal:lastSync", offset),
  },
});
```

## Gotchas

- **SurrealDB CREATE vs UPDATE.** The adapter maps `UPDATE` actions to
  `op: "upsert"` (not `"update"`), because SurrealDB doesn't distinguish
  partial vs full updates on the wire. The engine handles upsert
  semantics correctly.
- **DELETE payloads.** Sometimes carry only `{id}`, sometimes the full
  prior row. The adapter pulls `id` defensively.
- **WebSocket auth.** If your SurrealDB client refreshes auth tokens
  periodically, ensure the refresh happens before the websocket closes;
  the adapter's reconnect loop will re-authenticate via your client but
  cannot recover an expired token mid-flight.
- **Reconnect backoff.** 1s → 3s → 9s → 30s cap. The loop runs
  forever — that's intentional; document for ops.
- **`affectedListKeys` performance.** Lookup runs once per ChangeSet
  emit. Profile in apps with very high update rates; defer optimisation
  until it shows up.

## Tests

Canonical behavior reference:
`prometheus-entity-management/src/adapters/surreal-live.test.ts`. The
vitest suite uses a hand-rolled fake `Surreal` client (no live database
needed) and covers every action mapping, reconnect, and replay scenario.

Run from the entity-management repo root:

```sh
pnpm test src/adapters/surreal-live.test.ts
```
