# Next.js App Router graph ownership

Use this contract when a Next.js App Router application renders graph-backed
data on the server.

## Required ownership flow

1. Create one `createGraphStore()` instance inside the document request.
2. Populate that request-owned store in server infrastructure or services.
3. Serialize only JSON/structured-clone-safe graph data across the RSC boundary.
4. Create one browser store from the snapshot before descendant hooks render.
5. Wrap the client tree in `GraphStoreProvider`.
6. Inside hooks or infrastructure, use `useGraphStoreApi()` when an adapter,
   mutation, or realtime manager needs the selected store.
7. Start WebSocket/realtime ownership only after client mount and dispose it on
   unmount.

Never write request data into the process-wide `graphStore`. Never create the
hydrated client store on every render. Never import React graph hooks into a
Server Component.

## Minimal shape

```tsx
// Server infrastructure
const requestStore = createGraphStore();
requestStore.getState().upsertEntities("Task", normalizedTasks);
const snapshot = dehydrate(requestStore.getState());

// Client provider
"use client";
const [store] = useState(() => hydrate(snapshot));
return <GraphStoreProvider store={store}>{children}</GraphStoreProvider>;
```

The `dehydrate`/`hydrate` helpers are application-owned because the application
decides which graph fields and tenant/request metadata may cross the boundary.

## Verification

For an SSR claim, prove all of the following against a production server:

- concurrent document requests receive distinct graph markers;
- the browser performs no duplicate fetch for hydrated rows;
- no hydration mismatch is reported;
- client route transitions retain the graph;
- a document reload creates a new request graph;
- mutations and realtime writes land in the provider-owned graph;
- packed candidates work without workspace aliases.

The monorepo reference command is `pnpm run verify:nextjs-app-router`; see
`examples/nextjs-app` and `release/nextjs-app-router-example.md`.
