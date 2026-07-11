---
type: Reference
id: flint-forge-g4-graphql-subscription-wiring-scope-decision
title: Flint Forge G4 GraphQL Subscription Wiring Scope Decision
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T14:27:30Z` - **Status:** `in_progress` - **Progress:** changes `7/9` - **Related phase status:** [Flint Forge p3 Auth RLS Keto Phas"
tags:
- flint-forge
- graphql-subscriptions
- postgres-rls
- fdb-realtime
- change-stream
- auth-rls
- integration-scope
links:
- flint-forge-p3-auth-rls-keto-phase-status
sources:
- stdin
timestamp: 2026-07-03T14:34:59.312776+00:00
created_at: 2026-07-03T14:34:59.312776+00:00
updated_at: 2026-07-03T14:34:59.312776+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T14:27:30Z`
- **Status:** `in_progress`
- **Progress:** changes `7/9`
- **Related phase status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)

## Phase Gate

All four authentication and authorization layers must be live end-to-end:

1. A real flint-gate JWT causes a real Postgres RLS row filter.
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
  - Range header pagination.
  - Column-name safety validation.
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for Query/Mutation under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws` pulling from `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema ∪ subscription SDL.
- **G5 — Subscription RLS enforcement**
  - For each `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivery.
  - This is mandatory WAL-bypass protection.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state`.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated by service token.
  - Reconnect loop.
  - Fan-out to subscriber streams.

## Dependencies Delivered from Phase 2

- `CompiledState` and `DatabaseModel` — delivered in `p2-c003`.
- `RestCompiler` route registration — delivered in `p2-c004`; handler bodies remain Phase 3 scope.
- `StateManager` + `ArcSwap` hot-reload — delivered in `p2-c005`.
- `fdb-auth` JWT verify → `RlsContext` — delivered in `p2-c001`.
- `SET LOCAL` RLS propagation — delivered in `p2-c002`.

## G4 Pre-flight Check

Before starting GraphQL hybrid work, verify OQ-3 against the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Session Findings

A review of G4 showed it is more complete than expected. Current implementation state:

| Component | State |
|---|---|
| WebSocket / `graphql-transport-ws` handler mounted at `GET /graphql` | Done via `graphql_ws_handler` and `async_graphql_axum::GraphQLWebSocket` |
| RLS-filtering use case `subscribe_rls_filtered` | Done; performs per-event re-query as required |
| Introspection merge of `pg_graphql` ∪ subscription SDL | Done and tested |
| `GraphQlCompiler` subscription schema shape | Done |
| Subscription field stream body | Stub; yields `stream::empty()` and never calls the use case |
| Live change source `FabricChangeSource::watch()` | Blocked on OQ-FRF-1 |

## External Blocker: OQ-FRF-1

`fdb-realtime::watch()` is intentionally documented as a stub because the upstream `flint-realtime-fabric` `WatchEntityType` gRPC RPC has not shipped and its signature is not finalized.

Implementation must not guess this API. The live subscription data path currently terminates at this external RPC boundary.

## Recommended Scope Decision

Proceed with **wire-seam-only** for G4:

- Thread a stream factory from `Quarry` into `GraphQlCompiler`.
- Keep the factory typed in ports/domain terms to avoid layering violations.
- Ensure each subscription field calls `subscribe_rls_filtered` instead of returning `stream::empty()`.
- Accept that no live events will flow until the FRF RPC is available.

Do **not** include either of these in the same change:

- An in-process Postgres `LISTEN`-based `ChangeStreamSource` workaround.
- A guessed proto/API for the unfinalized `WatchEntityType` RPC.

Rationale: wire-seam-only closes every in-repo connection that can be made without inventing external API surface or expanding scope into a new adapter.

## Required Security Behavior

The seam must thread `who: RlsContext` through `graphql-transport-ws` connection-init into the subscription resolver and must **fail closed**. RLS is security-critical; unauthenticated or invalid connection-init state must not produce a subscription stream.

Planned implementation path:

1. Add compiler factory parameter.
2. Thread through `CompiledState` / `do_compile`.
3. Thread through `StateManager` constructor.
4. Add gateway factory wiring.
5. Add fail-closed `on_connection_init` RLS handling.
6. Run one `cargo check` as the first integration checkpoint.

## Session Operations

- Pushed branch `docs/dev-management-integration-first`.
- Opened PR #1: `https://github.com/Know-Me-Tools/flint-forge/pull/1`.
- PR includes docs and build config; wiki `index.md` linter update was folded in by amend.
- Switched back to `main` before investigating G4 state.
- Scratchpad plan recorded in `g4-seam-plan.md`.
- Test-wait budget: `0/3` spent.

# Citations

1. [1] stdin