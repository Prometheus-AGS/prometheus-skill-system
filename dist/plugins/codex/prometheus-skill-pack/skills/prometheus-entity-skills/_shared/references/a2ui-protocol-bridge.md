# Official A2UI protocol bridge

Use this reference when an agent creates, reviews, or documents an official
A2UI surface, catalog, action, or Prometheus graph projection.

The full engineering and evidence contract is
[`release/a2ui-protocol-bridge.md`](../../../release/a2ui-protocol-bridge.md).
The machine runtime-export ledger is
[`a2ui-library-exports.json`](a2ui-library-exports.json).

## First distinction

**Protocol validity never grants application authority.**

Official A2UI validation answers whether a message, catalog component, data
binding, or action is structurally valid. It does not authorize a tenant,
entity, field, graph write, URL navigation, or destructive operation.

A useful mental model is: the catalog is an approved vocabulary, not a database
credential.

## Package imports

Use the package root only for official A2UI:

```ts
import {
  PrometheusA2uiProvider,
  PrometheusA2uiSurface,
  createEntityGraphA2uiActionPolicy,
  createPrometheusA2uiRuntime,
} from "@prometheus-ags/a2ui-react";
```

Use the explicit compatibility subpath for the alpha AG-UI chat/state APIs:

```ts
import {
  EntityApproval,
  EntityChat,
  useChatSession,
} from "@prometheus-ags/a2ui-react/ag-ui";
```

Never import `EntityChat` from the package root in 3.0 code. Never describe
AG-UI state/message events as A2UI surface conformance.

## Version rules

- Wire targets: A2UI `1.0` final (`version: "1.0"`, native through the official
  processor's 1.0 adapter, catalog `PROMETHEUS_A2UI_V1_CATALOG_ID`) and `v0.9.1`
  (still accepted). The 1.0 release-candidate dialect `"v1.0"` is refused unless
  `acceptReleaseCandidateInput` is set.
- Official packages: `@a2ui/react@0.15.0` and `@a2ui/web_core@0.15.0`
  (bundled; `web_core` patched to import `zod/v3`), with `zod@4.6.7`.
  Markdown uses a package-owned port of `@a2ui/markdown-it@0.2.0` on
  `dompurify@3.4.17`.
- Pair catalogs with the protocol version: `createPrometheusA2uiCatalog({ protocolVersion: "1.0" })`.
  A 1.0 surface naming a catalog declared for another version is refused.
- In 1.0 component ids are UAX #31 identifiers (`update_button`, never
  `update-button`); there is no theme; Text `variant` is only `caption` or `body`; an
  action `event` takes no extra properties; response-aware actions are
  `callAgentFunction` calls, not `wantResponse`.
- The root entry renders the basic catalog as Custom Elements, which draw only
  in the browser. Importing it on the server is safe (server rendering emits the
  surface's fallback markup); use `@prometheus-ags/a2ui-react/headless` for
  React-free server work such as exporting surfaces.
- Treat package distribution versions and wire versions as separate axes.

## Architecture rules

```text
Components → A2UI hooks/provider → runtime policy → graph store
```

- Official `MessageProcessor`, schemas, surface/data model, catalog binder,
  and React renderer own protocol behavior.
- Do not implement another JSONL parser, surface model, or schema.
- Renderer components must not call the graph store directly.
- Application code owns runtime creation/disposal and transport delivery.
- Provide deterministic SSR fallback markup; do not claim the official
  renderer itself performs server rendering.

## Catalog and action rules

- Derive implementations from the official catalog.
- Allowlist exact components and functions.
- `openUrl` is excluded by default because navigation is a side effect.
- Start with deny-all action authority.
- Require exact action names and strict context schemas.
- For graph actions, allowlist entity types, action names, and fields.
- Always require application tenant/scope authorization.
- Require out-of-band approval for replace and remove.
- Record action decisions without exposing secrets.
- Never accept agent-supplied approval as authority.

## Required evidence

For package or API claims:

```bash
pnpm run test:a2ui-bridge
pnpm run verify:a2ui-bridge
pnpm run bdd:a2ui-bridge
pnpm --filter @prometheus-ags/a2ui-react run verify:skills
```

`verify:a2ui-bridge` uses packed tarballs and isolated ESM, CommonJS,
NodeNext, and server-render consumers. `verify:a2ui-visual` hash-checks the
built artifact, real browser screenshots, keyboard trace/video, and
accessibility receipt.

Upstream A2UI conformance is `pnpm run verify:a2ui-conformance -- --report <file>`;
the cases it may fail, and exactly where, are listed in
`tests/conformance/a2ui/divergences.json`.

These receipts do not certify A2A conformance, the full agentic showcase,
Flutter/native renderers, documentation deployment, registry authority, or npm
`latest` promotion. Consult `examples/coverage.json` and the release
contract before expanding a claim.


## AG-UI 1.0 delivery and headless entry

- AG-UI peer range is `^1.0.0` (`@ag-ui/core`). Activities are reconciled,
  never replayed: only an explicit `deleteSurface` removes a surface, and the
  producer wins on data conflicts (`onConflict` reports them).
- Ordering rides in activity `content.surfaceVersions` as `{ epoch, version }`
  per surface; `ACTIVITY_DELTA` supports `add /a2ui_operations/-` and `test` on
  the version only.
- User actions carry a UUIDv7 `forwardedProps.a2uiAction.actionId`, reused on
  retry. `buildA2uiForwardedProps` is the single builder.
- `@prometheus-ags/a2ui-react/headless` is the React-free runtime;
  `exportSurfaceMessages` rebuilds a surface and prunes orphans.
