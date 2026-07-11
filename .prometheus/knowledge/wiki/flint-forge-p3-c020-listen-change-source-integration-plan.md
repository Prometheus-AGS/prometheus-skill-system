---
type: Reference
id: flint-forge-p3-c020-listen-change-source-integration-plan
title: Flint Forge p3-c020 Listen Change Source Integration Plan
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T17:47:01Z` - **Position:** `p3-auth-rls-keto | status: in_progress` - **Progress:** changes `7/9` - **Branch:** `feat/p3-c020-liste"
tags:
- flint-forge
- auth-rls
- postgres-listen
- graphql-subscriptions
- ory-keto
- fdb-realtime
- change-stream
links:
- flint-forge-p3-auth-rls-keto-phase-status
- flint-forge-p3-c019-merged-status-and-follow-ups
- flint-forge-p3-c019-postgrest-engine-read-write-progress
- flint-forge-g4-graphql-subscription-wiring-scope-decision
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T17:52:52.706923+00:00
created_at: 2026-07-03T17:52:52.706923+00:00
updated_at: 2026-07-03T17:52:52.706923+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T17:47:01Z`
- **Position:** `p3-auth-rls-keto | status: in_progress`
- **Progress:** changes `7/9`
- **Branch:** `feat/p3-c020-listen-change-source` off `main`
- **Related phase status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)
- **Prior merged status:** [Flint Forge p3-c019 Merged Status and Follow-ups](/flint-forge-p3-c019-merged-status-and-follow-ups.md)

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
  - `handle_list`, `handle_insert`, `handle_update`, and `handle_delete`.
  - Parameterized SQL.
  - Filter operator dispatch for `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `in`, `is`, `cs`, `cd`.
  - Range header pagination.
  - Column-name safety validation.
  - See PostgREST implementation context in [Flint Forge p3-c019 PostgREST Engine Read/Write Progress](/flint-forge-p3-c019-postgrest-engine-read-write-progress.md).
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for Query/Mutation under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws` pulling from `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema with subscription SDL.
  - Scope decisions are captured in [Flint Forge G4 GraphQL Subscription Wiring Scope Decision](/flint-forge-g4-graphql-subscription-wiring-scope-decision.md).
- **G5 — Subscription RLS enforcement**
  - For each `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivering.
  - This WAL-bypass protection is non-negotiable.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state` for DEK serde security.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated via service token.
  - Includes reconnect loop and fan-out to subscriber streams.

## Phase 2 Dependencies Already Delivered

- `CompiledState` and `DatabaseModel` — delivered in `p2-c003`.
- `RestCompiler` route registration — delivered in `p2-c004`; handler bodies remain Phase 3 deliverables.
- `StateManager` + `ArcSwap` hot-reload — delivered in `p2-c005`.
- `fdb-auth` JWT verification to `RlsContext` — delivered in `p2-c001`.
- `SET LOCAL` RLS propagation — delivered in `p2-c002`.

## Pre-flight Check for GraphQL Hybrid

Before starting G4, verify OQ-3 against the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Current Workflow

A background workflow was launched as task `wj20mlio6`:

1. Design.
2. Implement adapter and migration in parallel.
3. Verify via security and concurrency adversarial reviews.

The workflow is advisory only: it produces artifacts, not trusted repo writes. Required follow-up after completion:

- Inspect the actual working tree.
- Integrate the adapter and migration manually.
- Apply defect fixes found by the two reviewers.
- Run real `cargo test`, `clippy`, and workspace checks.
- Wire the implementation into the gateway.
- Trust workflow claims only after compiler/test verification.

## p3-c020 Design Decisions

- **Adapter scope is intentionally narrow**:
  - Performs subscribe-time Keto check.
  - Produces raw `ChangeEvent` stream only.
  - Does not duplicate per-event RLS re-query logic.
  - Does not depend on `fdb-app`.
- **Per-event RLS re-query already exists**:
  - `Quarry::subscribe_rls_filtered` from merged G4 performs the subscriber-context re-query before delivery.
- **Fan-out architecture**:
  - One background `PgListener` owns the PostgreSQL `LISTEN` connection.
  - Events are published to `tokio::broadcast`.
  - Each `watch()` filters by `entity_type`.
  - One LISTEN connection serves many subscribers.
- **Keto behavior is fail-closed**:
  - If Keto is unreachable, deny access.
  - `keto_subject` must never be logged.
- **PostgreSQL NOTIFY payload limit is handled**:
  - 8000-byte NOTIFY payload overflow is explicitly addressed.
  - RLS re-query fetches the full row regardless of NOTIFY payload truncation/size constraints.

## Integration Plan

When the workflow completes:

1. Review its design, adapter, migration, and both adversarial reviews.
2. Integrate `fdb-realtime::ListenChangeSource`.
3. Add migration `0006`.
4. Apply reviewer-flagged fixes.
5. Add required `sqlx` and `tokio` dependencies.
6. Wire the gateway to select `ListenChangeSource` via environment flag alongside `FabricChangeSource`.
7. Verify with:
   - `cargo test`
   - `cargo clippy`
   - workspace check
8. Commit and open a PR.

## External Dependency / Coordination

PR #5, covering embedding REST wiring, still awaits review and merge. The `feat/p3-c020-listen-change-source` branch is independent of the unmerged PR #5 embedding work because it touches different crates and is expected not to conflict.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto