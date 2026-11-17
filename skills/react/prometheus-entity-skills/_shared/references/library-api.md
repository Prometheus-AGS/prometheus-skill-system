# @prometheus-ags/prometheus-entity-management — public API reference

Generated from `src/index.ts`. Use this when scaffolding imports or explaining capabilities. Types are TypeScript-only unless noted.

**Runtime exports ledger:** Agent skills that must match shipped code should align with `library-exports.json` (entry-keyed export names from the built ESM bundles). After changing any React public entry, run `pnpm run refresh:exports` at the repo root and commit `prometheus-entity-skills/_shared/references/library-exports.json`. CI runs `pnpm run verify:skills`.

The React ledger is entry-point keyed: `.` is the ordinary package root,
`./devtools` is the side-effect-free inspector API, and `./devtools/auto` is
the explicit development bootstrap. React inspector activation, UI semantics,
value policy, and evidence limits are documented in
[`devtools-react-inspector.md`](devtools-react-inspector.md).

**3.0 release surface:** Packaging, compatibility, protocol maturity, publication, and rollback claims come from [`v3-release-contract.md`](v3-release-contract.md), which links to the authoritative machine-readable contract. These release concerns are intentionally separate from the runtime export ledger.

**CI and dependency policy:** The implemented main-CI quality gate and its dependency/advisory policies are linked from `v3-release-contract.md`. CI-policy changes do not alter this API reference or `library-exports.json` unless a public package entry point changes.

**Packed package contract:** Loader, declaration, metadata, and tarball-payload claims are verified by `release/package-contracts.md` and `pnpm run verify:package-contracts`. These artifact rules remain separate from the runtime export-name ledger; update `library-exports.json` only when the built export set changes.

**Framework boundary:** Core/React store claims are verified by `release/framework-neutral-core.md` and `pnpm run verify:framework-neutral-core`. Cross-binding package resolution and behavior are separately verified by `release/binding-singleton-contract.md` and `pnpm run verify:binding-singletons`. Core exports vanilla stores and imperative readers; React owns callable subscription hooks. The application installs one required compatible core peer for each binding.

**Companion A2UI boundary:** `@prometheus-ags/a2ui-react` has its own two-entry-point [`a2ui-library-exports.json`](a2ui-library-exports.json) ledger and [`a2ui-protocol-bridge.md`](a2ui-protocol-bridge.md) agent reference. Its root is official A2UI v0.9.1; `./ag-ui` is the alpha chat/state compatibility surface. Protocol validity never grants application authority.

**Companion A2A boundary:** `@prometheus-ags/entity-graph-a2a` has its own two-entry-point [`a2a-library-exports.json`](a2a-library-exports.json) ledger and [`a2a-conformance-agent.md`](a2a-conformance-agent.md) reference. Its root implements official A2A v1 JSON-RPC; `./legacy` is only the pre-v3 slash-method migration surface. Protocol validity never grants application authority.

**Core DevTools boundary:** `@prometheus-ags/entity-graph-core/devtools` is an
optional, versioned entry point tracked under `./devtools` in
[`core-library-exports.json`](core-library-exports.json). Its entity/view/
relationship projections, local preview/restore rules, value policy, shared
fixture, and evidence limits are documented in
[`devtools-entity-inspection.md`](devtools-entity-inspection.md). These core
contracts do not claim that a React inspector, Flutter controller, browser
extension, or Flutter DevTools extension is complete.

**React DevTools boundary:**
`@prometheus-ags/prometheus-entity-management/devtools` exposes the explicit
SSR-safe host/provider/hooks and lazy inspector;
`@prometheus-ags/prometheus-entity-management/devtools/auto` is the only
side-effectful automatic browser mount. The ordinary root remains inspector-
free. See [`devtools-react-inspector.md`](devtools-react-inspector.md).

---

## Core (`graph.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `createGraphStore` | function | Create an isolated vanilla graph StoreApi for an SSR request, test, worker, or other non-shared host. |
| `graphStore` | store | Default process-wide vanilla graph singleton used by bindings and imperative consumers. |
| `GraphStoreProvider` | React provider | Scope all descendant React hooks to an application-owned `GraphStore`; required for per-request SSR hydration instead of process-global request state. |
| `useGraphStoreApi` | React hook | Resolve the nearest provider-owned `GraphStore`, falling back to `graphStore`; hook ownership releases scoped listeners and GC after the final unmount. |
| `useGraphStore` | React hook/store | React-package hook over the nearest scoped graph, with the default singleton's StoreApi methods attached for compatibility. Core's same-named export is only a deprecated StoreApi alias; never call it as a hook. |
| `GraphStoreProviderProps` | type | Provider input containing the application-owned `store` and React `children`. |
| `GraphStore` | type | Vanilla StoreApi returned by `createGraphStore()`. |
| `GraphState` | type | Full store shape: canonical entities, UI patches, per-entity fetch state, list slots keyed by query key, and the atomic `ingestFetchedList` action. |
| `EntityState` | type | Per-entity cache metadata: `isFetching`, `lastFetched`, `error`, `stale`. |
| `EntitySyncMetadata` | type | Optional sync/provenance metadata stored beside canonical rows (`synced`, `origin`, `updatedAt`). |
| `EntitySnapshot` | type | Sync-aware entity view returned by snapshot reads and graph query helpers (`$synced`, `$origin`, `$updatedAt`). |
| `ListState` | type | List slot: ordered **`ids`** only (not row copies), pagination cursors, `total`, fetch flags, page metadata. |
| `ListResultMeta` | type | Successful-list pagination and lifecycle metadata accepted by atomic list targets. |
| `FetchedListTarget` | type | Replace/append list target applied with a fetched response; may carry transport-derived IDs and metadata. |
| `FetchedEntityBatch` | type | Side-descriptor entity rows that commit or roll back with the primary fetched list. |
| `FetchedListProjection` | type | Existing view-backed list updated with matching fetched IDs inside the same graph publication. |
| `IngestFetchedListOptions` | type | Atomic ingestion options for list targets, side batches, view projections, and fetch completion. |
| `EntityType` | type | String key partitioning the graph (e.g. `"Post"`). |
| `EntityId` | type | Primary key string for an entity within its `EntityType`. |
| `QueryKey` | type | Stable string key used for list slots. |
| `SyncOrigin` | type | Provenance enum for sync metadata: `server`, `client`, or `optimistic`. |

---

## Stable JavaScript binding singleton surface

The runtime export ledger above is generated from the React package. The other stable JavaScript entry points expose these binding-level graph adapters; their package manifests keep core application-owned through a required peer.

| Package | Primary reactive surface | Shared-core evidence |
| --- | --- | --- |
| `@prometheus-ags/prometheus-entity-management` | `useEntity`, `useEntityList`, callable React `useGraphStore` | React hook/store observes `graphStore` writes |
| `@prometheus-ags/entity-graph-svelte` | `createEntityStore`, `createEntityList`, `initEntityGraph` | Svelte store observes the application core |
| `@prometheus-ags/entity-graph-solid` | `createEntity`, `createEntityList`, Solid `createGraphStore`, vanilla `graphStore` | `graphStore` identity matches core; deprecated core `useGraphStore` remains an alias |
| `@prometheus-ags/entity-graph-web-components` | entity controllers and registered custom elements | Lit controller observes the application core |
| `@prometheus-ags/entity-graph-alpine` | plugin, `$entity`/`$entityList`, binding factories | Alpine binding observes the application core |
| `@prometheus-ags/entity-graph-htmx` | SSE server, `createServerGraph`, fragment helpers | server graph and core observe two-way writes |

Use [`release/binding-singleton-contract.md`](../../../release/binding-singleton-contract.md) for the packed resolution contract and its explicit native/browser/publication limits.

---

## Graph runtime extensions

| Export | Kind | Description |
|--------|------|-------------|
| `queryOnce` | function | One-shot graph query helper: single row or list snapshot with local filter/sort/include support. |
| `selectGraph` | function | Alias for `queryOnce` when using the API as a projection helper. |
| `createGraphTransaction` | function | Explicit optimistic graph write transaction with rollback/commit semantics. |
| `createGraphAction` | function | Wraps optimistic transaction + async execution into one reusable mutation action. |
| `createGraphEffect` | function | Lightweight enter/update/exit workflow helper driven by graph query results. |
| `createGraphTool` | function | Small AI-facing helper that injects graph query/export context into a typed function. |
| `createSchemaGraphTool` | function | Schema-aware graph tool helper that injects schema lookup and schema-aware export helpers. |
| `exportGraphSnapshot` | function | Serialize graph data for prompt/context building or external workflows. |
| `GraphActionRecord` | type | Serializable pending action record (`id`, `key`, `input`, `enqueuedAt`). |
| `GraphActionEvent` | type | Event union emitted when graph actions are enqueued or settled. |
| `GraphQueryOptions` | type | Options for `queryOnce`: `type`, ids/list key, local predicates, includes, and projections. |
| `GraphIncludeMap` | type | Nested include/projection map for graph snapshot expansion. |
| `GraphIncludeRelation` | type | Relation resolver for snapshot includes (`field`, `array`, `list`, `resolver`). |
| `GraphTransaction` | type | Transaction surface for graph write orchestration. |
| `GraphActionOptions` | type | Async action config wrapping optimistic writes, success, and rollback behavior. |
| `GraphEffectHandle` | type | Disposable handle returned by `createGraphEffect`. |
| `GraphEffectOptions` | type | Enter/update/exit effect configuration over graph query results. |
| `GraphEffectEvent` | type | Update payload for graph effects. |
| `GraphSnapshotExportOptions` | type | Serialization options for `exportGraphSnapshot`. |
| `GraphToolContext` | type | Helper context injected into `createGraphTool` handlers. |
| `SchemaGraphToolContext` | type | Extended tool context with schema lookup and schema-aware export functions. |

---

## Schema-driven entities and markdown

| Export | Kind | Description |
|--------|------|-------------|
| `registerEntityJsonSchema` | function | Register a JSON Schema for an entity type and optional JSON column / schema id. |
| `registerRuntimeSchema` | function | Register or replace a runtime-generated schema (for AI/A2UI or other dynamic metadata). |
| `getEntityJsonSchema` | function | Resolve the active schema by `entityType`, `field`, or `schemaId`. |
| `buildEntityFieldsFromSchema` | function | Generate entity field descriptors from JSON Schema for dynamic views and editors. |
| `useSchemaEntityFields` | hook | Resolve registered schema config and memoize generated field descriptors. |
| `exportGraphSnapshotWithSchemas` | function | Serialize graph data with schema metadata for AI or external consumers. |
| `MarkdownFieldRenderer` | component | Built-in safe markdown renderer for schema-driven or manual field descriptors. |
| `MarkdownFieldEditor` | component | Built-in markdown editor with preview for schema-driven fields. |
| `renderMarkdownToHtml` | function | Escapes and renders a constrained markdown subset to HTML. |
| `JsonSchemaObject` | type | Lightweight JSON Schema shape used by the schema registry and generators. |
| `EntityJsonSchemaConfig` | type | Registration shape for static/runtime schemas. |
| `SchemaFieldDescriptor` | type | Schema-derived field descriptor compatible with entity sheets. |
| `BuildEntityFieldsFromSchemaOptions` | type | Input options for field generation (`schema`, optional `rootField`). |
| `GraphSnapshotWithSchemasOptions` | type | Serialization options for graph export with schemas. |

---

## Local-first runtime

| Export | Kind | Description |
|--------|------|-------------|
| `startLocalFirstGraph` | function | Starts graph hydration, persistence, pending action replay, and sync-status tracking over a host storage adapter. |
| `hydrateGraphFromStorage` | function | Restore a persisted graph snapshot into the Zustand graph. |
| `persistGraphToStorage` | function | Persist the current graph snapshot plus pending action metadata. |
| `useGraphSyncStatus` | hook | Read serializable sync status for browser/PWA/Tauri-style hosts. |
| `getGraphSyncStatus` | function | Imperatively read sync status without React. |
| `graphSyncStatusStore` | store | Framework-neutral vanilla sync-status StoreApi subscribed to by the React hook. |
| `GraphPersistenceAdapter` | type | Storage adapter contract: `get`, `set`, optional `remove`. |
| `PersistedGraphActionRecord` | type | Re-exported persisted action metadata shape used by the local-first runtime. |
| `GraphSyncStatus` | type | Status payload: phase, online state, sync state, pending count, hydration/persistence timestamps, error. |
| `GraphSnapshotPayload` | type | Persisted JSON payload containing graph snapshot and pending actions. |
| `StartLocalFirstGraphOptions` | type | Runtime bootstrap options for storage, replay, and online-source integration. |
| `LocalFirstGraphRuntime` | type | Returned runtime handle with `ready`, `persistNow`, `hydrate`, `dispose`, and `getStatus`. |

## Merge strategies

| Export | Kind | Description |
| --- | --- | --- |
| `createLoroMergeStrategy(loadLoro?)` | async function | Creates the optional Loro-backed entity merge strategy. Browser bundlers should supply `() => import("loro-crdt")`; Node consumers may use the runtime peer import. |
| `LoroModuleLoader` | type | Bundler-visible optional-peer loader returning a module with `LoroDoc`; it keeps `loro-crdt` out of the mandatory core bundle. |

---

## Engine (`engine.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `configureEngine` | function | App-wide engine options: base fetch, default `staleTime`, retry behavior, etc. |
| `serializeKey` | function | Stable string key from a query key array (for `lists` map and dedupe). |
| `fetchEntity` | function | Loads/refreshes one entity through store-scoped dedupe and graph writes; accepts an optional application-owned graph. |
| `fetchList` | function | Loads/refreshes a list into one selected graph, normalizes rows, and stores **IDs** under the list key. |
| `dedupe` | function | Collapse an in-flight key within one graph; same keys in different request stores remain independent. |
| `startGarbageCollector` | function | Starts optional TTL-based cleanup for a selected application-owned graph; defaults to the compatibility singleton. |
| `stopGarbageCollector` | function | Stops the selected graph's garbage-collection loop; defaults to the compatibility singleton. |
| `attachGlobalListeners` | function | Reference-count focus/reconnect listeners and GC for one graph; returns the matching disposer. React hooks own this lifecycle automatically. |
| `EngineOptions` | type | Configuration object for `configureEngine`. |
| `EntityQueryOptions` | type | Options for single-entity fetch pipeline (normalizer, subscriber behavior, etc.). |
| `ListQueryOptions` | type | Options for list fetch (pagination mode, normalizer, merge strategy). |
| `ListFetchParams` | type | Cursor/page parameters passed to list fetch implementations. |
| `ListResponse` | type | Normalized list result shape expected from adapters (ids + meta). |

---

## Hooks — REST (`hooks.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntity` | hook | Subscribe to one entity by type/id; triggers fetch/SWR via engine; returns merged entity + status. |
| `useEntityList` | hook | Subscribe to a list query key; resolves `ids` to entities from the graph; handles pagination helpers. |
| `useEntityMutation` | hook | Orchestrates create/update/delete with graph upserts/removals and optional invalidation hooks. |
| `useEntityAugment` | hook | Read/write **patches** for an entity (selection, loading flags, etc.) visible to all subscribers. |
| `useSuspenseEntity` | hook | Suspense variant: throws a promise while loading; requires non-null id; use inside `<Suspense>` boundaries. |
| `useSuspenseEntityList` | hook | Suspense variant for list queries; same graph semantics as `useEntityList`. |

---

## DevTools (`devtools.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `useGraphDevTools` | hook | Development-oriented subscription to graph stats / introspection (useful for debugging, not for production UI). |

### Core DevTools subpath (`@prometheus-ags/entity-graph-core/devtools`)

