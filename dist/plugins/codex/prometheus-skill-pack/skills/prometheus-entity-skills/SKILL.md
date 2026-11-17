---
license: MIT
name: prometheus-entity-skills
description: >
  Full-stack entity management skill suite for the Prometheus React ecosystem
  plus its canonical Dart/Flutter companion and Tauri v2 plugin. Covers setup,
  CRUD screens, GraphQL, Prisma, realtime, performance, verified Dart
  graph/Riverpod architecture, Tauri IPC, generated bindings, capabilities,
  and native evidence boundaries. Use when building entity-driven applications
  or synchronizing agent guidance with JavaScript, Flutter, and Tauri contracts.
metadata:
  version: '1.0.0'
  tags: "react, typescript, dart, flutter, riverpod, tauri, entity-management"
---

# Prometheus Entity Skills

A comprehensive skill suite for building entity-driven React applications with
`@prometheus-ags/prometheus-entity-management`, plus shared contract guidance
for the canonical `entity_graph_flutter` companion. The current invokable
sub-skills target the React stack; Flutter work must load the Dart reference
instead of translating React hooks literally.

## Architecture

All sub-skills enforce the library's canonical data flow:

```
Components → Hooks → Stores → APIs
```

- **Components** compose UI only — no direct store or API access
- **Hooks** (`useEntityCRUD`, `useEntityView`, etc.) mediate all data operations
- **Stores** (Zustand entity graph) are the single source of truth
- **APIs** handle server communication and are called only through hooks

## Sub-Skills

### Setup

| Command                 | Skill                                                                         | Purpose                                                    |
| ----------------------- | ----------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `/entity-graph-init`    | [entity-graph-setup](entity-graph-setup/SKILL.md)                             | Initialize the entity graph in a new or existing project   |
| `/entity-graph-detect`  | [entity-graph-setup](entity-graph-setup/skills/entity-graph-detect/SKILL.md)  | Auto-detect existing entity patterns and suggest migration |
| `/entity-graph-migrate` | [entity-graph-setup](entity-graph-setup/skills/entity-graph-migrate/SKILL.md) | Migrate from ad-hoc state to the entity graph              |

### CRUD

| Command                  | Skill                                                                        | Purpose                                                |
| ------------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------------ |
| `/entity-crud-page`      | [entity-graph-crud](entity-graph-crud/SKILL.md)                              | Full CRUD page with list, create, edit, detail, delete |
| `/entity-crud-form`      | [entity-graph-crud](entity-graph-crud/skills/entity-crud-form/SKILL.md)      | Form sheets with FieldDescriptor configuration         |
| `/entity-crud-table`     | [entity-graph-crud](entity-graph-crud/skills/entity-crud-table/SKILL.md)     | Table views with column helpers and sorting            |
| `/entity-crud-relations` | [entity-graph-crud](entity-graph-crud/skills/entity-crud-relations/SKILL.md) | Entity schema registration and cascade invalidation    |

### GraphQL

| Command                    | Skill                                                                                | Purpose                                         |
| -------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------- |
| `/entity-gql-setup`        | [entity-graph-graphql](entity-graph-graphql/SKILL.md)                                | GQLClient and EntityDescriptor setup            |
| `/entity-gql-hooks`        | [entity-graph-graphql](entity-graph-graphql/skills/entity-gql-hooks/SKILL.md)        | Typed GraphQL query/mutation hooks              |
| `/entity-gql-subscription` | [entity-graph-graphql](entity-graph-graphql/skills/entity-gql-subscription/SKILL.md) | GraphQL subscription wiring via RealtimeManager |

### Prisma Backend

| Command                    | Skill                                                                              | Purpose                                           |
| -------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------- |
| `/entity-prisma-setup`     | [entity-graph-prisma](entity-graph-prisma/SKILL.md)                                | Analyze schema.prisma and generate entity configs |
| `/entity-prisma-api`       | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-api/SKILL.md)       | Next.js API routes with Prisma CRUD               |
| `/entity-prisma-migrate`   | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-migrate/SKILL.md)   | Migrate manual Prisma patterns to entity hooks    |
| `/entity-prisma-generator` | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-generator/SKILL.md) | Prisma generator for entity graph codegen         |

### Realtime

| Command                        | Skill                                                                                      | Purpose                                           |
| ------------------------------ | ------------------------------------------------------------------------------------------ | ------------------------------------------------- |
| `/entity-realtime-setup`       | [entity-graph-realtime](entity-graph-realtime/SKILL.md)                                    | RealtimeManager initialization and adapter wiring |
| `/entity-realtime-channel`     | [entity-graph-realtime](entity-graph-realtime/skills/entity-realtime-channel/SKILL.md)     | Channel subscription configuration                |
| `/entity-realtime-local-first` | [entity-graph-realtime](entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md) | ElectricSQL + PGlite local-first sync             |

### Performance

| Command         | Skill                                                                      | Purpose                           |
| --------------- | -------------------------------------------------------------------------- | --------------------------------- |
| `/entity-audit` | [entity-graph-optimize](entity-graph-optimize/SKILL.md)                    | Full CLAUDE.md compliance audit   |
| `/entity-perf`  | [entity-graph-optimize](entity-graph-optimize/skills/entity-perf/SKILL.md) | Re-render and selector analysis   |
| `/entity-gc`    | [entity-graph-optimize](entity-graph-optimize/skills/entity-gc/SKILL.md)   | Entity eviction and GC strategies |

## Typical Workflow

1. `/entity-graph-init` — Set up the entity graph store
2. `/entity-prisma-setup` — Generate entity configs from Prisma schema
3. `/entity-crud-page` — Scaffold CRUD pages per entity
4. `/entity-gql-setup` — Wire GraphQL if applicable
5. `/entity-realtime-setup` — Add realtime sync if needed
6. `/entity-audit` — Verify architecture compliance

## Shared References

Cross-cutting schemas and patterns used by all sub-skills:

- [Audit Checklist](/_shared/references/schemas/audit-checklist.md) — Full compliance checklist
- [Framework-neutral core contract](../release/framework-neutral-core.md) — Vanilla core, React binding, migration aliases, and packed certification
- [Binding singleton contract](../release/binding-singleton-contract.md) — Required core peers, fixed releases, and six-binding packed behavior
- [Shared example contract](../examples/shared/README.md) — Deterministic cross-showcase fixtures, scenarios, verification, and evidence limits
- [React 19/Vite 8 showcase](_shared/references/vite-react19-example.md) — Implemented architecture, browser scenarios, optional-peer loading, and evidence boundary
- [Certified sync path](_shared/references/sync-persistence-path.md) — PGlite durability, Loro convergence, reconnect, packed consumers, and evidence limits
- [Official A2UI bridge](_shared/references/a2ui-protocol-bridge.md) — A2UI 1.0 and v0.9.1 rendering, default-deny graph actions, AG-UI 1.0, and certification limits
- [A2A conformance agent](_shared/references/a2a-conformance-agent.md) — official v1 JSON-RPC lifecycle, application authority, TCK scope, and alpha migration
- [Agentic A2A/A2UI example](_shared/references/agentic-a2ui-example.md) — safe end-to-end composition, exact action authority, keyless fixtures, and showcase evidence limits
- [Flutter source provenance](_shared/references/flutter-source-provenance.md) — licensed filtered history, sole Dart owner, non-public import, and blocked publication claims
- [Dart graph and Riverpod 3](_shared/references/dart-graph-riverpod.md) — canonical graph ownership, generated providers, views, optimistic rollback, retry, transports, API ledger, and evidence limits
- [Flutter DevTools controller](_shared/references/devtools-flutter-controller.md) — optional tooling entry, per-graph ownership, VM-service routing, value/security limits, API ledger, and assembled evidence
- [Flutter/Riverpod/A2UI showcase](_shared/references/flutter-riverpod-a2ui-example.md) — complete app composition, A2UI 1.0 through `a2ui_flutter`, the action-policy boundary, and the simulator acceptance gate
- [Tauri desktop/mobile plugin](_shared/references/tauri-mobile-plugin.md) — generated bindings, least-privilege capabilities, in-memory mirror ownership, packed host proof, and mobile evidence limits
- [Universal Tauri application](_shared/references/tauri-universal-example.md) — shared React/Tauri architecture, durable queue, deep-link/capability policy, focused checks, and pending platform/visual gates
- [Flint portable contracts](_shared/references/flint-portable-contracts.md) — structural realtime client, issuer/tenant/key/JWKS rules, immutable live lane, and external Forge provisioning boundary
- [Recoverable release-candidate pipeline](_shared/references/release-candidate-pipeline.md) — contract-derived artifacts, non-mutating rehearsal, OIDC staging boundary, protected tags, and restart recovery
- [Entity Schema Reference](/_shared/references/schemas/entity-schema.md) — registerSchema contract
- [3.0 Release Contract](_shared/references/v3-release-contract.md) — authoritative packaging, compatibility, maturity, promotion, and recovery rules

