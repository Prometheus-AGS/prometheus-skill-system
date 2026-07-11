---
type: Reference
id: flint-forge-p3-c019-merged-status-and-follow-ups
title: Flint Forge p3-c019 Merged Status and Follow-ups
description: "Project:** Flint Forge - **Phase:** `p3-auth-rls-keto` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T17:30:32Z` - **Position:** `p3-auth-rls-keto | status: in_progress` - **Displayed progress:** changes `7/9` - **Related phase recor"
tags:
- flint-forge
- auth-rls
- postgrest
- graphql-subscriptions
- postgres-rls
- ory-keto
- kbd-bookkeeping
links:
- flint-forge-p3-auth-rls-keto-phase-status
- flint-forge-g4-graphql-subscription-wiring-scope-decision
- flint-forge-p3-c019-postgrest-query-engine-foundation
- flint-forge-p3-c019-postgrest-engine-read-write-progress
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T17:37:02.691764+00:00
created_at: 2026-07-03T17:37:02.691764+00:00
updated_at: 2026-07-03T17:37:02.691764+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T17:30:32Z`
- **Position:** `p3-auth-rls-keto | status: in_progress`
- **Displayed progress:** changes `7/9`
- **Related phase record:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)

## Phase Gate

All four authentication/authorization layers must be live end-to-end:

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
  - Before GraphQL hybrid work, verify OQ-3: `SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';` against the PG18 container. If missing, defer G4 to `p3-c007` with a stub.
- **G5 — Subscription RLS enforcement**
  - For each `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with full `RlsContext` before delivery.
  - This WAL-bypass protection is non-negotiable.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state`.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated via service token.
  - Reconnect loop and fan-out to subscriber streams.

## Phase 2 Dependencies Already Delivered

- `CompiledState` and `DatabaseModel` — `p2-c003`.
- `RestCompiler` route registration — `p2-c004`; handler bodies are Phase 3 scope.
- `StateManager` + `ArcSwap` hot-reload — `p2-c005`.
- `fdb-auth` JWT verify → `RlsContext` — `p2-c001`.
- `SET LOCAL` RLS propagation — `p2-c002`.

## Session Outcome

All session work was merged to `main`; the working tree is clean and there is no work in flight.

Merged PRs:

| PR | Outcome |
|---|---|
| #1 | Integration-First + Compile Economy development-management policy: `CLAUDE.md`, `AGENTS.md`, docs, build config. |
| #2 | G4 GraphQL subscription seam: transport ↔ RLS-filtered use case. See [Flint Forge G4 GraphQL Subscription Wiring Scope Decision](/flint-forge-g4-graphql-subscription-wiring-scope-decision.md). |
| #3 | `p3-c019` PostgREST engine core: `fdb-query`, live `PgRest::execute`, reflection wired. Builds on [Flint Forge p3-c019 PostgREST Query Engine Foundation](/flint-forge-p3-c019-postgrest-query-engine-foundation.md). |
| #4 | `p3-c019` parity: resource embedding, full-text search, edge cases. Built via multi-agent workflow; two security defects were caught and fixed. See [Flint Forge p3-c019 PostgREST Engine Read/Write Progress](/flint-forge-p3-c019-postgrest-engine-read-write-progress.md). |

## Progress Counter Note

`progress.json` tracks `p3-c010` through `p3-c016` in the original planned phase change set, all marked `qa_passed`. The displayed `7/9` counter reflects that original planned list.

This session added new changes:

- `c017` / `c018`: stub + verification work.
- `c019`: substantial PostgREST engine work, merged across four PRs.

The phase progress files (`current-waypoint.json`, `progress.json`) are owned by the KBD orchestrator flow. They should be reconciled by a `/kbd-*` skill, not hand-edited mid-session. The session therefore reports the footer value faithfully instead of fabricating a new counter.

## Remaining Follow-ups

No follow-up was started. Recommended next options:

1. **Wire embedding into the `fdb-reflection` REST list handler**
   - Convert `DatabaseModel → EmbedSchema`.
   - Engine support is already on `main` and tested.
   - HTTP handler does not yet call it.
   - Smallest and recommended next step to make embedding reachable end-to-end.
2. **Add an in-process Postgres `LISTEN` `ChangeStreamSource`**
   - Workaround for OQ-FRF-1.
   - With `PgRest::execute` live, this is the last piece needed for G4 subscriptions to emit real events.
   - `FabricChangeSource::watch` remains an empty stream until the upstream FRF RPC is available.
3. **Run KBD phase bookkeeping**
   - Use `/kbd-status` → `/kbd-reflect` or equivalent.
   - Reconcile `progress.json` with `c017`–`c019` and advance phase state.
   - This is orchestrator-owned state, not a manual edit target.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto