# Core DevTools time travel

This reference governs agent guidance for controller-owned snapshot history in
the optional `@prometheus-ags/entity-graph-core/devtools` entry. Runtime export
names remain authoritative in
[`core-library-exports.json`](core-library-exports.json); cross-runtime semantics
are frozen by
`packages/entity-graph-core/fixtures/devtools/time-travel-v1.json`.

## Ownership and capture

- One attached controller owns history for exactly one `GraphStore`. Never add
  a UI ring, process-global snapshot registry, second graph, or shared cursor.
- Capture the initial attach baseline and the completed five-field graph data
  slice (`entities`, `patches`, `entityStates`, `syncMetadata`, and `lists`)
  associated with each semantic mutation publication.
- Deep-clone on capture and restore. Snapshot data contains no graph actions or
  controller state and is released on eviction, clearing, or final detach.
- Mutation events retain only their stable capture reference. Complete snapshot
  values never enter event, inspection, or command-result envelopes.

## Retention and stable cursors

The default policy is at most 50 complete snapshots and at most 10 MiB per
controller; whichever ceiling is reached first wins. Evict whole oldest
snapshots only. Stable monotonically increasing cursors are never reused after
eviction or clearing. An oversize or failed capture is represented explicitly
as unavailable metadata and is never partially retained.

Resolve rewind commands by stable cursor, never by array index. An evicted,
cleared, or unavailable cursor returns `expired-history` with the retained
range and leaves the graph unchanged.

## Rewind, live return, and branching

Rewind writes through the real Zustand store boundary so every subscriber sees
the selected state. The first rewind protects the exact pre-rewind live head;
later cursor changes preserve that same head. `returnToLive()` restores it
atomically and releases it.

Internal replay publications are not ordinary business mutations and must not
recursively capture snapshots. If a real graph mutation occurs while rewound,
the controller leaves rewind mode first, emits the live transition with
`reason: "mutation"`, then emits and snapshots the completed mutation as the
new live branch. The prior protected future is discarded.

## Import boundary

Inspection accepts only JSON-safe complete graph data with the current protocol
version, the exact controller store ID, ordered positive cursors, valid
timestamps, and the controller's shared count/byte budget. Inspection is inert.
Restore requires explicit confirmation of the same one-shot candidate ID and a
cursor contained by that candidate. Live graph activity, replacement
inspection, confirmation, clearing, or disposal invalidates the candidate.

## Deprecated root facade

`recordGraphSnapshot`, `restoreGraphSnapshot`,
`restoreGraphSnapshotBySeq`, `stepTimeTravel`, `getTimeTravelState`,
`subscribeTimeTravel`, and `configureTimeTravel` remain compatibility exports.
They delegate to the selected store's optional controller and own no snapshot
payload, retention, or cursor. Root-only consumers receive unavailable results
until the explicit `./devtools` module is loaded; new tools should attach a
controller and use stable cursors plus explicit return-to-live.

## Fixture and evidence boundary

The source fixture has a byte-identical Flutter copy at
`packages/entity_graph_flutter/fixtures/devtools/time-travel-v1.json` and is
published from core at `./devtools/fixtures/time-travel-v1.json`. A semantic
change requires a new fixture version; runtime-specific edits are forbidden.

The assembled acceptance command is:

```bash
pnpm run verify:devtools-time-travel
```

It proves packed root-only ESM, full ESM, CommonJS, and strict NodeNext
consumption; count/byte retention; oversize and expired cursors; rewind and
exact live return; mutation branching order; bounded confirmed import; store
isolation; teardown; root facade compatibility; optional root-payload
exclusion; and shared fixture parity. It does not prove the later React
inspector, Flutter controller, Chrome or Flutter extensions, documentation
site, release certification, or registry publication.
