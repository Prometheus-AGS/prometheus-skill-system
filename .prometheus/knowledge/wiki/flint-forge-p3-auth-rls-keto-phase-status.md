---
type: Reference
id: flint-forge-p3-auth-rls-keto-phase-status
title: Flint Forge p3 Auth RLS Keto Phase Status
tags:
- flint-forge
- auth-rls
- ory-keto
- cedar-policy
- graphql-subscriptions
- postgres-rls
- okf
links:
- flint-gate-agent-authorization-control-plane-execution-plan
- okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T14:21:34.240395+00:00
created_at: 2026-07-03T14:21:34.240395+00:00
updated_at: 2026-07-03T14:21:34.240395+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T14:16:23Z`
- **Status:** `in_progress`
- **Progress:** changes `7/9`

## Phase Gate

All four authentication and authorization layers must be live end-to-end:

1. A real [flint-gate](/flint-gate-agent-authorization-control-plane-execution-plan.md) JWT causes a real Postgres RLS row filter.
2. A Keto relation check gates mutations.
3. A Cedar policy controls capability-level access.
4. Zero plaintext credentials appear in logs or tracing spans.
5. CRUD handler bodies execute parameterized SQL.

## Goals

- **G1 — `forge-policy`: Cedar policy evaluation crate**
  - `PolicyEngine::evaluate(principal, action, resource, context)` returns allow/deny.
  - Policy bundles are loaded from `flint_meta.cedar_policies`.
- **G2 — Keto coarse relationship checks**
  - Subscribe-time and mutation-time relation checks.
  - `KetoCacheClient` caches relation tuples with TTL.
  - Cache invalidates on Keto webhook.
  - Integrated into `fdb-app` use cases.
- **G3 — Full RLS CRUD handler bodies in `RestCompiler`**
  - Implement `handle_list`, `handle_insert`, `handle_update`, and `handle_delete`.
  - Use parameterized SQL.
  - Support filter operators: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `in`, `is`, `cs`, `cd`.
  - Support Range header pagination.
  - Validate column-name safety.
- **G4 — GraphQL hybrid**
  - `pg_graphql` passthrough for Query/Mutation under RLS.
  - `async-graphql` `Subscription` over `graphql-transport-ws`.
  - Subscriptions pull from `ChangeStreamSource`.
  - Introspection merges `pg_graphql` schema with subscription SDL.
- **G5 — Subscription RLS enforcement**
  - For every `EntityChange` from `fdb-realtime`, re-query the changed row as the subscriber with the full `RlsContext` before delivery.
  - This protects against WAL-bypass leaks and is non-negotiable.
- **G6 — Gate tests**
  - `test_rest_select_with_eq_filter` covering all 12 filter operators.
  - `test_vault_dek_not_in_compiled_state` for DEK serde security.
  - `test_subscription_rls_drops_unauthorized_events`.
  - `test_keto_check_gates_mutation`.
- **G7 — `fdb-realtime` gRPC client**
  - `ChangeStreamSource` adapter connects to `flint-realtime-fabric` `WatchEntityType` RPC.
  - Authenticated with service token.
  - Reconnect loop.
  - Fan-out to subscriber streams.

## Phase 2 Dependencies Already Delivered

- `CompiledState` and `DatabaseModel` — delivered in `p2-c003`.
- `RestCompiler` route registration — delivered in `p2-c004`; handler bodies remain a Phase 3 deliverable.
- `StateManager` plus `ArcSwap` hot reload — delivered in `p2-c005`.
- `fdb-auth` JWT verification to `RlsContext` — delivered in `p2-c001`.
- `SET LOCAL` RLS propagation — delivered in `p2-c002`.

## GraphQL Pre-flight Check

Before starting G4, verify OQ-3 against the PG18 container:

```sql
SELECT extversion
FROM pg_extension
WHERE extname = 'pg_graphql';
```

Decision rule:

- If `pg_graphql` is installed, proceed with GraphQL hybrid implementation.
- If `pg_graphql` is not installed, defer G4 to `p3-c007` with a stub.

## Session Outcome

A documentation/development-management integration change was committed cleanly on a dedicated branch.

- **Commit:** `9fa07fb`
- **Branch:** `docs/dev-management-integration-first`
- **Diff:** 8 files, +468 lines

| File | Change |
|---|---|
| `docs/RUST-DEVELOPMENT-MANAGEMENT.md` | New canonical policy |
| `AGENTS.md` | New cross-tool agent guide |
| `CLAUDE.md` | Binding summary section |
| `Cargo.toml` | Live `[profile.dev]` / `[profile.release]` |
| `.cargo/config.toml.example` | Fast-linker template with verified path |
| `.gitignore` | Ignores live `.cargo/config.toml` |
| `.prometheus/knowledge/wiki/{integration-first-compile-economy,index}.md` | OKF v0.1 wiki page and index |

## Git Workflow Notes

- The commit was branched from `main`, not made directly on the default branch.
- Unrelated pre-existing working-tree changes were intentionally excluded:
  - `fdb-app/*`
  - `gate_tests.rs`
  - `p3-c016` OpenSpec archive move
  - KBD state files: `current-waypoint.json`, `progress.json`, `events.jsonl`
- Those excluded changes remain uncommitted on the branch and belong to the phase-execution flow, not the docs commit.
- The wiki page was reformatted to conform to the repository's [OKF v0.1](/okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness.md) schema conventions: frontmatter plus `# Citations`, so `pk lint` should pass.

## Current Next Step

No push or PR was opened. Recommended options:

1. Push `docs/dev-management-integration-first` and open a PR, after deciding how to handle the unrelated uncommitted changes.
2. Resume `p3-auth-rls-keto` implementation. The main remaining gap is G4: GraphQL hybrid subscription over `graphql-transport-ws` pulling from `ChangeStreamSource`.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto