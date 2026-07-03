---
type: Reference
id: flint-forge-p3-auth-rls-keto-reflection-and-merge-summary
title: Flint Forge p3 Auth/RLS/Keto Reflection and Merge Summary
tags:
- flint-forge
- auth-rls
- ory-keto
- cedar-policy
- graphql-subscriptions
- postgres-rls
- kbd-reflection
- change-stream
links:
- flint-forge-p3-auth-rls-keto-goals-and-p3-c020-relaunch
- flint-forge-p3-auth-rls-keto-phase-status
- flint-forge-p3-c020-listen-change-source-pr-6-status
- flint-forge-p3-c020-listen-change-source-integration-plan
- flint-forge-p3-c019-merged-status-and-follow-ups
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T20:03:03.490479+00:00
created_at: 2026-07-03T20:03:03.490479+00:00
updated_at: 2026-07-03T20:03:03.490479+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T18:43:45Z`
- **Position:** `p3-auth-rls-keto | status: in_progress`
- **Progress marker:** changes `7/9`
- **Related phase record:** [Flint Forge p3 Auth/RLS/Keto Goals and p3-c020 Relaunch](/flint-forge-p3-auth-rls-keto-goals-and-p3-c020-relaunch.md)
- **Related status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)

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
  - Introspection merges `pg_graphql` schema and subscription SDL.
- **G5 — Subscription RLS enforcement**
  - For each `EntityChange` from `fdb-realtime`, re-query changed row as the subscriber with full `RlsContext` before delivery.
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

## Phase 2 Dependencies

- `CompiledState` and `DatabaseModel` — delivered in `p2-c003`.
- `RestCompiler` route registration — delivered in `p2-c004`; handler bodies remained a Phase 3 deliverable.
- `StateManager` + `ArcSwap` hot-reload — delivered in `p2-c005`.
- `fdb-auth` JWT verification to `RlsContext` — delivered in `p2-c001`.
- `SET LOCAL` RLS propagation — delivered in `p2-c002`.

## Pre-flight Check for GraphQL Hybrid

Before starting G4, verify OQ-3 against the PG18 container:

```sql
SELECT extversion FROM pg_extension WHERE extname = 'pg_graphql';
```

If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Merge Results

Two PRs were reviewed and merged:

- **PR #5 — embedding REST wiring**
  - Status before merge: `MERGEABLE` / `CLEAN`.
  - Merge method: squash.
  - Result commit: `b786335`.
- **PR #6 — LISTEN change source + live-PG tests**
  - Status before merge: `UNKNOWN` because of GitHub lag.
  - Safety step: trial-merged locally into post-PR #5 `main` to confirm no conflicts.
  - Merge method: real merge after local conflict check.
  - Result commit: `094f74e`.
  - Related PR status: [Flint Forge p3-c020 LISTEN Change Source PR #6 Status](/flint-forge-p3-c020-listen-change-source-pr-6-status.md)
  - Related integration plan: [Flint Forge p3-c020 Listen Change Source Integration Plan](/flint-forge-p3-c020-listen-change-source-integration-plan.md)

After syncing `main`, the integrated workspace was verified:

- `cargo check --workspace` clean.
- Test suites green together:
  - `fdb-query`: `128 + 29`.
  - `fdb-postgres`: `4`.
  - `fdb-realtime`: `24` plus `2` ignored live-PG tests.
  - `fdb-reflection`: `53` plus gates.

## Reflection Outcome

`/kbd-reflect` completed for `p3-auth-rls-keto` as step 7 of 9, and `reflection.md` was written.

Measured against G1–G7:

- **G1 — MET**.
- **G2 — MET**.
- **G3 — MET / exceeded**: scope expanded to full PostgREST parity via `fdb-query`.
- **G4 — MET**.
- **G5 — MET**.
- **G6 — MET**.
- **G7 — PARTIAL**: `flint-realtime-fabric` `WatchEntityType` remains blocked by `OQ-FRF-1`; the capability is currently delivered through the in-process `ListenChangeSource` alternative rather than the gRPC FRF adapter.

## KBD Bookkeeping Drift

The reflection documents a `progress.json` reconciliation issue:

- `c019`, `c020`, and the G4 seam are merged but not represented in the tracked change array.
- `c017` is superseded by `c020`.
- `c018` overlaps merged work.
- Recommendation: reconcile `progress.json` before phase close, preferably via `/kbd-status` or the orchestrator rather than a manual edit.

This aligns with prior p3 bookkeeping notes in [Flint Forge p3-c019 Merged Status and Follow-ups](/flint-forge-p3-c019-merged-status-and-follow-ups.md).

## Operational Caveats

- KBD hooks, stage-gate, and waypoint-advance could not be fired programmatically because `KBD_ORCHESTRATOR_ROOT` was unset and shell libraries such as `waypoint.sh` and `stage-gate.sh` were not resolvable in the environment.
- Durable artifacts were written according to the established convention:
  - `reflection.md`
  - `handoffs/reflect.md`
- Trigger echo was emitted, but shell side effects were not faked.
- The waypoint was **not** auto-advanced.
- The reflection is a git-tracked KBD state file authored during the session, but it was **not committed**; KBD state commits remain a separate, user-driven step.

## Recommended Next Steps

1. Commit `reflection.md` and `handoffs/reflect.md` if they should be tracked.
2. Reconcile `progress.json`:
   - register `c019` and `c020`;
   - mark `c017` superseded;
   - mark `c018` resolved or otherwise account for overlap.
3. Advance with `/kbd-new-phase` after clean reconciliation.
4. Candidate next phases:
   - CI/Postgres hardening plus `OQ-FRF-1` resolution;
   - Flint Kiln `fke-*` work.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto