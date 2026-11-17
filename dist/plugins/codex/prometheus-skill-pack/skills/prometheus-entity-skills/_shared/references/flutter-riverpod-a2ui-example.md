# Flutter, Riverpod, and A2UI application reference

Load this reference when generating, reviewing, or documenting a Flutter
application that combines `entity_graph_flutter`, generated Riverpod 3
providers, policy-controlled A2UI, offline-aware CRUD, realtime changes, and
an optional native transport.

The runnable example is
[`examples/flutter-riverpod`](../../../examples/flutter-riverpod/README.md),
and its release boundary is
[`release/flutter-riverpod-a2ui-example.md`](../../../release/flutter-riverpod-a2ui-example.md).
Also load `dart-graph-riverpod.md` for the canonical package contract and
`flutter-source-provenance.md` before making source-lineage claims.

## Required composition

```text
Widget -> generated provider/controller -> application controller
  -> EntityGraph -> EntityTransport

A2UI 1.0 messages -> atomic load (a2ui_core MessageProcessor) -> official a2ui_flutter renderer
  -> exact action policy -> human approval -> CRUD controller -> EntityGraph
```

- Create one application-owned `EntityGraph` and override
  `entityGraphProvider`; never create per-screen graphs.
- Keep widgets free of transport calls and copied entity state.
- Keep lists ID-only and join rows from the graph at read time.
- Route create/update/delete through generated Riverpod controllers.
- Keep offline queue state and realtime normalization below the widget layer.
- Treat `FfiEntityTransportAdapter` as an optional transport seam, not graph
  ownership and not proof of a bundled Rust runtime.

## A2UI safety contract

- Pin `a2ui_flutter` 0.3.0 and `a2ui_core` 0.7.0 exactly while they are pre-1.0.
- Load each untrusted batch atomically: if the processor rejects any message,
  or the batch leaves a surface other than the registered one, remove every
  surface and show the refusal instead of a partial surface.
- Narrow the basic catalog with `Catalog.derive` under the app's own catalog id
  (never the basic catalog's id) to the components the app renders and no
  client functions; allow only createSurface, updateComponents,
  updateDataModel and deleteSurface messages; refuse undeclared action names
  before the batch reaches the processor.
- Apply tenant/task authorization after protocol validation.
- Require trusted human approval for archival/destructive intent.
- Route allowed actions through generated CRUD controllers; the renderer never
  receives direct graph authority.
- Do not claim byte-identical React/Flutter A2UI output: the two renderers
  draw the same 1.0 messages with different widget sets (1.0 Text has only
  `caption` and `body` styles).

## Certified evidence

```bash
pnpm run dart:bootstrap:frozen
pnpm run verify:flutter-a2ui-v1 -- --report <file>
```

The A2UI 1.0 gate checks the package pins and runs the integration test on a
booted iOS simulator (denied action, allowed action, refused unknown-component
batch), plus a flipped copy that must fail. The 3.0 test counts, goldens and
Android emulator lane predate the A2UI 1.0 migration and are not re-certified
by it. Consult `examples/coverage.json` for exact paths.

The example changes no package declaration. Do not refresh
`dart-library-exports.json` unless `packages/entity_graph_flutter/lib` actually
changes its public declarations. Do not describe an authored workflow alone as
a hosted execution receipt, or simulator/emulator smoke as physical-device,
native accessibility, durable-persistence, or publication authority.
