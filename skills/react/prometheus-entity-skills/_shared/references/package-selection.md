# Package selection — which 3.0 artifact for which stack

All npm packages are `3.0.0-rc.x` prereleases until `v3-stable-publication`
(human-gated) completes. Every claim below names its evidence gate; run the
gate before repeating the claim in generated docs or client guidance.

## npm packages (12 public)

| Package | Use it for | Evidence gate |
| ------- | ---------- | ------------- |
| `@prometheus-ags/entity-graph-core` | Framework-neutral graph, engine, view layer, realtime manager, adapters (Flint, Surreal, ElectricSQL, PGlite, Tauri SQL) | `pnpm run verify:framework-neutral-core` |
| `@prometheus-ags/prometheus-entity-management` | React 19 bindings: hooks, CRUD, UI components, GraphQL, Prisma helpers | `pnpm run verify:vite-react19` |
| `@prometheus-ags/entity-graph-sync` | Local-first sync: Loro CRDT provider, PGlite persistence, convergence | `pnpm run verify:sync-persistence` |
| `@prometheus-ags/entity-graph-svelte` | Svelte stores over the graph (`initEntityGraph`, `createEntityStore`, `createEntityList`) | `pnpm run verify:binding-singletons` |
| `@prometheus-ags/entity-graph-solid` | Solid signals (`createGraphStore`, `createEntity`, `createEntityList`) | `pnpm run verify:binding-singletons` |
| `@prometheus-ags/entity-graph-alpine` | Alpine.js plugin (`createEntityGraphPlugin`, entity/list bindings) | `pnpm run verify:binding-singletons` |
| `@prometheus-ags/entity-graph-htmx` | HTMX + SSE server rendering (`createHtmxSseServer`, `renderFragment`, OOB) | `pnpm run verify:binding-singletons` |
| `@prometheus-ags/entity-graph-web-components` | Framework-agnostic `<entity-list/detail/form>` elements + controllers | `pnpm run verify:binding-singletons` |
| `@prometheus-ags/a2ui-react` | Official A2UI v0.9.1 surfaces + AG-UI chat entry points | `pnpm run verify:a2ui-bridge` |
| `@prometheus-ags/entity-graph-a2a` | A2A v1 JSON-RPC agent server with graph policy authority | `pnpm run verify:a2a-conformance` |
| `@prometheus-ags/entity-graph-tauri` | Tauri 2 plugin bindings, commands, snapshot lifecycle | `pnpm run verify:tauri-plugin` |
| `@prometheus-ags/entity-graph-sdl` | SDL JSON schema parsing/validation (`parseSdl`, `parseSdlJson`) | `pnpm --filter @prometheus-ags/entity-graph-sdl test` |

## Dart / Flutter

| Package | Use it for | Evidence gate |
| ------- | ---------- | ------------- |
| `entity_graph_flutter` (`packages/entity_graph_flutter`) | Canonical Dart graph + Riverpod providers; repository-source optional DevTools controller/VM-service bridge | static contracts: `pnpm run verify:dart-graph-riverpod` + `pnpm run verify:dart-exports`; assembled controller: `pnpm run verify:devtools-flutter-controller`; assembled mobile/A2UI 1.0: `pnpm run verify:flutter-a2ui-v1 -- --report <file>` |

## Rust crates

| Crate | Use it for | Evidence gate |
| ----- | ---------- | ------------- |
| `entity-graph-cli` (`packages/entity-graph-cli`) | Scaffold `schema.json` (`init`), generate TypeScript types + transport stubs (`generate`) | `cargo test --manifest-path packages/entity-graph-cli/Cargo.toml` |
| `entity-graph-mcp` (`packages/entity-graph-mcp`) | Expose the graph as MCP resources/tools over stdio or Streamable HTTP | `cargo test --manifest-path packages/entity-graph-mcp/Cargo.toml` |

## Rules

- One graph per application: bindings are singleton facades over
  `entity-graph-core`; the six-binding packed contract is
  `pnpm run verify:binding-singletons`.
- Data flow: components use hooks/controllers; hooks orchestrate store
  methods; stores/adapters own all I/O. Never write fetch logic into a
  component or a hook body.
- Flint fabric integration (security, realtime, provisioning) is documented in
  `docs/flint-integration.md`; its gate is `pnpm run verify:flint-contracts`.
- Examples that back these claims live in `examples/`; see
  `prometheus-entity-skills/_shared/references/examples-gallery.md`.
