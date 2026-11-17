---
name: entity-realtime-local-first
description: >
  Wire the full production local-first stack: ElectricSQL shape sync into PGlite via the
  trio pattern (synced/local/view + INSTEAD OF triggers + local_writes queue + write drain),
  createTenantScopedElectricAdapter for tenant validation, createPGlitePersistenceAdapter
  + startLocalFirstGraph for entity graph hydration, the certified optional Loro provider
  path for peer convergence, and Supabase Realtime for Tier-C event push alongside the
  Electric read path. Replaces TanStack Query entirely.
license: MIT
metadata:
  version: '1.0.0'
  tags: "react, typescript, entity-management"
---

# `/entity-realtime-local-first` — Production PGlite Local-First Stack

## When to Use

- Building a multi-tenant SaaS with offline-capable reads and optimistic writes
- Replacing TanStack Query with a local Postgres replica that is always fast and works offline
- Self-hosted Supabase + ElectricSQL deployment (never Supabase Cloud)
- Needing Supabase Realtime for event push / presence alongside Electric for the data plane

## Architecture Overview

```
Server Postgres (self-hosted Supabase)
         │
         │  ElectricSQL shape SSE  (read path — server → client)
         ▼
PGlite *_synced tables  (Electric writes here; app never writes directly)
         │
         │  UNION ALL view
         ▼
PGlite <entity> view  ←── INSTEAD OF triggers ←── app writes view
         │                       │
         │                       ▼
         │               *_local table  (optimistic shadow + is_deleted tombstone)
         │                       │
         │               local_writes queue
         │                       │
         │               pg_notify('local_write')
         │                       │
         │               Write drain  (Supabase REST POST)
         │                       │
         └───────────────────────┘  Electric confirms canonical row back into *_synced
```

The app always reads and writes the **view**. The synced/local split is invisible to
stores and components.

## Tier Model

| Tier | Contract | Local Objects | Write Path |
|---|---|---|---|
| **A** | Synced + writeable offline | `*_synced`, `*_local`, view, INSTEAD OF triggers | local_writes → Supabase REST → Electric confirms |
| **B** | Synced read-only | `*_synced` + alias view | Server only |
| **C** | Server-only | Not in PGlite | Supabase REST on demand; Realtime for live events |

## SYNC_CONFIG — Declarative Entity Registry

All Tier-A/B entities are declared in `sync-config.ts`. This drives both Electric shape
subscriptions and the PGlite SQL generator (`pnpm gen:pglite-schema`).

```typescript
export const SYNC_CONFIG: SyncEntityConfig[] = [
  {
    name: 'client',
    tier: 'A',
    tenantColumn: 'company_id',
  },
  {
    name: 'metadata_type',
    tier: 'A',
    tenantColumn: 'company_id',
    // Two-class shape: company rows + system-wide rows
    shapeWhere: (cid) => `(company_id = ${cid} OR company_id IS NULL)`,
  },
  {
    name: 'service_catalog',
    tier: 'B',
    tenantColumn: null,
    shapeWhere: (cid) => `company_id = ${cid} OR is_global = true`,
  },
];

export const EMIT_ORDER = ['company', 'client', 'metadata_type', 'service_catalog'] as const;
```

**PGlite has no RLS.** Every `shapeWhere` must be a strict subset of the matching server
RLS `USING` clause.

## PGlite Worker Bootstrap

```typescript
// pglite.worker.ts
import { PGlite } from '@electric-sql/pglite';
import { live } from '@electric-sql/pglite/live';
import { electricSync } from '@electric-sql/pglite-sync';
import { worker } from '@electric-sql/pglite/worker';
import localSchema from './local-schema.sql?raw'; // GENERATED — never hand-edit

worker({
  async init() {
    const db = await PGlite.create('idb://my-app-db', {
      extensions: { live, electric: electricSync() },
    });
    await db.exec(localSchema); // idempotent: all DDL uses IF NOT EXISTS
    return db;
  },
});
```

