---
name: prometheus-entity-skills
description: >
  Full-stack entity management skill suite for React applications using
  @prometheus-ags/prometheus-entity-management. Covers setup, CRUD screens,
  GraphQL integration, Prisma backend, realtime sync, and performance
  optimization. Use when building entity-driven admin UIs, data management
  dashboards, or any React app with relational entity graphs.
---

# Prometheus Entity Skills

A comprehensive skill suite for building entity-driven React applications with
`@prometheus-ags/prometheus-entity-management`. Each sub-skill targets a specific
layer of the entity stack — from initial setup through CRUD UI, GraphQL wiring,
Prisma backend, realtime sync, and performance optimization.

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

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-graph-init` | [entity-graph-setup](entity-graph-setup/SKILL.md) | Initialize the entity graph in a new or existing project |
| `/entity-graph-detect` | [entity-graph-setup](entity-graph-setup/skills/entity-graph-detect/SKILL.md) | Auto-detect existing entity patterns and suggest migration |
| `/entity-graph-migrate` | [entity-graph-setup](entity-graph-setup/skills/entity-graph-migrate/SKILL.md) | Migrate from ad-hoc state to the entity graph |

### CRUD

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-crud-page` | [entity-graph-crud](entity-graph-crud/SKILL.md) | Full CRUD page with list, create, edit, detail, delete |
| `/entity-crud-form` | [entity-graph-crud](entity-graph-crud/skills/entity-crud-form/SKILL.md) | Form sheets with FieldDescriptor configuration |
| `/entity-crud-table` | [entity-graph-crud](entity-graph-crud/skills/entity-crud-table/SKILL.md) | Table views with column helpers and sorting |
| `/entity-crud-relations` | [entity-graph-crud](entity-graph-crud/skills/entity-crud-relations/SKILL.md) | Entity schema registration and cascade invalidation |

### GraphQL

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-gql-setup` | [entity-graph-graphql](entity-graph-graphql/SKILL.md) | GQLClient and EntityDescriptor setup |
| `/entity-gql-hooks` | [entity-graph-graphql](entity-graph-graphql/skills/entity-gql-hooks/SKILL.md) | Typed GraphQL query/mutation hooks |
| `/entity-gql-subscription` | [entity-graph-graphql](entity-graph-graphql/skills/entity-gql-subscription/SKILL.md) | GraphQL subscription wiring via RealtimeManager |

### Prisma Backend

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-prisma-setup` | [entity-graph-prisma](entity-graph-prisma/SKILL.md) | Analyze schema.prisma and generate entity configs |
| `/entity-prisma-api` | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-api/SKILL.md) | Next.js API routes with Prisma CRUD |
| `/entity-prisma-migrate` | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-migrate/SKILL.md) | Migrate manual Prisma patterns to entity hooks |
| `/entity-prisma-generator` | [entity-graph-prisma](entity-graph-prisma/skills/entity-prisma-generator/SKILL.md) | Prisma generator for entity graph codegen |

### Realtime

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-realtime-setup` | [entity-graph-realtime](entity-graph-realtime/SKILL.md) | RealtimeManager initialization and adapter wiring |
| `/entity-realtime-channel` | [entity-graph-realtime](entity-graph-realtime/skills/entity-realtime-channel/SKILL.md) | Channel subscription configuration |
| `/entity-realtime-local-first` | [entity-graph-realtime](entity-graph-realtime/skills/entity-realtime-local-first/SKILL.md) | ElectricSQL + PGlite local-first sync |

### Performance

| Command | Skill | Purpose |
|---------|-------|---------|
| `/entity-audit` | [entity-graph-optimize](entity-graph-optimize/SKILL.md) | Full CLAUDE.md compliance audit |
| `/entity-perf` | [entity-graph-optimize](entity-graph-optimize/skills/entity-perf/SKILL.md) | Re-render and selector analysis |
| `/entity-gc` | [entity-graph-optimize](entity-graph-optimize/skills/entity-gc/SKILL.md) | Entity eviction and GC strategies |

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
- [Entity Schema Reference](/_shared/references/schemas/entity-schema.md) — registerSchema contract