| Export | Kind | Description |
|--------|------|-------------|
| `attachGraphDevtools` | function | Reference-count one per-store controller attachment with bounded history and an explicit value policy. |
| `getGraphDevtoolsController` | function | Read the currently attached controller for a selected `GraphStore`, or `null`. |
| `createGraphDevtoolsClient` | function | Create a transport-only command/event client for one controller store ID. |
| `projectGraphDevtoolsEntityRecords` | function | Project canonical, patch, merged, dirty, fetch/sync, revision, and view-membership records without retaining another graph. |
| `projectGraphDevtoolsRelationships` | function | Derive deterministic resolved/missing relationship edges from the existing CRUD schema registry and current merged graph values. |
| `GRAPH_DEVTOOLS_PROTOCOL` | constant | Stable protocol identifier used by every command, result, snapshot, and event envelope. |
| `GRAPH_DEVTOOLS_PROTOCOL_VERSION` | constant | Current wire major; incompatible envelopes are rejected. |
| `GraphDevtoolsController` | type | Per-store event/snapshot history, inspection, stable-cursor rewind/live, confirmed import, view-registration, command, transport, and lifecycle surface. |
| `GraphDevtoolsSnapshotHistoryStatus` | type | Live/rewound mode, source, active cursor, retained count/bytes, limits, stable range, unavailable capture, and inert import-candidate metadata. |
| `GraphDevtoolsSnapshotReference` | type | Stable retained snapshot metadata or an unavailable capture with a typed retention/capture/oversize reason. |
| `GraphDevtoolsRewindResult` | type | Successful retained rewind or a typed expired-history receipt that never substitutes another cursor. |
| `GraphDevtoolsReturnToLiveReceipt` | type | Exact restoration of the live head protected by the first rewind. |
| `GraphDevtoolsHistoryImportEnvelope` | type | Versioned, same-store, JSON-safe complete graph snapshots accepted only for inert inspection. |
| `GraphDevtoolsHistoryImportInspectionResult` | type | Awaiting-confirmation metadata or a typed non-mutating import rejection. |
| `GraphDevtoolsHistoryImportRestoreResult` | type | One-shot confirmed import rewind receipt or typed rejection. |
| `GraphDevtoolsTimeTravelEvent` | type | Ordered rewound/live transition event with retained/import source and command/import/mutation reason. |
| `GraphDevtoolsEntityRecord` | type | One inspection record with distinct canonical, patch, and merged values plus dirty and lifecycle metadata. |
| `GraphDevtoolsViewRegistration` | type | Token-scoped handle used by a rendered view to update membership and unregister safely. |
| `GraphDevtoolsRelationship` | type | Schema-derived outgoing/reverse edge with resolved or missing-target status. |
| `GraphDevtoolsPreviewRestoreReceipt` | type | Successful exact restore or typed conflict result after an intervening entity/patch publication. |
| `GraphDevtoolsValuePolicy` | type | Metadata-only default or explicit value inclusion with a host redactor for history and inspection destinations. |

---

## View layer (`view/`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntityView` | hook | Virtual collection over the graph: local / remote / hybrid completeness, filter/sort/search, optional fetch integration. |
| `FilterSpec` | type | Top-level filter: flat `FilterClause[]` **or** nested `FilterGroup` (`and` / `or`). |
| `FilterGroup` | type | Recursive boolean group of clauses and sub-groups. |
| `FilterClause` | type | Atomic predicate: `field`, `op`, optional `value`, optional `predicate` for `custom` ops. |
| `FilterOperator` | type | Supported comparison operators (`eq`, `in`, `contains`, `custom`, …). |
| `SortSpec` | type | Ordered array of `SortClause` (multi-key sort). |
| `SortClause` | type | One sort key: `field`, `direction`, optional `nulls`, optional `comparator`. |
| `SortDirection` | type | `"asc"` \| `"desc"`. |
| `ViewDescriptor` | type | `filter?`, `sort?`, `search?` — everything needed to describe a view for local or remote execution. |
| `CompletenessMode` | type | `"local"` \| `"remote"` \| `"hybrid"` — how much can be answered from the graph vs server. |
| `toRestParams` | function | Compile `ViewDescriptor` to flat REST query string params. |
| `toSQLClauses` | function | Compile to parameterized SQL `where` / `orderBy` fragments + `params` array. |
| `toGraphQLVariables` | function | Produce a Hasura/Postgraphile-style variable object from a view (starting point for wiring). |
| `toPrismaWhere` | function | JSON-serializable Prisma `where` object from filters (pair with server route using Prisma). |
| `toPrismaOrderBy` | function | JSON-serializable Prisma `orderBy` from `SortSpec`. |
| `flattenClauses` | function | Normalize nested `FilterGroup` trees to a flat `FilterClause[]` for simple compilers. |
| `hasCustomPredicates` | function | Returns true if any clause uses `op === "custom"` (cannot be pushed to generic REST/SQL). |
| `applyView` | function | Apply filter/sort/search in **local** JS against in-memory entities (evaluator). |
| `compareEntities` | function | Multi-key comparator implementing `SortSpec` (stable ordering semantics). |
| `matchesFilter` | function | Test one entity against a `FilterSpec`. |
| `matchesSearch` | function | Multi-field substring search helper aligned with view search config. |
| `checkCompleteness` | function | Determine whether graph data is complete enough for local vs remote handling for a descriptor. |
| `UseEntityViewResult` | type | Return type of `useEntityView`. |
| `UseEntityViewOptions` | type | Options for `useEntityView`. |
| `ViewFetchParams` | type | Compiled params passed to optional remote fetch in view hook. |

---

## CRUD (`crud/`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntityCRUD` | hook | Unified list + detail + create/edit **edit buffer** (React state) + save/cancel + dirty tracking + optional optimistic patch. |
| `registerSchema` | function | Register `EntitySchema` for an `EntityType` (relations + invalidation hints). |
| `getSchema` | function | Lookup registered schema or `null`. |
| `cascadeInvalidation` | function | After mutation success, mark related lists/entities stale from schema graph (FK moves, collections, M2M). |
| `readRelations` | function | Resolve relation placeholders for UI: joins graph reads for belongs-to / has-many / many-to-many. |
| `CRUDOptions` | type | Configuration for `useEntityCRUD` (type, keys, fetchers, etc.). |
| `CRUDState` | type | Full CRUD hook state snapshot type. |
| `CRUDMode` | type | Current UI mode enum for CRUD flow. |
| `DirtyFields` | type | Map of which fields diverge from canonical entity. |
| `EntitySchema` | type | Declares `type`, optional `relations`, optional `globalListKeys`. |
| `RelationDescriptor` | type | Union of relation shapes below. |
| `BelongsToRelation` | type | FK edge: `foreignKey`, `targetType`, optional `invalidateTargetLists`. |
| `HasManyRelation` | type | Inverse collection via `foreignKey` + `listKeyPrefix(parentId)`. |
| `ManyToManyRelation` | type | Id array on entity + `listKeyPrefix` for partner lists. |
| `CascadeContext` | type | `{ type, id, previous, next, op }` passed into cascade logic after mutations. |

---

## Realtime (`adapters/`)

| Export | Kind | Description |
|--------|------|-------------|
| `RealtimeManager` | class | Coalesces incoming `ChangeSet`s and applies them to the graph on a flush window (default ~16ms). |
| `getRealtimeManager` | function | Singleton accessor; configure `flushInterval`, status callbacks, etc. |
| `resetRealtimeManager` | function | Testing / HMR helper to tear down singleton state. |
| `createWebSocketAdapter` | function | Build a `RealtimeAdapter` from a WebSocket URL/protocol. |
| `createSupabaseRealtimeAdapter` | function | Adapter for Supabase Realtime channels. |
| `createConvexAdapter` | function | Adapter for Convex subscription-style feeds. |
| `createGraphQLSubscriptionAdapter` | function | Adapter bridging GraphQL subscriptions to `ChangeSet`s. |
| `createFlintAdapter` | function | Bridge a caller-owned Flint client's `watchEntities` stream into `RealtimeManager`. |
| `publishFlintMutation` | function | Delegate one `FlintEntityRecord` to the caller-owned client's `mutateEntity` method. |
| `ManagerOptions` | type | Options for realtime manager construction. |
| `WebSocketAdapterOptions` | type | WebSocket adapter configuration. |
| `SupabaseAdapterOptions` | type | Supabase-specific adapter options. |
| `RealtimeAdapter` | type | Interface: subscribe/unsubscribe, emit normalized changes. |
| `SyncAdapter` | type | Optional sync-oriented adapter surface. |
| `ChangeSet` | type | Batch of entity changes for one flush tick. |
| `EntityChange` | type | Single create/update/delete payload inside a changeset. |
| `ChangeOperation` | type | Operation discriminator for a change. |
| `ChannelConfig` | type | Channel subscription metadata. |
| `AdapterStatus` | type | Connection health enum / union. |
| `UnsubscribeFn` | type | Teardown function returned from subscriptions. |
| `SubscriptionConfig` | type | Subscription parameters. |
| `FlintClientLike` | type | Structural `watchEntities` / `mutateEntity` client boundary; no Flint SDK is bundled. |
| `CreateFlintAdapterOptions` | type | Channel, consumer, entity filter, checkpoint store, and operation resolver. |
| `FlintEntityEvent`, `FlintEntityQuery`, `FlintEntityRecord` | types | Decoded event, subscription query, and mutation record contracts. |
| `FlintCheckpointStore` | type | Optional per-channel/consumer bigint resume-offset persistence. |

---

## GraphQL (`graphql/`)

| Export | Kind | Description |
|--------|------|-------------|
| `createGQLClient` | function | Factory returning a configured `GQLClient` instance. |
| `GQLClient` | class | Executes operations, normalizes entity-shaped responses into the graph. |
| `normalizeGQLResponse` | function | Map arbitrary GQL JSON into entity inserts keyed by `EntityDescriptor`s. |
| `executeGQL` | function | One-shot execution helper using client config. |
| `useGQLEntity` | hook | Graph-backed single entity with GQL document + variables. |
| `useGQLList` | hook | Graph-backed list via GQL with normalization into entities + list ids. |
| `useGQLMutation` | hook | Mutation helper that updates graph + handles errors. |
| `useGQLSubscription` | hook | Subscribe to server pushes and forward into graph/realtime pipeline as appropriate. |
| `GQLClientConfig` | type | Endpoint, headers, fetcher, default type map. |
| `GQLError` | type | Structured GraphQL error. |
| `GQLResponse` | type | Typed response envelope. |
| `EntityDescriptor` | type | Describes how to normalize a GQL selection into `EntityType` + id field + nested fragments. |
| `GQLEntityOptions` | type | Options for `useGQLEntity`. |
| `GQLListOptions` | type | Options for `useGQLList`. |

---

## Prisma adapter (`adapters/prisma.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `createPrismaEntityConfig` | function | Build config bridging Prisma models to entity types and list keys. |
| `prismaRelationsToSchema` | function | Derive `EntitySchema` relation entries from Prisma relation metadata. |
| `toPrismaInclude` | function | Compute `include` object for nested fetches from descriptors. |
| `PrismaEntityConfigOptions` | type | Options for Prisma entity config factory. |

*Note:* `toPrismaWhere` and `toPrismaOrderBy` are also exported from `view/types` (re-exported from `prisma-compile`) for view-driven Prisma queries.

---

## Local-first (`adapters/electricsql.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `createElectricAdapter` | function | ElectricSQL / shape stream adapter emitting `ChangeSet`s compatible with `RealtimeManager`. |
| `useLocalFirst` | hook | Orchestrates local-first sync + graph hydration patterns with PGlite/Electric. |
| `usePGliteQuery` | hook | Run SQL against embedded PGlite and reflect results into React + graph workflows. |

## Companion peer-sync package

