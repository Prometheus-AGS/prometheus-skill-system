# Flutter DevTools controller and VM-service bridge

This reference governs agent guidance for the optional
`package:entity_graph_flutter/devtools.dart` library implemented in repository
source after the published `3.0.1` archive. Do not claim that pub.dev `3.0.1`
contains this entry. The separate official Flutter DevTools extension UI is a
later change and is not certified by the controller gate.

## Entry and ownership boundary

- The ordinary `entity_graph_flutter.dart` library remains the application
  graph/Riverpod entry and does not export or initialize DevTools.
- A development host explicitly imports `devtools.dart` and calls
  `EntityGraphDevtoolsBinding.attach` for its application-owned `EntityGraph`.
- Repeated enabled attachments to one graph share a reference-counted
  controller. Detach every retained binding; the final detach disposes history,
  snapshots, subscriptions, preview receipts, and VM-service store routing.
- One controller observes one graph's completed publication boundary. It does
  not own canonical entities, patches, lists, or another event/state store.
- Riverpod entity and list providers register logical views with the graph;
  the controller projects those registrations and memberships without widgets
  calling the graph directly.

## Protocol and store isolation

The wire protocol is `prometheus.entity-graph.devtools`, major version 1. The
isolate-wide VM-service router registers:

- `ext.entity_graph_flutter.devtoolsV1.listStores`;
- `ext.entity_graph_flutter.devtoolsV1.command`;
- Extension kind `prometheus.entity-graph.devtools.v1`.

Every command carries an explicit active `storeId` and request ID. Never add an
implicit default graph. Duplicate active store IDs fail; final detach removes
the store from discovery. Dart service-extension method registration is
isolate-global and cannot be undone, so an empty registry after final detach is
the correct terminal state.

The controller exposes capabilities, counts, bounded event/history status,
entity canonical/patch/merged records, dirty/error/sync state, registered views
and ID-only membership, optional-schema relationships, conflict-safe local
preview/restore, snapshot status, rewind, return-to-live, inert history import
inspection, explicit confirmed restore, and history clearing.

## Debugger trust boundary

Flutter debug mode enables service extensions and DevTools; release mode strips
debugging services. Still gate attachment explicitly with the host's development
mode. The VM service is a debugger/tool trust boundary, not an authorization
system and not a production remote API.

- Metadata-only is the default value policy.
- Value inclusion is a host construction choice. Apply any required redactor
  before values enter retained history, events, or inspection results.
- Remote commands cannot enable values, replace the redactor, or commit a local
  preview. Restore succeeds only when the entity revision still matches the
  preview receipt.
- Request envelopes are limited to 256 KiB, responses to 8 MiB, and events to
  the controller's default 256 KiB ceiling. Oversize values are represented as
  bounded/truncated metadata rather than partially trusted payloads.
- Defaults retain at most 500 events / 5 MiB and 50 snapshots / 10 MiB per
  controller; both the count and byte ceilings apply.
- Hidden, included, redacted, redaction-error, truncated, unavailable, and
  actual `null` states remain distinguishable.

Never expose the VM-service URI, debugger authentication token, entity secrets,
registry credentials, or raw diagnostic streams in evidence or documentation.

## Public API ledgers

- `dart-library-exports.json` covers
  `package:entity_graph_flutter/entity_graph_flutter.dart`.
- `dart-devtools-library-exports.json` covers
  `package:entity_graph_flutter/devtools.dart`, including public declarations
  in its controller parts.

Both are regenerated and verified together:

```bash
pnpm run refresh:dart-exports
pnpm run verify:dart-exports
```

## Evidence boundary

The full assembled controller gate is:

```bash
pnpm run verify:devtools-flutter-controller
```

It launches a configured Flutter debug application on a real device or
simulator, connects from an external Node process over VM-service WebSocket
JSON-RPC, mounts real Riverpod entity/list providers, and drives two isolated
graphs. The accepted receipt proves fixture parity, store discovery and
teardown, semantic events, projections, view membership, redaction and policy
refusal, payload limits, preview conflict refusal, rewind/live behavior,
history clearing, and 28 versioned Extension events.

This gate does not certify the separate Flutter DevTools web extension,
Chrome extension, documentation deployment, immutable release-wide source SHA,
pub.dev payload, or publication authority.
