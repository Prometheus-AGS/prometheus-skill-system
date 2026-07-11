---
type: Reference
id: flint-forge-p3-auth-rls-keto-goals-and-p3-c020-relaunch
title: Flint Forge p3 Auth/RLS/Keto Goals and p3-c020 Relaunch
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T17:55:07Z` - **Position:** `p3-auth-rls-keto | status: in_progress` - **Progress:** changes `7/9` - **Related status:** [Flint Forg"
tags:
- flint-forge
- auth-rls
- ory-keto
- cedar-policy
- graphql-subscriptions
- fdb-realtime
- postgres-rls
- change-stream
links:
- flint-forge-p3-auth-rls-keto-phase-status
- flint-forge-p3-c020-listen-change-source-integration-plan
- flint-forge-p3-c019-postgrest-engine-read-write-progress
- flint-forge-g4-graphql-subscription-wiring-scope-decision
- flint-forge-p3-c019-merged-status-and-follow-ups
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T18:01:07.511023+00:00
created_at: 2026-07-03T18:01:07.511023+00:00
updated_at: 2026-07-03T18:01:07.511023+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T17:55:07Z`
- **Position:** `p3-auth-rls-keto | status: in_progress`
- **Progress:** changes `7/9`
- **Related status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)
- **Related integration plan:** [Flint Forge p3-c020 Listen Change Source Integration Plan](/flint-forge-p3-c020-listen-change-source-integration-plan.md)

## Phase Gate

All four authentication and authorization layers must be live end-to-end:

1. A real `flint-gate` JWT causes a real Postgres RLS row filter.
2. A Keto relation check gates mutations.
3. A Cedar policy controls capability-level access.
4. Zero plaintext credentials appear in any log line or tracing span.
5. CRUD handler bodies execute parameterized SQL.

## Phase Goals

- **G1 — `forge-policy`: Cedar policy evaluation crate**
  - `PolicyEngine::evaluate(principal, action, resource, context)` returns allow/deny.
  - Policy bundles loaded from `flint_meta.cedar_policies`.
- **G2 — Keto coarse relationship checks**
  - Enforced at subscribe-time and mutation-time.
  - `KetoCacheClient` caches relation tuples with TTL.
  - Cache invalidates on Keto webhook.
  - Integrated into `fdb-app` use cases.
- **G3 — Full RLS CRUD handler bodies in `RestCompiler`**
  - Implement `handle_list`, `handle_insert`, `handle_update`, and `handle_delete`.
  - SQL must be parameterized.
  - Filter operator dispatch must support: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `in`, `is`, `cs`, `cd`.
  - Support `Range` header pagination.
  - Validate column names for safety.
  - Related implementation context: [Flint Forge p3-c019 PostgREST Engine Read/Write Progress](/flint-forge-p3-c019-postgrest-engine-read-write-progress.md).
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for `Query`/`Mutation` under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws`.
  - Subscriptions pull from `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema ∪ subscription SDL.
  - Scope context: [Flint Forge G4 GraphQL Subscription Wiring Scope Decision](/flint-forge-g4-graphql-subscription-wiring-scope-decision.md).
- **G5 — Subscription RLS enforcement**
  - For every `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivering.
  - This is required WAL-bypass protection and is non-negotiable.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state` for DEK serde security.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated via service token.
  - Includes reconnect loop and fan-out to subscriber streams.

## Dependencies Delivered from Phase 2

- `CompiledState` and `DatabaseModel` — delivered in `p2-c003`.
- `RestCompiler` route registration — delivered in `p2-c004`; handler bodies remain Phase 3 deliverables.
- `StateManager` + `ArcSwap` hot-reload — delivered in `p2-c005`.
- `fdb-auth` JWT verify → `RlsContext` — delivered in `p2-c001`.
- `SET LOCAL` RLS propagation — delivered in `p2-c002`.

## Pre-flight Check for GraphQL Hybrid

Before starting G4, verify OQ-3 against the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Session Update: p3-c020 Relaunch

- Relaunched background task `wpcj66bwg` with a fix: use prose design instead of the previously over-constrained schema.
- Pipeline remains: **Design → Implement** with adapter and migration in parallel → **Verify** with security and concurrency review.
- Root cause of previous failure:
  - `DESIGN_SCHEMA` required 8 long free-text fields.
  - Model output repeatedly failed JSON-schema validation past the 5-retry cap.
  - Prose design is more robust and downstream implementation agents consume it identically.
- Repository was confirmed untouched by the failed run.

## Planned Follow-up When `wpcj66bwg` Completes

1. Review design, adapter, migration, and two adversarial reviews.
2. Inspect the actual working tree.
3. Integrate `fdb-realtime::ListenChangeSource` and migration `0006` manually.
4. Apply reviewer-flagged fixes.
5. Add `sqlx` and `tokio` dependencies.
6. Wire the implementation into the gateway as an env-selectable `ChangeStreamSource`.
7. Verify with `cargo test`, `clippy`, and workspace checks before commit and PR.

## External Review Dependency

- **PR #5** for embedding REST wiring is still awaiting review/merge.
- Prior merged context: [Flint Forge p3-c019 Merged Status and Follow-ups](/flint-forge-p3-c019-merged-status-and-follow-ups.md).

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto