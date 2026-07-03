---
type: Reference
id: flint-forge-p3-c019-postgrest-query-engine-foundation
title: Flint Forge p3-c019 PostgREST Query Engine Foundation
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
- flint-forge-g4-graphql-subscription-wiring-scope-decision
sources:
- stdin
- manual:Flint Forge/p3-auth-rls-keto
timestamp: 2026-07-03T15:45:06.833439+00:00
created_at: 2026-07-03T15:45:06.833439+00:00
updated_at: 2026-07-03T15:45:06.833439+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3-auth-rls-keto`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T15:37:15Z`
- **Status:** `in_progress`
- **Progress:** changes `7/9`
- **Branch:** `feat/p3-c019-postgrest-query-engine`
- **Related phase status:** [Flint Forge p3 Auth RLS Keto Phase Status](/flint-forge-p3-auth-rls-keto-phase-status.md)

## Phase Gate

All four authentication and authorization layers must be live end-to-end:

1. A real `flint-gate` JWT causes a real Postgres RLS row filter.
2. A Keto relation check gates mutations.
3. A Cedar policy controls capability-level access.
4. Zero plaintext credentials appear in logs or tracing spans.
5. CRUD handler bodies execute parameterized SQL.

## Decision: Single Pure `fdb-query` Crate

Recommended architecture is **option b3: a single, pure, I/O-free `fdb-query` crate** consumed by both:

- `fdb-reflection` REST router
- `fdb-postgres::PgRest`

Rationale:

- The PostgREST-compatible translator is a security-critical surface.
- Two separate translators would drift and risk SQL injection or filter-bypass defects.
- Keeping translation pure as `(request) -> (sql, params)` avoids DB coupling and enables a large, cheap unit-test matrix.
- This aligns with the Compile Economy rule: PostgREST parity can be verified mostly through fast unit tests.

## Change Scope: `p3-c019`

A dedicated KBD change, `p3-c019`, was created to implement the shared PostgREST query engine.

Committed artifacts:

- `proposal.md`
- `tasks.md`

The change spec covers the full PostgREST surface:

- Horizontal operators
- Logical trees
- `select`
- `order`
- Pagination
- Count modes
- Writes and upsert
- Resource embedding
- Full-text search variants
- Edge cases and adversarial security checks

Execution strategy: **core-complete first, parity second**.

## Implemented Foundation

Phase-1 tasks T1–T3 are complete in the `fdb-query` crate.

### `param.rs`

Backend-agnostic bind parameter model:

- Every value is represented as a bind parameter.
- SQL rendering uses positional parameters such as `$1`, `$2`, etc.
- Literal interpolation is avoided by construction.

### `operator.rs`

Implemented all **21 PostgREST horizontal operators**:

- Standard comparisons and pattern operators
- `not.` negation support
- `any` and `all` quantifiers
- Exact-SQL render tests for operators

This extends beyond the Phase 3 G3 minimum filter list (`eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `in`, `is`, `cs`, `cd`) toward full PostgREST parity.

### `ident.rs`

Hardened column-reference layer:

- JSON path support
- Cast support
- Identifier validation
- Escaped quotes for rendered identifiers
- Injection-resistant column reference rendering

## Verification

The available test-wait budget for this epoch was fully spent: `3/3`.

Verified commands:

```bash
cargo test -p fdb-query
# 25 pass

cargo clippy -p fdb-query --all-targets -- -D warnings
# clean
```

## Current Status

The committed work is the **foundation**, not the finished query engine.

Remaining `p3-c019` core tasks:

- **T4:** Logical trees
- **T5:** `select`, `order`, pagination, count
- **T6:** Writes
- **T7:** Wire `fdb-query` into `fdb-reflection`
- **T8:** Implement `PgRest::execute`

Completing T8 is important because it retires the current `todo!()` and makes the G4 subscription re-query path live. That path is part of the GraphQL subscription/RLS work described in [Flint Forge G4 GraphQL Subscription Wiring Scope Decision](/flint-forge-g4-graphql-subscription-wiring-scope-decision.md).

Remaining parity pass:

- Resource embedding, identified as the hardest remaining area
- Full-text search variants
- PostgREST edge cases
- Additional adversarial verification for the shared translator

## Open Items

- Continue `p3-c019` T4→T8, then parity.
- Optionally push `feat/p3-c019-postgrest-query-engine` and open a draft PR so the foundation is visible.
- PR #2 for the G4 seam still awaits review/merge.
- In-process `LISTEN` `ChangeStreamSource` remains a separate change.
- A fresh session is recommended before continuing because the current epoch's test-wait budget is spent.

## Workflow Note

The remaining work is suitable for a multi-agent workflow because the work can be split across:

- Operator-family parity
- Consumer wire-ups
- Adversarial verification of the security-critical SQL translator

No workflow was started because explicit opt-in was not available during the session.

# Citations

1. stdin
2. manual:Flint Forge/p3-auth-rls-keto