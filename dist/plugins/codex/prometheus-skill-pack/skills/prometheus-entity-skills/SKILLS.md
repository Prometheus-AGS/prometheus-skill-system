---
name: prometheus-entity-skills
description: >
  Bundle index for Agent Skills that teach coding agents the Prometheus normalized entity graph
  across its full 3.0 ecosystem — framework-neutral core, React, Svelte, Solid, Alpine, HTMX,
  Web Components, sync, SDL, A2A, A2UI, Tauri, Dart/Flutter, Rust CLI/MCP tooling, and Flint
  fabric contracts. Use for package selection, graph, CRUD, GraphQL, realtime, Prisma,
  performance, Riverpod, or public-ledger work. Load the matching plugin or shared reference,
  then verify agent guidance against the package-specific ledgers under
  prometheus-entity-skills/_shared/references/ after API changes.
license: MIT
metadata:
  bundle: prometheus-entity-management
  library: "@prometheus-ags/prometheus-entity-management"
  spec: "https://agentskills.io/specification"
  progressive_disclosure:
    - "Tier 1: this file + plugin name — pick a plugin"
    - "Tier 2: plugin root SKILL.md — workflow and constraints"
    - "Tier 3: skills/<sub-skill>/SKILL.md — focused playbooks"
    - "Tier 4: references/, agents/, prompts/ — load on demand"
---

# Prometheus entity skills (bundle index)