`@prometheus-ags/entity-graph-sync` is a separately published companion package;
its runtime export ledger is `sync-library-exports.json` and its full certified
boundary is documented in `sync-persistence-path.md`.

| Export | Kind | Purpose |
| --- | --- | --- |
| `createSyncProviderRegistry` | function | Create client-owned provider registration state for an injected graph store. |
| `startSyncBridge` | function | Bridge one store/registry pair to its providers while suppressing inbound echo. |
| `applyPeerChanges` | function | Apply peer fields and sync metadata to the default or injected graph store. |
| `createLoroProvider` | function | Create deterministic per-type Loro documents over a selected channel. |
| `createLoroLoopbackNetwork` | function | Create the deterministic offline/reconnect reference fabric. |
| `createWebSocketLoroChannel` | function | Create a queued, reconnecting, peer-resynchronizing WebSocket channel. |
| `encodeLoroWebSocketMessage`, `decodeLoroWebSocketMessage` | functions | Encode/decode stable opaque relay frames. |
| `SyncProviderRegistry`, `SyncBridgeOptions` | types | Client ownership and bridge configuration contracts. |
| `LoroProviderOptions`, `LoroChannel`, `LoroChannelStatus` | types | Loro provider and transport lifecycle contracts. `LoroProviderOptions.loadLoro` exposes the bundler-visible optional-peer loader. |
| `LoroLoopbackNetwork`, `LoroWebSocketChannelOptions` | types | Deterministic and real-transport channel configuration contracts. |
| `ElectricAdapterOptions` | type | Shape URLs, table maps, auth, etc. |
| `ElectricTableConfig` | type | Per-table sync configuration. |
| `UseLocalFirstResult` | type | Hook result: status, connection, helpers. |

---

## UI (`ui/`)

| Export | Kind | Description |
|--------|------|-------------|
| `EntityTable` | component | Opinionated data table: pagination, inline edit hooks, integration with column defs. |
| `InlineCellEditor` | component | Cell-level editor used by `EntityTable` for editable columns. |
| `EntityDetailSheet` | component | Read-only side sheet for one entity. |
| `EntityFormSheet` | component | Create/edit sheet bound to field descriptors and submit handlers. |
| `Sheet` | component | Primitive sheet/drawer wrapper used by the entity sheets. |
| `SortHeader` | component | Table header control wiring sort state to `SortSpec` / table APIs. |
| `selectionColumn` | function | Column factory for row selection checkboxes + graph patch patterns. |
| `textColumn` | function | Text column with optional filter metadata. |
| `numberColumn` | function | Numeric column + filter type hints. |
| `dateColumn` | function | Date/datetime column. |
| `enumColumn` | function | Enum / select-style column. |
| `booleanColumn` | function | Boolean toggle column. |
| `actionsColumn` | function | Row actions menu column. |
| `EntityColumnMeta` | type | Metadata for filters and entity-aware table features. |
| `ColumnFilterType` | type | Drives toolbar filter control selection. |
| `ActionItem` | type | Declarative row action definition. |
| `FieldDescriptor` | type | Describes one form field for `EntityFormSheet`; now supports dotted field paths for JSON-column-backed editors. |
| `FieldType` | type | Field editor type enum / union, including `json` and `markdown`. |

---

## Import map (package name)

```ts
import {
  useGraphStore,
  useEntity,
  useEntityList,
  useEntityView,
  registerSchema,
  // …
} from "@prometheus-ags/prometheus-entity-management";
```

In this monorepo’s examples, the package may resolve via TypeScript path alias to `src/index.ts` during development.

## Canonical Dart companion library

`entity_graph_flutter` has an independent declaration ledger at
`dart-library-exports.json`, an optional tooling-entry ledger at
`dart-devtools-library-exports.json`, and detailed agent guidance in
`dart-graph-riverpod.md` plus `devtools-flutter-controller.md`. Its generated Riverpod families orchestrate one
Dart-native graph; it is not a wrapper around this React package and does not
share a runtime singleton across JavaScript and Dart processes. Run
`pnpm run verify:dart-exports` after any Dart public declaration change.
