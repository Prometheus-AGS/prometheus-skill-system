---
type: Reference
id: flint-forge-p3-c019-postgrest-engine-read-write-progress
title: Flint Forge p3-c019 PostgREST Engine Read/Write Progress
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T15:47:56Z` - **Source phase record:** `manual:Flint Forge/p3-auth-rls-keto` - **Status:** `in_progress` - **Progress:** changes `7/"
tags:
- flint-forge
- postgrest
- fdb-query
- postgres-rls
- sql-injection
- auth-rls
- graphql-subscriptions
links:
- flint-forge-p3-auth-rls-keto-phase-status
- flint-forge-p3-c019-postgrest-query-engine-foundation
- flint-forge-g4-graphql-subscription-wiring-scope-decision
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T15:51:19.643859+00:00
created_at: 2026-07-03T15:51:19.643859+00:00
updated_at: 2026-07-03T15:51:19.643859+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T15:47:56Z`
- **Source phase record:** `manual:Flint Forge/p3-auth-rls-keto`
- **Status:** `in_progress`
- **Progress:** changes `7/9`
- **Branch:** `feat/p3-c019-postgrest-query-engine`
- **Related phase status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)
- **Foundation:** [Flint Forge p3-c019 PostgREST Query Engine Foundation](/flint-forge-p3-c019-postgrest-query-engine-foundation.md)

## Phase Gate

All four authentication and authorization layers must be live end-to-end:

1. A real `flint-gate` JWT causes a real Postgres RLS row filter.
2. A Keto relation check gates mutations.
3. A Cedar policy controls capability-level access.
4. Zero plaintext credentials appear in logs or tracing spans.
5. CRUD handler bodies execute parameterized SQL.

## Phase Goals

- **G1 — `forge-policy`: Cedar policy evaluation crate**
  - `PolicyEngine::evaluate(principal, action, resource, context)` returns allow/deny.
  - Policy bundles loaded from `flint_meta.cedar_policies`.
- **G2 — Keto coarse relationship checks**
  - Enforced at subscribe-time and mutation-time.
  - `KetoCacheClient` caches relation tuples with TTL.
  - Cache invalidated on Keto webhook.
  - Integrated into `fdb-app` use cases.
- **G3 — Full RLS CRUD handler bodies in `RestCompiler`**
  - `handle_list`, `handle_insert`, `handle_update`, `handle_delete`.
  - Parameterized SQL.
  - Filter operator dispatch: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `in`, `is`, `cs`, `cd`.
  - `Range` header pagination.
  - Column-name safety validation.
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for Query/Mutation under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws` using `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema with subscription SDL.
  - Scope decision tracked in [Flint Forge G4 GraphQL Subscription Wiring Scope Decision](/flint-forge-g4-graphql-subscription-wiring-scope-decision.md).
- **G5 — Subscription RLS enforcement**
  - For every `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivery.
  - WAL-bypass protection is non-negotiable.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state`.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter for `flint-realtime-fabric` `WatchEntityType` RPC.
  - Service-token authentication.
  - Reconnect loop and fan-out to subscriber streams.

## Dependencies from Phase 2

- `CompiledState` and `DatabaseModel` delivered in `p2-c003`.
- `RestCompiler` route registration delivered in `p2-c004`; handler bodies are Phase 3 deliverables.
- `StateManager` plus `ArcSwap` hot reload delivered in `p2-c005`.
- `fdb-auth` JWT verification to `RlsContext` delivered in `p2-c001`.
- `SET LOCAL` RLS propagation delivered in `p2-c002`.

## Pre-flight Requirement for G4

Before starting GraphQL hybrid work, verify `pg_graphql` availability in the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Completed in This Session

The `fdb-query` PostgREST translator moved from scaffold to complete read/write engine and is now live through `PgRest::execute`.

| Task | Delivered |
|---|---|
| T4 | `filter.rs`: `and`/`or`/`not` trees, arbitrary nesting, parameterized rendering |
| T5 | `clause.rs`: `select` rename/cast/JSON paths, multi-column `order` with direction/nulls, `limit`, `offset`, `Range`, count strategy |
| T4/T5 | `plan.rs`: `parse_select_request` implements full read grammar including `not.`, `op(any)`, `op(all)`, nested logical groups, `Range` override, `Prefer: count` into `SELECT` |
| T6 | `mutation.rs`: bulk `INSERT`, `UPSERT` with `ON CONFLICT DO UPDATE/NOTHING`, `UPDATE`, `DELETE`, and `Prefer` directives |
| T8 | `PgRest::execute`: `RestQuery -> fdb-query -> (sql, params) -> RLS-scoped exec -> RestResult`; previous `todo!()` retired |

Previously completed T1-T3 supplied the safety layer and 21 operators. With this session, the full 21-operator PostgREST surface, logical trees, and writes are implemented and unit-tested.

## Security Properties Preserved

- Translator remains pure and does **not** receive `RlsContext`; RLS enforcement stays in executor `SET LOCAL` GUC propagation.
- Every SQL identifier is validated.
- Every value is bound as a positional parameter (`$n`).
- JSON keys are escaped.
- CRUD execution remains compatible with the phase gate requirement for parameterized SQL.

## Verification

Executed successfully:

```bash
cargo test -p fdb-query -p fdb-postgres
cargo clippy … -- -D warnings
cargo check --workspace
```

Results:

- `cargo test -p fdb-query -p fdb-postgres`: 73 passing tests.
- Clippy: clean with `-D warnings`.
- Workspace check: clean.
- Changes committed on `feat/p3-c019-postgrest-query-engine`.

## Impact on Subscription RLS Work

The concrete effect of T8 is that the p3-G4 subscription RLS re-query path from PR #2 is no longer blocked by a `todo!()`. Once a change source feeds events, it can perform a real RLS-scoped re-query before delivery. The in-process `LISTEN` source remains a separate agreed split-out change.

## Remaining Work in `p3-c019`

### T7: Wire `fdb-reflection` REST handlers through `fdb-query`

- Route `fdb-reflection` REST handlers through `fdb-query`.
- Retire duplicate `filters::build_where`.
- Preserve existing REST test coverage.
- Rationale for deferral: this is careful surgery across three working, separately-tested handler bodies; it provides the anti-drift/DRY win but should not be rushed.

### Parity pass

- Resource embedding:
  - FK joins.
  - `!fk`.
  - `!inner`.
  - Spread.
  - Nested embedding.
- Full-text search variants:
  - `fts`.
  - `plfts`.
  - `phfts`.
  - `wfts`.
- Edge-case hardening.

## PR Status and Next Steps

- Not pushed; no PR opened yet because `p3-c019` is not complete.
- Plan: open the `p3-c019` PR after T7 lands, or earlier as a draft if visibility is needed.
- PR #2 for the G4 seam is still awaiting review/merge.
- Next implementation step: complete T7, verify existing REST tests remain green, then begin the parity pass with resource embedding first.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto