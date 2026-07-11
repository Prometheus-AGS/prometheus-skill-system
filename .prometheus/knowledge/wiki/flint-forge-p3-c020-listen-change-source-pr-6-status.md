---
type: Reference
id: flint-forge-p3-c020-listen-change-source-pr-6-status
title: 'Flint Forge p3-c020 LISTEN Change Source PR #6 Status'
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T18:18:01Z` - **Position:** `p3-auth-rls-keto | status: in_progress` - **Progress:** changes `7/9` - **PR:** #6 \u2014 `https://github.co"
tags:
- flint-forge
- auth-rls
- postgres-listen
- graphql-subscriptions
- ory-keto
- postgres-rls
- change-stream
- fdb-realtime
links:
- flint-forge-p3-auth-rls-keto-goals-and-p3-c020-relaunch
- flint-forge-p3-c020-listen-change-source-integration-plan
- flint-forge-p3-c019-merged-status-and-follow-ups
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
- https://github.com/Know-Me-Tools/flint-forge/pull/6
timestamp: 2026-07-03T18:23:08.838332+00:00
created_at: 2026-07-03T18:23:08.838332+00:00
updated_at: 2026-07-03T18:23:08.838332+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T18:18:01Z`
- **Position:** `p3-auth-rls-keto | status: in_progress`
- **Progress:** changes `7/9`
- **PR:** #6 — `https://github.com/Know-Me-Tools/flint-forge/pull/6`
- **Related phase record:** [Flint Forge p3 Auth/RLS/Keto Goals and p3-c020 Relaunch](/flint-forge-p3-auth-rls-keto-goals-and-p3-c020-relaunch.md)
- **Related integration plan:** [Flint Forge p3-c020 Listen Change Source Integration Plan](/flint-forge-p3-c020-listen-change-source-integration-plan.md)
- **Prior merged dependency:** [Flint Forge p3-c019 Merged Status and Follow-ups](/flint-forge-p3-c019-merged-status-and-follow-ups.md)

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
  - Range header pagination.
  - Column-name safety validation.
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for Query/Mutation under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws` pulling from `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema ∪ subscription SDL.
- **G5 — Subscription RLS enforcement**
  - For each `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivering.
  - WAL-bypass protection is mandatory.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state`.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated via service token.
  - Reconnect loop.
  - Fan-out to subscriber streams.

## Phase 2 Dependencies

- `CompiledState` and `DatabaseModel` delivered by `p2-c003`.
- `RestCompiler` route registration delivered by `p2-c004`; handler bodies remain Phase 3 work.
- `StateManager` + `ArcSwap` hot-reload delivered by `p2-c005`.
- `fdb-auth` JWT verification to `RlsContext` delivered by `p2-c001`.
- `SET LOCAL` RLS propagation delivered by `p2-c002`.

## Pre-flight Check for GraphQL Hybrid

Before G4 work, verify OQ-3 against the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## PR #6 Delivered Work

PR #6 implements the recommended next step for the p3 LISTEN change source path:

- Added `ListenChangeSource`.
- Added migration `0006`:
  - trigger support,
  - opt-in helper,
  - PK-only overflow fallback.
- Added gateway wiring:
  - `FLINT_CHANGE_SOURCE=listen` selects the LISTEN implementation.
  - `build_subscription_factory` is now async.
- Verified and integrated the implementation manually after design→implement→adversarial-verify workflow.

## Design Constraints and Architecture

Reasoning chain:

```text
Layer 1: BoxStream<'static> Send captures + fail-closed auth + no-leak lifecycle
    ↑
Layer 3: Cloud-Native + Web — Postgres LISTEN/NOTIFY change feed for GraphQL subs
  Constraint: dedicated long-lived LISTEN connection; NOTIFY bypasses RLS → per-event
              re-query mandatory (§3.3); 8000-byte payload cap must not break user writes.
    ↓
Layer 2: single PgListener → tokio::broadcast fan-out; adapter does Keto + raw stream
         only; RLS re-query reused from Quarry; no fdb-app dependency; PK-only overflow fallback.
```

Key security invariant: `NOTIFY` bypasses RLS, so every delivered subscription event must pass through `subscribe_rls_filtered` and re-query the changed row as the subscriber before delivery.

## Adversarial Verification Findings

Security review result: **clean**.

Concurrency review found and fixed 3 issues, with regression tests:

1. **Identifier validation before Keto URL construction**
   - Defense-in-depth.
   - Fail-closed behavior.
2. **Task/connection leak on drop**
   - Fixed via `ListenTaskGuard` aborting on final `Arc` drop.
3. **Miss window**
   - Fixed by subscribing before awaiting Keto.

## Validation Performed

Manual validation after integration:

- Full test + clippy gate rerun by implementer.
- **24 tests pass**.
- `fdb-realtime` clippy clean with `-D warnings`.
- Gateway binary clippy clean with `-D warnings`.
- Workspace check clean.
- A `single_match_else` clippy error in gateway wiring was caught and fixed.

Known unrelated failures were confirmed as pre-existing on clean `main` and intentionally not modified:

- `fdb-gateway` `keto_sync` environment flake.
- `a2ui_seed_test` lint.

## End-to-End Realtime Path

With `PgRest::execute` from p3-c019 merged and PR #6 applied, the alternative realtime path becomes:

```text
DML on opted-in table
  → Postgres NOTIFY
  → ListenChangeSource
  → subscribe_rls_filtered
  → RLS re-query as subscriber
  → GraphQL subscription delivery
```

`flint-realtime-fabric` remains the default source. `FLINT_CHANGE_SOURCE=listen` enables the working Postgres LISTEN/NOTIFY alternative.

## Open Gaps and Next Steps

Open gap:

- Unit/mock-tested throughout, but no live-Postgres integration test yet proving actual `NOTIFY` → subscription delivery.

Open PRs awaiting review/merge:

- #5 — embedding REST wiring.
- #6 — LISTEN change source.

Recommended closers after #5 and #6 land:

1. Add a live-Postgres integration test proving `NOTIFY` → subscription delivery.
2. Run `/kbd-reflect` to reconcile phase state and advance `p3-auth-rls-keto`.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto
3. https://github.com/Know-Me-Tools/flint-forge/pull/6