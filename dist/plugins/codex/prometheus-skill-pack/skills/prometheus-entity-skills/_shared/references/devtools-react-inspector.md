# React DevTools inspector

This reference governs agent guidance for the optional React inspector
implemented in repository source after the published `3.1.0` package. The
entries will ship in the next minor release and must not be claimed as present
in the `3.1.0` tarball. Runtime export names for
the root, `./devtools`, and `./devtools/auto` entries are authoritative in
[`library-exports.json`](library-exports.json).

## Entry and bundle boundary

- The ordinary root remains side-effect-free. It retains the lightweight
  compatibility hook `useGraphDevTools` but does not import the inspector,
  React DOM client mounting, or CSS.
- `./devtools` is the side-effect-free explicit host/provider/hook entry.
  `EntityGraphDevtools` is SSR-safe and lazy-loads the inspector only after
  its mode is enabled.
- `./devtools/auto` is the only import-time mounting entry. The import is the
  consumer's explicit debug opt-in; it checks development/browser/host mode
  before mounting and exports explicit mount/unmount controls.
- The host renders one open Shadow Root with scoped styles. No global reset,
  remote font, selector, or custom-property mutation enters the application.

## Vite and Next activation

Vite clients use a literal debug-only dynamic import:

```ts
if (import.meta.env.DEV) {
  void import("@prometheus-ags/prometheus-entity-management/devtools/auto");
}
```

Next.js clients dynamically import `./devtools` from a `"use client"`
component after hydration and pass the application/provider-owned
`GraphStore` to `EntityGraphDevtools`. Never mount the launcher in server
markup or attach the inspector to the process-global fallback during
request-scoped rendering.

## UI and interaction contract

- Automatic development activation displays a floating **Graph** launcher.
- The display menu moves or compacts the launcher, selects floating/right/bottom
  layouts, hides it until reload, or persists browser hiding.
- The default restore/toggle shortcut is
  <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>G</kbd>. Hosts may supply
  another shortcut or disable it.
- The four workspaces are Overview, Entities, Views, and Activity. Graph Pulse
  is the persistent causal ribbon across them.
- Entities expose canonical original, local patch, merged live value, field
  diff, dirty reasons, relationships, registered rendered-view membership,
  retained history, and conflict-safe preview/restore.
- Activity uses controller-owned events and stable snapshot cursors for rewind
  and explicit return to live. The React UI does not create another event or
  snapshot store.
- Lists over 50 items are virtualized. Search is deferred. Controller
  subscriptions are store-scoped and visible projections publish on a bounded
  cadence.

## Value and authority boundary

The embedded same-origin inspector may read its selected local store. Values
crossing serialized browser, VM-service, or remote transports remain
metadata-only until the host explicitly enables include mode and supplies any
required redactor. A transport cannot escalate the value policy or gain commit
authority. Hidden, redacted, unavailable, truncated, and actual `null` values
must remain distinct.

## Public runtime exports

`./devtools` exposes:

- `EntityGraphDevtools`, `EntityGraphDevtoolsProvider`, and
  `EntityGraphInspectorShell`;
- `useEntityGraphDevtools`, `useEntityGraphDevtoolsSnapshot`,
  `useEntityGraphInspectorModel`, and
  `useEntityGraphInspectorViewModel`;
- inspector model and URL-state adapter factories;
- mode, preference, shortcut, preload, parse, normalize, serialize, and
  persistence helpers.

`./devtools/auto` exposes `mountAutoEntityGraphDevtools` and
`unmountAutoEntityGraphDevtools`.

## Evidence boundary

The assembled acceptance command is:

```bash
pnpm run verify:devtools-react-inspector
```

It proves packed Vite/Next consumption, ordinary-root production exclusion,
automatic development activation, hide/restore, dirty/original/view/history
workflows, Graph Pulse causality, hydration, keyboard and screen-reader
accessibility, responsive layouts, and sustained 500-event interaction. It
does not certify Chrome or Flutter DevTools extensions. A 12-developer
formative study remains required before calling the UX formatively certified
“world class.”