For Flutter source-lineage or adaptation claims, load the Flutter source provenance reference and require `pnpm run verify:flutter-source-provenance`; provenance never certifies the Dart runtime, rendered Flutter, or publication.

For Dart graph, Riverpod, Flutter-library, optional FFI, or public Dart API
claims, load the Dart graph reference and require the static
`pnpm run verify:dart-graph-riverpod` contract plus `pnpm run
verify:dart-exports`; neither is behavioral test evidence. For the optional
repository-source DevTools entry, also load the Flutter DevTools controller
reference and require the full assembled `pnpm run
verify:devtools-flutter-controller` acceptance flow. Keep its
controller/VM-service evidence separate from the pending extension UI and from
the published pub.dev `3.0.1` payload.

For the complete Flutter/Riverpod/A2UI application, load the Flutter showcase
reference too. Preserve the one-graph/provider/transport layering and atomic
default-deny A2UI boundary. Analyzer, unit, widget, provider, golden, and other
partial checks are not test evidence; require the complete assembled
host/device acceptance receipt for behavioral claims.

For Tauri plugin, IPC, capability, generated-binding, or Android/iOS claims,
load the Tauri reference and require `pnpm run verify:tauri-plugin` plus the
Tauri package `verify:skills` gate. Keep `entity-graph-tauri:default` read-only,
do not describe the in-memory snapshot mirror as durable SQLite, and never use
desktop or packed-source proof as Android/iOS runtime certification.

For the complete universal Tauri application, also load the universal example
reference. Preserve one React frontend, one canonical normalized graph, the
component-to-hook-to-store-to-service boundary, SQL-owned durability, and the
default-deny deep-link/capability policy. Focused unit, MockRuntime, or source
checks do not authorize browser visual, desktop bundle, Android/iOS application,
registry, or stable-release claims.

For React 19/Vite 8 showcase or React-first RC readiness claims, load the React
showcase reference and require `pnpm run bdd:vite-react19`. Preserve its
source-workspace evidence boundary; for installability or registry claims also
require new packed-candidate and immutable RC-rehearsal receipts.

For Flint realtime, authentication, JWKS, or Forge provisioning claims, load
the Flint portable-contract reference and require
`pnpm run verify:flint-contracts`. Keep service-role material out of client
code, do not generalize RSA compatibility to EC, and do not claim a Forge
adapter. The manual live lane proves only its exact pinned source revisions.

When a task makes a 3.0 compatibility or publication claim, load the 3.0 release contract reference first. For dependency-currentness or vulnerability claims, also follow its links to `release/dependency-policy.json` and `security/advisory-policy.json`. For npm module or type claims, require the packed-candidate procedure in `release/package-contracts.md`; workspace aliases are not release evidence. For cross-binding singleton claims, require `pnpm run verify:binding-singletons`. For PGlite/Loro durability, convergence, or reconnect claims, read the certified sync reference and require `pnpm run test:sync-persistence` plus `pnpm run verify:sync-persistence`; persistence, CRDT merge, graph projection, and transport recovery are separate receipts. For official A2UI work, read the A2UI bridge reference, use the package root for official v0.9.1 only, move alpha chat/state APIs to `./ag-ui`, and require explicit application authority after protocol validation. For A2A work, read the A2A conformance reference, use the package root for official v1 JSON-RPC and `./legacy` only for pre-v3 slash-method migration, require `pnpm run test:a2a-conformance`, `pnpm run verify:a2a-conformance`, and the pinned TCK receipt, and never infer application authority from protocol validity. For Tauri work, preserve the four separate gates for public declarations, Rust-derived bindings, webview capability authority, and platform-native execution. For example-coverage claims, read the shared contract and require `pnpm run verify:example-coverage`; semantic evidence is a flight plan, not framework, browser, device, accessibility, or visual certification. Treat `planned` entries in `examples/coverage.json` as work remaining. An `implemented` showcase proves only the commands and paths declared on that entry; it does not automatically prove packed installation, another platform, or publication authority.

For an end-to-end A2A-to-A2UI application, load the agentic example reference
in addition to both protocol references. Keep the deterministic reference
agent keyless, make external endpoints explicit opt-ins, route rendered actions
through exact schemas plus application authorization and human approval, and
require `pnpm run verify:agentic-a2ui` before claiming rendered,
accessible, or visual showcase evidence.

For release-candidate planning, rehearsal, staging, or recovery, load the
release-candidate pipeline reference and require the non-mutating release
pipeline tests. Do not infer npm trusted-publisher configuration, GitHub
environment approval, registry mutation, `latest`, or stable 3.0.0 from a
passing local gate.
