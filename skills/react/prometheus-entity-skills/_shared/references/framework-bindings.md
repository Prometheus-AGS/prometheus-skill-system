# Framework bindings — Svelte, Solid, Alpine, HTMX, Web Components

All five bindings are thin reactive facades over
`@prometheus-ags/entity-graph-core`. They share one contract: the core graph
is the single source of truth, each binding is a **singleton facade** per
application, and no binding stores entity copies. The packed six-binding
contract (React + these five) is verified by
`pnpm run verify:binding-singletons`; per-package export ledgers are verified
by `pnpm run verify:skills`.

## Svelte — `@prometheus-ags/entity-graph-svelte`

Exports: `initEntityGraph`, `createEntityStore`, `createEntityList`.

- Call `initEntityGraph` once at app bootstrap; `createEntityStore(type, id)`
  and `createEntityList(queryKey)` return Svelte stores that re-render on
  graph writes.

## Solid — `@prometheus-ags/entity-graph-solid`

Exports: `createGraphStore`, `graphStore`, `setupGraphProvider`,
`useGraphStore`, `createEntity`, `createEntityList`, `TerminalError`,
`TransientError`.

- `setupGraphProvider` installs the graph; `createEntity`/`createEntityList`
  return reactive accessors.

## Alpine — `@prometheus-ags/entity-graph-alpine`

Exports: `createEntityGraphPlugin`, `EntityGraphAlpinePlugin`,
`createEntityBinding`, `createListBinding`.

- Register with `Alpine.plugin(createEntityGraphPlugin(...))`; bindings expose
  entity/list state to `x-data` scopes.

## HTMX — `@prometheus-ags/entity-graph-htmx`

Exports: `createServerGraph`, `createHtmxSseServer`, `createSseClient`,
`renderFragment`, `wrapOobFragment`, `autoRenderEntity`, `subscriptionKey`,
`clientSubscribedTo`.

- Server owns the graph; SSE pushes rendered fragments;
  `wrapOobFragment` emits out-of-band swaps so one entity change updates every
  rendered occurrence.

## Web Components — `@prometheus-ags/entity-graph-web-components`

Exports: `EntityListElement`, `EntityDetailElement`, `EntityFormElement` and
their controllers.

- Framework-agnostic custom elements; usable from plain HTML, React, Vue, or
  server-rendered pages.

## Common rules

- Lists carry entity IDs only; bindings join IDs against the graph at render
  time.
- Realtime flows through the core `RealtimeManager`; bindings never open
  sockets themselves.
- Local-only UI state (selection, expansion) goes through graph patches, not
  component-local copies, when it must be visible across views.
