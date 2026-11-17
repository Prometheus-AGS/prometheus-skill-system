---
license: MIT
name: entity-realtime-surreal-live
description: >
  Wire SurrealDB LIVE SELECT subscriptions into a Prometheus entity graph
  via createSurrealLiveAdapter. Covers select-then-live seeding,
  CREATE/UPDATE/DELETE action mapping, reconnect with exponential backoff,
  and optional checkpoint-based replay on reconnect.
metadata:
  version: '1.0.0'
  tags: "entity, realtime, surrealdb, adapter, prometheus"
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
- Optional `checkpointStore` + `checkpointField` to replay missed updates
  after a disconnect.

## Setup

```ts
// app-init.ts
import {
  createSurrealLiveAdapter,
  getRealtimeManager,
  type ChannelConfig,
  type SurrealLike,
  type SurrealTableConfig,
} from "@prometheus-ags/prometheus-entity-management";

// your already-connected Surreal client (see surreal-client.ts in your app)
declare const db: SurrealLike;

const tables: SurrealTableConfig[] = [
  { type: "client",  table: "client" },
  { type: "deal",    table: "deal" },
  { type: "contact", table: "contact" },
];

const adapter = createSurrealLiveAdapter({
  surreal: db,
  tables,
  initialQueryStrategy: "select-then-live",
});

const channels: ChannelConfig[] = tables.map(({ type }) => ({ type }));
getRealtimeManager().register(adapter, channels);
```

That's it — every consumer of `useEntity`, `useEntityList`, etc. now
receives live updates as the underlying rows change.

## Patterns

### Tenant-scoped subscriptions

Scope tenancy at the **connection layer**: give each tenant its own
authenticated Surreal client (namespace / database / credentials) and its own
adapter instance with a distinct `name`, and re-register on tenant change:

```ts
import {
  createSurrealLiveAdapter,
  type SurrealLike,
  type SurrealTableConfig,
} from "@prometheus-ags/prometheus-entity-management";

declare const tenantId: string;
declare const tenantDb: SurrealLike; // client connected with this tenant's credentials
declare const tenantTables: SurrealTableConfig[];

const tenantAdapter = createSurrealLiveAdapter({
  name: `surreal-live:${tenantId}`,
  surreal: tenantDb,
  tables: tenantTables,
});
```

### Row-shape mapping

The adapter pulls entity ids via `idField` (default `"id"`). When the
SurrealDB row shape differs from your entity model (for example snake_case
columns), map changes with the `normalize` argument to
`RealtimeManager.register(adapter, channels, normalize)` rather than inside
components.

### Checkpoint-based replay

For offline-tolerant apps, supply a `checkpointStore`. The adapter persists
the latest `checkpointField` value it has seen (default `"updated_at"`), and
on reconnect runs `SELECT * FROM <table> WHERE <field> > <stored>` per table
before re-attaching the live stream:

```ts
import {
  createSurrealLiveAdapter,
  type SurrealCheckpointStore,
  type SurrealLike,
  type SurrealTableConfig,
} from "@prometheus-ags/prometheus-entity-management";

declare const db: SurrealLike;
declare const tables: SurrealTableConfig[];
declare const localforage: {
  getItem<T>(key: string): Promise<T | null>;
  setItem(key: string, value: string): Promise<unknown>;
};

const checkpointStore: SurrealCheckpointStore = {
  get: (key) => localforage.getItem<string>(key).then((value) => value ?? undefined),
  set: (key, value) => localforage.setItem(key, value).then(() => undefined),
};

const adapter = createSurrealLiveAdapter({
  surreal: db,
  tables,
  checkpointField: "updated_at",
  checkpointStore,
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
- **`affectedListKeys` performance.** List-key lookup is derived per
  ChangeSet emit; supply `listKeyResolver` to override the default mapping.
  Profile in apps with very high update rates; defer optimisation until it
  shows up.

## Tests

Canonical behavior reference:
`packages/entity-graph-core/src/adapters/surreal-live.test.ts`. The
vitest suite uses a hand-rolled fake `Surreal` client (no live database
needed) and covers every action mapping, reconnect, and replay scenario.

Run from the entity-management repo root:

```sh
pnpm test packages/entity-graph-core/src/adapters/surreal-live.test.ts
```