```typescript
// pglite-client.ts — main thread singleton
import { PGliteWorker } from '@electric-sql/pglite/worker';
import { live } from '@electric-sql/pglite/live';
import { electricSync } from '@electric-sql/pglite-sync';

const EXTENSIONS = { live, electric: electricSync() };
export type LocalDB = PGliteWorker & PGliteInterfaceExtensions<typeof EXTENSIONS>;

// Must match the timestamp stamped into local-schema.sql by the generator
export const BUNDLED_PGLITE_SCHEMA_VERSION = '20260523000018';

let bootPromise: Promise<LocalDB> | null = null;

export function getLocalDB(): Promise<LocalDB> {
  if (!bootPromise) {
    bootPromise = (async () => {
      const pgWorker = new Worker(
        new URL('./pglite.worker.ts', import.meta.url),
        { type: 'module', name: 'pglite' },
      );
      const db = (await PGliteWorker.create(pgWorker, { extensions: EXTENSIONS })) as LocalDB;

      // Boot-time schema migration check
      const { rows } = await db.query<{ version: string }>(
        'SELECT version FROM _pglite_schema_version WHERE id = 1',
      );
      if (rows[0]?.version && rows[0].version !== BUNDLED_PGLITE_SCHEMA_VERSION) {
        // Drop all Tier-A data; Electric will re-hydrate from server
        await db.exec('TRUNCATE client_synced, client_local, local_writes RESTART IDENTITY');
        await db.query('UPDATE _pglite_schema_version SET version = $1 WHERE id = 1',
          [BUNDLED_PGLITE_SCHEMA_VERSION]);
        await db.exec('UPDATE _sync_meta SET tenant_id = NULL, hydrated_at = NULL WHERE id = 1');
      }
      return db;
    })();
  }
  return bootPromise;
}
```

## Electric Sync — createTenantScopedElectricAdapter

The adapter is a **validation gate** that ensures every shape has a tenant predicate.
Actual sync happens via `syncShapeToTable` inside the factory:

```typescript
import {
  createTenantScopedElectricAdapter,
  type TenantScopedTableConfig,
} from '@prometheus-ags/prometheus-entity-management';

export async function startTenantSync(companyId: string, awaitInitialSync: boolean) {
  const db = await getLocalDB();
  const subs: Array<{ unsubscribe: () => void }> = [];
  const initialPromises: Array<Promise<void>> = [];

  const tables: TenantScopedTableConfig[] = SYNC_CONFIG.map((config) => {
    let resolveInitial = () => {};
    const p = new Promise<void>((r) => { resolveInitial = r; });
    initialPromises.push(p);

    return {
      type: config.name,
      table: config.name,
      tenantColumn: config.tenantColumn,
      primaryKey: config.primaryKey ?? ['id'],
      shapeStreamFactory: ({ table, where }) => {
        const finalWhere = config.shapeWhere
          ? config.shapeWhere(`'${companyId}'`)
          : where;

        void db.electric.syncShapeToTable({
          shape: { url: `${ELECTRIC_URL}/v1/shape`, params: { table, where: finalWhere } },
          table: `${table}_synced`,
          primaryKey: config.primaryKey ?? ['id'],
          shapeKey: `${table}:${companyId}`,   // stable key = offset resumption
          onInitialSync: resolveInitial,
        }).then((sub) => {
          if (sub.isUpToDate) resolveInitial(); // fast path: resume case
          subs.push({ unsubscribe: () => sub.unsubscribe() });
        });

        return { subscribe: () => () => {}, isUpToDate: false, lastOffset: '' };
      },
    };
  });

  // Validation gate — throws if any shape lacks scope or companyId is not a UUID
  createTenantScopedElectricAdapter({ pglite: db, tenantClaim: { companyId }, tables, onSynced: () => {} });

  if (awaitInitialSync) {
    await Promise.all(initialPromises); // wait for full hydration on first login
  }

  return { unsubscribe: async () => { for (const s of subs) s.unsubscribe(); } };
}
```

