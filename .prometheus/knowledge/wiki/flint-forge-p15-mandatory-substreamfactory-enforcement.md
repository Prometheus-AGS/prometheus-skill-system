---
type: Reference
id: flint-forge-p15-mandatory-substreamfactory-enforcement
title: Flint Forge p15 Mandatory SubStreamFactory Enforcement
tags:
- flint-forge
- production-readiness
- graphql-subscriptions
- state-manager
- substream-factory
- rust
- kbd-phase
links:
- flint-forge-p15-statemanager-constructor-removal
sources:
- stdin
timestamp: 2026-07-16T20:34:24.445388+00:00
created_at: 2026-07-16T20:34:24.445388+00:00
updated_at: 2026-07-16T20:34:24.445388+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p15-v1.0-production-readiness`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge/.claude/worktrees/ecstatic-ardinghelli-9e3717`
- **Captured:** `2026-07-16T20:32:20Z`
- **Position:** `p5-kiln-edge-runtime | status: completed`
- **Progress:** changes `5/5`
- **Related p15 constructor cleanup:** [Flint Forge p15 StateManager Constructor Removal](/flint-forge-p15-statemanager-constructor-removal.md)

## Phase Goal

Close the gap between “workspace compiles and unit tests pass” and a production-ready Flint Forge `v1.0`, focusing on build integrity, operator tooling, end-to-end validation, documentation accuracy, and production packaging. No new features were targeted.

## Planned Production-Readiness Blockers

### p15-c001 — Anvil Extension Stabilization

Goal: make all five `ext-flint-*` / `flint_*` pgrx extensions compile and pass `cargo pgrx test` on one supported toolchain.

Required work:

- Unify pgrx version and Postgres target.
- Fix `DatumWithOid` compile error in `ext-flint-meta`.
- Resolve workspace-inheritance misconfiguration for excluded crates.
- Add a pgrx CI job in a Linux container.
- Gate: `cargo pgrx test` passes for all extensions in CI.

### p15-c002 — Migration Integrity

Goal: restore strict linear migration ordering and verify migrations in CI.

Required work:

- Renumber colliding `migrations/0005_*` and `migrations/0006_*` files.
- Add CI step running `sqlx migrate run` against an empty Postgres 18 database.

## Change Summary

The no-factory GraphQL subscription state is now unrepresentable in the type system. `SubStreamFactory` is mandatory instead of optional across the GraphQL compile and state-manager construction path.

Updated signatures/fields:

- `GraphQlCompiler::compile()` now requires `SubStreamFactory`.
- `StateManager::new_with_gates()` now requires `SubStreamFactory`.
- Private `do_compile()` now requires `SubStreamFactory`.
- `StateManager` stores a plain `SubStreamFactory`, not `Option<SubStreamFactory>`.
- Gateway startup no longer wraps the factory in `Some(...)` at `crates/fdb-gateway/src/main.rs:189`.

Result: `Option<SubStreamFactory>` no longer appears anywhere in the codebase after this change and the prior Option 2 work.

## Resolver Behavior

The resolver match was reduced from four cases to two:

- Authenticated subscribers receive the stream created by the configured factory.
- Unauthenticated subscribers receive an error.

Removed behavior:

- The `(None, _)` no-factory arm.
- `Option` plumbing through `state_manager.rs`, including `.as_ref()` and `.cloned()` usage for the factory.

Rationale: there is no valid runtime where the gateway has no `SubStreamFactory`. Keeping a no-factory branch made a dead boot hazard look live and required future readers to reason about a state the program should never enter.

## Test Changes

Removed test:

- `no_factory_yields_no_events`
  - Reason: it guarded a state that is no longer constructible.

Added/replacement test:

- `authenticated_subscriber_receives_only_factory_events`
  - Asserts the surviving security property: subscription events reach authenticated subscribers only through the factory.

SDL tests were updated to pass an explicit `empty_factory()`. “No live stream” is now deliberate test setup rather than an omitted optional dependency.

## Security Verification

The remaining resolver guard was mutation-tested because it is now the only fail-closed check in that path.

Mutation tested:

- Rewrote the authenticated branch to call the factory without an `RlsContext`.
- This simulates the important fail-open regression.
- The test failed under the mutation and passed after restoration.

Conclusion: the test detects the real regression of bypassing `RlsContext` during authenticated subscription stream creation.

## Verification Gates

Passed:

- `72/72` library tests.
- `cargo check --workspace --all-targets`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- Formatting clean for the three touched files:
  - `graphql.rs`
  - `state_manager.rs`
  - `main.rs`

Formatting scope was intentionally limited to touched files to avoid pulling in unrelated `openapi.rs` drift.

## Remaining Known Issue

Pre-existing rustfmt drift remains at `openapi.rs:376`. It will fail a repository-wide `cargo fmt --check` CI gate independently of this change. The drift is described as a one-line fix but was left out of scope for this work.

## Commit Readiness

Modified files ready to commit:

- `graphql.rs`
- `state_manager.rs`
- `main.rs`

No outstanding work remains for this item besides the separate `openapi.rs` formatting debt.

# Citations

1. stdin