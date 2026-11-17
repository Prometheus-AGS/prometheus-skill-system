# Migration patterns — legacy cache → entity graph

Patterns for moving from TanStack Query, Apollo, Redux, or SWR to **@prometheus-ags/prometheus-entity-management** without breaking production.

## 1. Parallel run (feature flag)

**When:** Large app, unknown edge cases, need instant rollback.

- Introduce graph-powered **duplicate route** or component tree under `flags.entityGraph`.
- Keep legacy `useQuery` path as default until parity testing passes.
- **Cutover:** flip default; remove dead path in a follow-up PR.

**Pros:** Lowest risk. **Cons:** Temporary bundle size increase.

## 2. Vertical slice (one entity)

**When:** Team wants proof of value fast.

- Pick one **`EntityType`** with clear list + detail (e.g. `Project`).
- Migrate **all screens** for that type in one PR: `useEntityList`, `useEntity`, mutations via `useEntityMutation` or CRUD hook later.
- Register **`EntitySchema`** for relations touching only that slice’s lists.

**Pros:** Teaches patterns once. **Cons:** Cross-entity screens need interim bridging.

## 3. Read-first

**When:** Writes are complex (optimistic workflows, multi-step wizards).

- Replace **reads** with graph hooks; **writes** stay on old mutation APIs until fetchers stabilize.
- Ensure successful writes still **`upsertEntity`** or invalidate so reads stay fresh.

**Pros:** Reduces cache divergence bugs early. **Cons:** Two mental models until writes migrate.

## 4. Apollo → graph

- Map each **normalized** Apollo typename to **`EntityType`**.
- GQL queries: prefer **`useGQLEntity` / `useGQLList`** from the library once descriptors exist; or REST-style `fetchList` that normalizes GQL responses manually.
- Subscriptions: hand off to **`entity-graph-realtime`** after base graph works.

## 5. TanStack Query → graph

- **Query keys** → **`serializeKey`** list keys; document mapping in `keys.ts`.
- **`placeholderData` / `initialData`** → often replaceable with graph **patches** or SSR hydration.
- **Dependent queries** → graph freshness + `invalidateEntity` / `invalidateLists` instead of manual `queryClient` coupling where possible.

## 6. Redux normalized entities → graph

- If using `entities` slice by id: migration is conceptual alignment—**graph is the new slice**.
- Move **async thunks** into engine fetchers or existing API modules called from engine.
- Keep Redux only for **truly UI-global** state if needed; do not duplicate server rows.

## 7. SSR hydration (Next.js)

- Create one isolated `createGraphStore()` instance per server request; never reuse the process-wide `graphStore` across requests.
- Populate and dehydrate that request graph on the server; pass only a serializable snapshot across the RSC boundary.
- In a client provider, create the hydrated graph exactly once and pass it to `GraphStoreProvider` before descendant hooks render.
- Hooks and infrastructure use `useGraphStoreApi()` when they must pass the selected graph to fetch, mutation, or realtime machinery.
- Follow `_shared/references/nextjs-app-router.md` and the library Next.js example; a one-time batch upsert into the global singleton is not request-isolation evidence.

## 8. Core/React store imports for 3.0

- Non-React modules import `graphStore` or `createGraphStore` from `@prometheus-ags/entity-graph-core`.
- React hooks import callable `useGraphStore` from `@prometheus-ags/prometheus-entity-management` only when no domain hook covers the use case.
- Replace early-alpha core imports of `useGraphSyncStatus` with `getGraphSyncStatus()` for imperative code or the React-package hook for components/custom hooks.
- Import React table/view presentation types from the React package, not core.
- Core's deprecated `useGraphStore` alias may ease an imperative migration, but new code must not treat it as callable.
- Install `@prometheus-ags/entity-graph-core` explicitly with every framework binding. It is the application's required compatible peer; do not add a second core version under a binding or suppress peer-resolution failures.
- When multiple JavaScript bindings coexist, run `pnpm run verify:binding-singletons` in the library workspace before claiming one physical packed core. Matching source aliases or version strings are not singleton evidence.

## 9. Rollback

Each plan should name:

- **Git ref** before migration
- **Files** introduced
- **Env flags** to disable graph path

## Anti-patterns

- Big-bang delete of TanStack Query without replacing **loading/error** UX.
- Mixing **two sources of truth** for the same `type:id` without a documented winner (graph must win post-migration).