## Write Drain — local_writes → Supabase REST

```typescript
export async function startWriteSync() {
  const db = await getLocalDB();

  const drain = async () => {
    const { rows } = await db.query<PendingWrite>(
      `SELECT id, entity, operation, row_id, payload
         FROM local_writes WHERE synced_at IS NULL
         ORDER BY created_at ASC`,
    );
    for (const w of rows) {
      try {
        if (w.operation === 'delete') {
          const { error } = await supabase.from(w.entity).delete().eq('id', w.row_id);
          if (error) throw error;
        } else {
          const { error } = await supabase
            .from(w.entity).upsert(w.payload ?? {}, { onConflict: 'id' });
          if (error) throw error;
        }
        await db.query('UPDATE local_writes SET synced_at = now() WHERE id = $1', [w.id]);
        if (w.operation !== 'delete') {
          // Clear optimistic shadow; Electric streams canonical row back into *_synced
          await db.query(`DELETE FROM ${w.entity}_local WHERE id = $1`, [w.row_id]);
        }
      } catch { break; } // retry on next NOTIFY
    }
  };

  const unlisten = await db.listen('local_write', () => void drain());
  void drain(); // flush backlog from before this session
  return async () => { await unlisten(); };
}
```

## Entity Graph Bootstrap — createPGlitePersistenceAdapter

```typescript
import {
  createPGlitePersistenceAdapter,
  startLocalFirstGraph,
} from '@prometheus-ags/prometheus-entity-management';

export async function bootstrapEntityGraph(pglite: LocalDB, companyId: string) {
  const storage = await createPGlitePersistenceAdapter(pglite);

  return startLocalFirstGraph({
    storage,
    key: `tenant:${companyId}`,
    replayPendingActions: true,
    onlineSource: {
      getIsOnline: () => navigator.onLine,
      subscribe: (listener) => {
        const on = () => listener(true);
        const off = () => listener(false);
        window.addEventListener('online', on);
        window.addEventListener('offline', off);
        return () => { window.removeEventListener('online', on); window.removeEventListener('offline', off); };
      },
    },
  });
}
```

## Optional peer collaboration — entity-graph-sync + Loro

PGlite snapshot durability and Electric server replication do not prove
peer-to-peer conflict resolution. Add the companion sync package only when
multiple offline replicas must reconcile entity fields:

```typescript
import { createGraphStore } from '@prometheus-ags/entity-graph-core';
import {
  createLoroProvider,
  createSyncProviderRegistry,
  createWebSocketLoroChannel,
  startSyncBridge,
} from '@prometheus-ags/entity-graph-sync';

export async function startPeerSync(url: string, peerId: number) {
  const store = createGraphStore();
  const registry = createSyncProviderRegistry();
  const channel = createWebSocketLoroChannel(url, {
    onError: (error) => reportSyncError(error),
  });
  registry.register({
    entityTypes: ['Task', 'Comment'],
    provider: createLoroProvider({
      channel,
      peerId,
      registerMergeStrategies: false,
      onError: (error, operation) => reportSyncError(error, operation),
    }),
  });
  const bridge = await startSyncBridge({ store, registry });
  return { store, bridge };
}
```

Use one unique numeric `peerId` per live replica. The bridge suppresses inbound
peer projections from its outbound subscription. The WebSocket channel queues
the latest offline write per entity type and requests current peer snapshots
after reconnect. A relay broadcasts opaque binary frames to other clients; it
does not own canonical entities or understand Loro.

Do not add peer sync if Electric's server-confirmed replication is the only
required synchronization model. Do not claim peer convergence from a PGlite
reload test, or persistence from an in-memory convergence test.

## Supabase Realtime — Tier-C Only

```typescript
import { createSupabaseRealtimeAdapter, getRealtimeManager } from '@prometheus-ags/prometheus-entity-management';

// Only for tables NOT in SYNC_CONFIG (Tier-C server-only tables)
const realtimeAdapter = createSupabaseRealtimeAdapter(supabase, {
  tableTypeMap: { notification: 'Notification', hsh_review: 'HshReview' },
  extractId: (r) => String(r.id),
});

const unsub = getRealtimeManager().register(realtimeAdapter, [
  { type: 'Notification', filter: { company_id: companyId } },
  { type: 'HshReview',    filter: { company_id: companyId } },
]);
// On logout: unsub()
```

**Never subscribe Supabase Realtime to Tier-A/B tables** — Electric already covers those.

## Full Tenant Lifecycle

```typescript
// Sign-in
const db = await getLocalDB();
const meta = await getSyncMeta();
const syncHandle = await startTenantSync(companyId, /* awaitInitial */ !meta.hydratedAt);
const stopDrain = await startWriteSync();
const runtime = await bootstrapEntityGraph(db, companyId);
if (!meta.hydratedAt) await markHydrated(companyId);

// Sign-out / tenant switch
runtime.dispose();
await stopDrain();
await syncHandle.unsubscribe();
await clearLocalTenantData(); // TRUNCATE + reset _sync_meta
bootPromise = null;           // reset singleton
```

## Architectural Rules

- PGlite has no RLS — every `shapeWhere` must be ⊆ its RLS USING clause
- Never write to `*_synced` tables — Electric's exclusive write surface
- Never subscribe Realtime to Tier-A/B tables — Electric covers those
- Self-hosted Supabase only — enforce with `*.supabase.co` URL guard at module load
- `BUNDLED_PGLITE_SCHEMA_VERSION` must always match the timestamp in `local-schema.sql`
- Never hand-edit `local-schema.sql` — regenerate with `pnpm gen:pglite-schema`
- Keep peer `GraphStore` and `SyncProviderRegistry` ownership aligned; inject
  both for isolated clients, workers, and SSR/request graphs
- Install `loro-crdt >=1.13.9 <2` when selecting the Loro provider

## Pitfalls

| Issue | Mitigation |
|---|---|
| Full shape re-download on every reload | Set `shapeKey: "${table}:${companyId}"` |
| `onInitialSync` never fires on return visits | Check `sub.isUpToDate` immediately after attach; call resolver in both paths |
| FK violations on initial sync | Use `syncShapesToTables` (multi-table transactional) |
| Duplicate graph writes | Remove Realtime subscriptions from Tier-A/B tables |
| Peer updates bounce forever | Use `startSyncBridge`; do not bypass its inbound echo-suppression boundary |
| Same-field conflicts vary in tests | Assign distinct deterministic numeric Loro peer IDs |
| Reconnect misses remote work | Use the sync-request-aware WebSocket channel and an opaque binary relay |
| Schema version mismatch after deploy | Sync `BUNDLED_PGLITE_SCHEMA_VERSION` with SQL stamp |

## Cross-Reference

`prometheus-skill-pack/skills/typescript/pglite/references/LOCAL_FIRST_ARCHITECTURE.md` —
full tier model, trio pattern SQL, write drain, schema versioning details.

`prometheus-skill-pack/skills/typescript/pglite/references/SUPABASE_REALTIME.md` —
Realtime coexistence rules, presence, broadcast, Tier-C patterns.

`prometheus-skill-pack/skills/typescript/pglite/references/SYNC_CONFIG_SCHEMA_GEN.md` —
SYNC_CONFIG declaration, EMIT_ORDER, gen-pglite-schema, EntityName type safety.

`prometheus-entity-skills/_shared/references/sync-persistence-path.md` —
certified PGlite/Loro boundaries, public sync APIs, conflict policy, exclusions,
and mandatory verification commands.