This directory is the **canonical skill pack** shipped beside the library. It follows the [Agent Skills specification](https://agentskills.io/specification) for leaf skills: each invokable skill lives in its own folder with a **`SKILL.md`** (singular) file, YAML frontmatter (`name`, `description`, …), and optional `scripts/`, `references/`, `assets/`.

**`SKILLS.md` (this file)** is a **bundle catalog** only—it is not a substitute for per-skill `SKILL.md` files. Use it to choose a plugin, map sub-skills, and find shared references.

## Package selection (3.0 ecosystem)

The 3.0 surface is **twelve public npm packages, one Dart package, and two
Rust crates**. Pick by stack; every claim names its evidence gate. Full table
with gates: `prometheus-entity-skills/_shared/references/package-selection.md`.

| Stack | Package | Ledger |
| ----- | ------- | ------ |
| Any (framework-neutral core) | `@prometheus-ags/entity-graph-core` | `core-library-exports.json` |
| React 19 | `@prometheus-ags/prometheus-entity-management` | `library-exports.json` |
| Svelte | `@prometheus-ags/entity-graph-svelte` | `svelte-library-exports.json` |
| Solid | `@prometheus-ags/entity-graph-solid` | `solid-library-exports.json` |
| Alpine.js | `@prometheus-ags/entity-graph-alpine` | `alpine-library-exports.json` |
| HTMX + SSE | `@prometheus-ags/entity-graph-htmx` | `htmx-library-exports.json` |
| Web Components | `@prometheus-ags/entity-graph-web-components` | `web-components-library-exports.json` |
| Local-first sync (Loro/PGlite) | `@prometheus-ags/entity-graph-sync` | `sync-library-exports.json` |
| A2UI / AG-UI agentic UI | `@prometheus-ags/a2ui-react` | `a2ui-library-exports.json` |
| A2A agent server | `@prometheus-ags/entity-graph-a2a` | `a2a-library-exports.json` |
| Tauri 2 desktop/mobile | `@prometheus-ags/entity-graph-tauri` | `tauri-library-exports.json` |
| SDL schema parsing | `@prometheus-ags/entity-graph-sdl` | `sdl-library-exports.json` |
| Dart / Flutter | `entity_graph_flutter` | `dart-library-exports.json`, `dart-devtools-library-exports.json` |
| Rust scaffolding CLI | `entity-graph-cli` | n/a (Cargo) |
| Rust MCP server | `entity-graph-mcp` | n/a (Cargo) |

React v2 → v3 migration guidance lives in
`release/framework-neutral-core.md` (React migration map) and the
`entity-graph-migrate` sub-skill.

## Claude Code plugins and marketplace

Each first-level folder under `prometheus-entity-skills/` with `.claude-plugin/plugin.json` is an installable **plugin**. Paths in `plugin.json` are **relative to the plugin root** (the folder that contains `.claude-plugin/`), per [Claude Code plugin manifests](https://code.claude.com/docs/en/plugin-marketplaces).

Add the marketplace by pointing Claude Code at the directory that contains this file’s sibling `.claude-plugin/marketplace.json` (see [Create and distribute a plugin marketplace](https://code.claude.com/docs/en/plugin-marketplaces)), then install plugins by name, for example:

```text
/plugin install prometheus-entity-graph-crud@prometheus-entity-skills
```

Plugin sources in the marketplace are paths relative to `prometheus-entity-skills/` (for example `./entity-graph-crud`).

## Plugin map

| Plugin directory | `plugin.json` name | Focus |
| ---------------- | ------------------- | ----- |
| `entity-graph-setup/` | `prometheus-entity-graph-setup` | Adopt the library in an existing app; detect legacy data layers; migration plans |
| `entity-graph-crud/` | `prometheus-entity-graph-crud` | CRUD UI, `useEntityCRUD`, tables, forms, relations / `registerSchema` |
| `entity-graph-graphql/` | `prometheus-entity-graph-graphql` | GQL client, descriptors, hooks, subscriptions |
| `entity-graph-realtime/` | `prometheus-entity-graph-realtime` | RealtimeManager, adapters, channels, local-first |
| `entity-graph-prisma/` | `prometheus-entity-graph-prisma` | Prisma mapping, generators, API routes |
| `entity-graph-optimize/` | `prometheus-entity-graph-optimize` | Audits, performance, GC / eviction |

## Sub-skills (nested `skills/*/SKILL.md`)

| Plugin | Sub-skill folders |
| ------ | ----------------- |
| **entity-graph-setup** | `entity-graph-init`, `entity-graph-detect`, `entity-graph-migrate` |
| **entity-graph-crud** | `entity-crud-page`, `entity-crud-form`, `entity-crud-table`, `entity-crud-relations` |
| **entity-graph-graphql** | `entity-gql-setup`, `entity-gql-hooks`, `entity-gql-subscription` |
| **entity-graph-realtime** | `entity-realtime-setup`, `entity-realtime-channel`, `entity-realtime-local-first` |
| **entity-graph-prisma** | `entity-prisma-setup`, `entity-prisma-generator`, `entity-prisma-api`, `entity-prisma-migrate` |
| **entity-graph-optimize** | `entity-audit`, `entity-perf`, `entity-gc` |

Each sub-skill is a normal Agent Skill directory with its own `SKILL.md` and must keep `name` in frontmatter aligned with the folder name per agentskills.io rules.

## Shared references (monorepo paths)

All paths below are relative to the **repository root** of `prometheus-entity-management`:

| Path | Role |
| ---- | ---- |
| `prometheus-entity-skills/_shared/references/library-exports.json` | Entry-keyed runtime exports for React `.`, `./devtools`, and `./devtools/auto`; must match `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/core-library-exports.json` | Runtime exports for `entity-graph-core`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/svelte-library-exports.json` | Runtime exports for `entity-graph-svelte`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/solid-library-exports.json` | Runtime exports for `entity-graph-solid`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/alpine-library-exports.json` | Runtime exports for `entity-graph-alpine`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/htmx-library-exports.json` | Runtime exports for `entity-graph-htmx`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/web-components-library-exports.json` | Runtime exports for `entity-graph-web-components`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/sdl-library-exports.json` | Runtime exports for `entity-graph-sdl`; verified by `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/package-selection.md` | Which 3.0 artifact (12 npm + Dart + Rust crates) for which stack, with evidence gates |
| `prometheus-entity-skills/_shared/references/framework-bindings.md` | Svelte/Solid/Alpine/HTMX/Web Components singleton-facade guidance |
| `prometheus-entity-skills/_shared/references/sdl-and-rust-tooling.md` | SDL package, Rust scaffolding CLI, and Rust MCP server guidance |
| `prometheus-entity-skills/_shared/references/examples-gallery.md` | Certified examples, their verifier commands, and evidence boundaries |
| `prometheus-entity-skills/_shared/references/ecosystem-claims.json` | Machine-readable claim → evidence map for every supported binding/integration |
| `docs/flint-integration.md` | Flint fabric security, realtime, and provisioning contracts (owned by `v3-flint-portable-contracts`) |
| `prometheus-entity-skills/_shared/references/sync-library-exports.json` | Sorted runtime exports for the companion `entity-graph-sync` package; verified by its package skill gate |
| `prometheus-entity-skills/_shared/references/a2ui-library-exports.json` | Root and `./ag-ui` runtime exports for `a2ui-react`; verified by its package skill gate |
| `prometheus-entity-skills/_shared/references/a2a-library-exports.json` | Official root and `./legacy` runtime exports for `entity-graph-a2a`; verified by its package skill gate |
| `prometheus-entity-skills/_shared/references/dart-library-exports.json` | Public declarations from the canonical Dart barrel and generated Riverpod part; verified against source |
| `prometheus-entity-skills/_shared/references/dart-devtools-library-exports.json` | Public declarations from the optional Dart DevTools entry and its controller parts; verified against source |
| `prometheus-entity-skills/_shared/references/tauri-library-exports.json` | Runtime and declaration exports for `entity-graph-tauri`; verified against its built ESM and declaration entry points |
| `prometheus-entity-skills/_shared/references/library-api.md` | Human-oriented API notes for agents |
| `prometheus-entity-skills/_shared/references/sync-persistence-path.md` | Certified PGlite/Loro persistence, convergence, reconnect, API, and evidence boundary |
| `prometheus-entity-skills/_shared/references/a2ui-protocol-bridge.md` | Official v0.9.1/AG-UI boundary, graph action policy, migration, and evidence limits |
| `prometheus-entity-skills/_shared/references/a2a-conformance-agent.md` | Official A2A v1 JSON-RPC lifecycle, application authority, TCK scope, migration, and evidence limits |
| `prometheus-entity-skills/_shared/references/dart-graph-riverpod.md` | Canonical Dart graph/Riverpod architecture, API, commands, and evidence limits |
| `prometheus-entity-skills/_shared/references/tauri-mobile-plugin.md` | Tauri v2 bindings, capabilities, in-memory mirror, desktop/packed proof, and Android/iOS evidence boundary |
| `prometheus-entity-skills/_shared/references/devtools-entity-inspection.md` | Optional core `./devtools` entity/view/relationship projections, preview/restore semantics, value boundary, fixtures, and evidence limits |
| `prometheus-entity-skills/_shared/references/devtools-time-travel.md` | Optional core `./devtools` controller-owned snapshot retention, rewind/live, expiry, import, compatibility, fixture, and evidence boundaries |
| `prometheus-entity-skills/_shared/references/devtools-react-inspector.md` | Optional React `./devtools` and `./devtools/auto` activation, inspector workflows, bundle/value boundaries, public exports, and evidence limits |
| `prometheus-entity-skills/_shared/references/devtools-flutter-controller.md` | Optional Dart `devtools.dart` controller, Riverpod view instrumentation, VM-service routing, value/security limits, public exports, and evidence boundary |
| `prometheus-entity-skills/_shared/references/release-candidate-pipeline.md` | Contract-derived RC manifest, non-mutating rehearsal, OIDC staging, protected tags, and restart recovery boundary |
| `prometheus-entity-skills/_shared/references/v3-release-contract.md` | Entry point for authoritative 3.0 artifact, compatibility, maturity, promotion, and recovery rules |
| `release/ci-baseline.md` | Implemented hermetic main-CI guarantees and explicit certification limits |
| `release/package-contracts.md` | Implemented twelve-tarball module/type/payload gate and explicit certification limits |
| `release/framework-neutral-core.md` | Implemented vanilla-core boundary, React migration map, and packed non-React gate |
| `release/binding-singleton-contract.md` | Implemented required-peer policy and packed six-binding singleton gate |
| `examples/shared/README.md` | Implemented deterministic shared example domain, scenarios, and evidence boundary |
| `prometheus-entity-skills/_shared/references/example-coverage-contract.md` | Agent guidance for applying and extending the shared example contract |
| `prometheus-entity-skills/_shared/references/vite-react19-example.md` | Implemented React 19/Vite 8 architecture, scenario gate, optional-peer loader, and evidence boundary |
| `release/dependency-policy.json` | Machine-readable compatible-current dependency holds and revisit owners |
| `security/advisory-policy.json` | Machine-readable critical/high production advisory dispositions |
| `prometheus-entity-skills/_shared/references/architecture-rules.md` | Non-negotiable layering (Components → Hooks → Stores) |
| `prometheus-entity-skills/_shared/references/branding.md` | Example UI tokens for generated demos |
| `prometheus-entity-skills/_shared/references/schemas/*.schema.json` | JSON Schemas for manifests and filters |

Regenerate the export ledger after changing `src/index.ts` exports:

```bash
pnpm run refresh:exports
```

## Non-negotiable architecture (summary)

- **Components** must not call the graph store directly; **hooks** orchestrate store methods; **stores/adapters** own all I/O (fetch, mutation, realtime). No skill may prescribe a hook or component calling `fetch`/an API client directly.
- **Lists store entity IDs only**; entity data lives once in the graph.
- Skills that generate code must follow `AGENTS.md` / `CLAUDE.md` in the library repo.
- Skills that claim 3.0 readiness must use `release/v3-release-contract.json`, consult the dependency/advisory policies for currentness or security claims, require packed-candidate evidence for npm module/type claims, require the framework-neutral verifier for React-free core claims, require the binding verifier for six-binding singleton claims, require `pnpm run verify:example-coverage` for shared scenario claims, require `pnpm run bdd:vite-react19` for the implemented React/Vite showcase, require the exact official SDK/TCK and explicit application authority for A2A claims, keep Tauri desktop/packed/mobile evidence distinct, keep Flint default-lane evidence fixture-backed (live interop is env-gated via `verify:flint-contracts`), and load `release-candidate-pipeline.md` for RC or recovery claims. Every binding/integration claim must map to a consumer fixture or example in `ecosystem-claims.json`. An implemented showcase proves only its declared evidence boundary and never authorizes publication by itself.

## Validation

- **Library ↔ ledgers:** `pnpm run verify:skills` after building the npm packages; verifies all twelve public npm package ledgers (react, core, sync, A2UI, A2A, Tauri, Svelte, Solid, Alpine, HTMX, Web Components, SDL) plus the Dart source declaration ledger.
- **Public snippets:** `pnpm run verify:skills-snippets` compiles every public ```ts/tsx snippet in this pack against PACKED packages.
- **Ecosystem gate:** `pnpm run verify:skills-ecosystem` runs ledgers, snippets, and the release checks (referenced paths exist, claims backed, data-flow language).
- **Leaf skills:** Prefer the official validator from the Agent Skills ecosystem when packaging for external marketplaces (`skills-ref validate ./path` per [agentskills.io](https://agentskills.io/specification)).
