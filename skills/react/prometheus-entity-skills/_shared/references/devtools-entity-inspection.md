# Core DevTools entity inspection

This reference governs agent guidance for the optional
`@prometheus-ags/entity-graph-core/devtools` inspection surface. The runtime
export names are authoritative in [`core-library-exports.json`](core-library-exports.json),
and the wire semantics are frozen by
`packages/entity-graph-core/fixtures/devtools/entity-inspection-v1.json`.

## Architecture boundary

- Attach a controller to an explicit `GraphStore`. Each store owns independent
  events, history, entity revisions, view registrations, preview receipts,
  clients, and teardown.
- Inspection is a projection of `entities`, `patches`, entity lifecycle state,
  sync metadata, lists, and the existing CRUD schema registry. Never create a
  second entity graph, durable DevTools patch store, or parallel relationship
  registry.
- UI bindings may register stable rendered-view IDs and update their current
  entity membership, but registration state belongs to the controller and is
  cleared on final detach.
- Remote/browser/VM-service bridges carry the versioned transport contract;
  they do not gain authority to mutate canonical business state.

## Entity semantics

- `canonical` is the server-confirmed original.
- `patch` is the current local graph overlay.
- `merged` is the live value graph consumers read.
- `dirtyReasons` enumerates patched fields and unsynchronized state. `dirty`
  is derived from that list.
- Entity revisions advance once per store publication that touches entity,
  patch, fetch, or sync state. Preview conflict checks use the narrower
  canonical/patch revision so metadata-only updates do not cause false
  conflicts.
- Fetch errors retain the graph's public message and expose `retryable: null`
  when the compatibility state does not retain a typed retryability flag.

## Views and relationships

Register a stable `viewId`, label, kind, entity type, and optional list query
key through `GraphDevtoolsController.registerView()`. Call
`updateMembership()` when rendered IDs change and `unregister()` on cleanup.
Duplicate registrations for one stable ID are token-scoped; the projected view
persists until the final registration leaves.

`get-views` returns view-to-entity membership, entity-to-view membership, render
counts/timestamps, and list statistics. `get-relationships` derives
`belongsTo`, reverse `hasMany`, and `manyToMany` edges from registered CRUD
schemas and current merged values. Unresolved IDs remain visible as
`missing-target` edges.

## Preview and restore

`preview-entity-patch` applies a proposed patch through the graph's existing
`patchEntity` action and returns a receipt containing the exact prior patch.
Only one active receipt is retained per entity. `restore-entity-preview`
atomically restores the prior patch when the canonical/patch revision still
matches the preview receipt; otherwise it returns a typed conflict and leaves
the current patch untouched. Preview is local and inspectable, never a commit
or server mutation.

## Security and transport values

The default value policy is metadata-only. Canonical values, patches, merged
values, and preview receipt values are represented by an explicit
`hidden-by-policy` marker. A host must opt into `values.mode: "include"` and
apply any required redactor before values may cross the inspection boundary.
Redactor failures yield a marker and never expose the input or exception.
Values are JSON-normalized, and controllers/receipts are bounded by controller
lifecycle.

## Fixture and evidence boundary

The source fixture has a byte-identical Flutter copy at
`packages/entity_graph_flutter/fixtures/devtools/entity-inspection-v1.json` and
is published from core at
`./devtools/fixtures/entity-inspection-v1.json`. A semantic change requires a
new fixture version; runtime-specific edits are forbidden.

The assembled acceptance command is:

```bash
pnpm run verify:devtools-entity-inspection
```

It proves packed ESM/CommonJS/NodeNext consumption, projections, membership,
relationships, preview propagation, exact and metadata-only restore, conflict
refusal, multi-store isolation, policy enforcement, package payload, and shared
fixture parity. It does not prove the later React inspector, Flutter controller,
Chrome extension, Flutter DevTools extension, documentation site, or release
publication changes.
