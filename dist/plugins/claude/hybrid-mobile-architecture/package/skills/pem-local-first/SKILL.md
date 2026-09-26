---
name: pem-local-first
description: ALWAYS invoke when wiring client entity/server state on any surface — adding an entity type, registering a transport, persisting conversations, hydrating from PGlite/pglite-oxide/SQLite, showing optimistic writes, or whenever TanStack Query/SWR/Apollo-cache is about to be suggested (it is prohibited — Prometheus Entity Management 3.x owns this layer). Triggers on entity, entities, PEM, prometheus-entity-management, entity transport, registerEntityTransport, useEntities, entity graph, TanStack Query, react-query, SWR, query cache, stale-while-revalidate, optimistic update, PGlite, pglite-oxide, persistence adapter, durable conversation, conversation store, hydrate, normalized state.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Decisions: ADR-LFS-2/3/4 in
> [`decisions.md`](references/decisions.md). Layer contract:
> [`patterns.md`](references/tauri-patterns.md). The invariants below
> remain authoritative when the optional references are unavailable.

# PEM Local-First — the entity layer

## Why not TanStack Query (the layering argument, once)

Query caches model **requests** ("is this response stale?"). Local-first apps
model **data**: a synced local store already knows freshness — the sync engine
owns staleness, dedup, and revalidation. A query cache on top re-answers that
question in the wrong layer, with a second source of truth that drifts. PEM's
normalized entity graph sits AT the data layer, over the local store. This is
enforced: `audit.mjs` fails a scaffolded project that depends on TanStack Query.

## The stack (per surface)

```
React:   Component → Hook (useFeature) → PEM hooks/Zustand → transports → local store
Flutter: Widget → @riverpod provider → prometheus_entity_management (Dart) → FFI → Rust store
```

- Local store per tier: PGlite (web `idb://`), pglite-oxide via typed Tauri
  commands (desktop), SQLite via FFI (mobile). Embedded-engine singleton
  lifecycle rules apply (`references/rust-patterns.md`).
- Zustand keeps ONLY transient interaction state (selection, filters, stream
  buffers). Anything durable is a PEM entity.

## Wiring recipe (React surface)

1. **Schema first**: table with client-UUID `id`, `tenant_id`, `updated_at`
   (+ privacy class declared — see `sync-doctrine` skill).
2. **Transport**: `registerEntityTransport(entityType, transport)` where the
   transport reads the LOCAL store (PGlite query or plugin `entityList`) —
   transports never open sockets and never call remote APIs directly
   (ADR-LFS-2). Honor the PEM `ListQuery` (filters/sorts/limit/cursor); do not
   fetch-everything-and-ignore-the-query.
3. **Persistence**: `createPGlitePersistenceAdapter` + `startLocalFirstGraph`
   once at the runtime boundary (one place per app, reference-counted).
4. **Consume**: features import hooks (`useEntities`, `useEntityMutation`)
   via their feature `hooks/` layer — components never import stores or call
   `invoke()`.

## Optimism and the queue (one queue, not two)

Mutations write the local store AND enqueue into the sync `_operation_queue`.
PEM's pending-action state reflects the queue's row state
(`PENDING/IN_FLIGHT/…`); on `FATAL_ERROR → DEAD_LETTER` the mutation rolls
back visibly (toast + SyncChip detail). Never maintain a separate replay queue
in JS — the queue in the store is the truth (doctrine §write queue).

## Sync-fed reactivity

Scope streams and lookup-bundle refreshes land in local tables; PEM transports
re-emit through the graph, so UI updates without feature code touching sync.
When adding a synced entity: define the scope (tenant predicate!), register
the transport, and the rest is doctrine machinery. Lookup tables are read-only
entities — never expose mutations for them.

## Conversations (the canonical durable example)

Threads/messages/blocks are PEM entities persisted in PGlite (web) or
pglite-oxide (desktop). Assistant UI renders; streaming deltas live in Zustand
until the block finalizes, then the entity write commits (and embeds — see
`client-rag` skill). Mobile mirrors via the Dart PEM package over FFI.

## Checklist

- [ ] No TanStack Query/DB/SWR/Apollo-cache anywhere (audit-enforced)
- [ ] Transport reads local store only; honors ListQuery; tenant-scoped
- [ ] `startLocalFirstGraph`/persistence adapter wired ONCE at runtime boundary
- [ ] Durable data = PEM entity; Zustand = transient only
- [ ] Mutations visible in queue state; DEAD_LETTER surfaces to the user
- [ ] Boundary tests: transport contract against a real local store (no mocks)
