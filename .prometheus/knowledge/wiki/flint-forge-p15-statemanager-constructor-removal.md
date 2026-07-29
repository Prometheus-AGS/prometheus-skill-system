---
type: Reference
id: flint-forge-p15-statemanager-constructor-removal
title: Flint Forge p15 StateManager Constructor Removal
tags:
- flint-forge
- production-readiness
- state-manager
- graphql-subscriptions
- rust
- kbd-phase
sources:
- stdin
- manual:Flint Forge/p15-v1.0-production-readiness
timestamp: 2026-07-16T19:59:51.085984+00:00
created_at: 2026-07-16T19:59:51.085984+00:00
updated_at: 2026-07-16T19:59:51.085984+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p15-v1.0-production-readiness`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge/.claude/worktrees/ecstatic-ardinghelli-9e3717`
- **Captured:** `2026-07-16T19:57:35Z`
- **Position:** `p5-kiln-edge-runtime | status: completed`
- **Progress:** changes `5/5`

## Phase Goal

Close the gap between “workspace compiles and unit tests pass” and a production-ready Flint Forge `v1.0`, focusing on:

- Build integrity
- Operator tooling
- End-to-end validation
- Documentation accuracy
- Production packaging

No new features were targeted.

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

## Completed Change: `StateManager::new` Removed

`StateManager::new` was deleted from `crates/fdb-reflection/src/state_manager.rs`.

Rationale:

- It was dead code.
- It was the only entry point that could inject `None` into the system for the subscription stream factory path.
- Removing it made the production-invalid state unreachable by construction rather than relying only on convention.

Safety checks before removal:

- Full-tree search for `StateManager::new(` found **zero Rust callers**.
- The only matches were archived OpenSpec documents from `p2-c005` describing an obsolete two-argument signature: `engine, db_url`.
- The removed constructor had a newer four-argument signature, so archived references were historical records, not live callers.
- The workspace is not published to a registry, so no external crate was expected to compile against this API.
- `cargo check --workspace --all-targets` passed, confirming no live target, including tests, called the removed function.

## Documentation Updates

Useful documentation from the removed constructor was preserved:

- The sentence that the process must not accept requests until constructor initialization succeeds was folded into `new_with_gates`.
- `new_with_gates` is now the sole constructor.

The `sub_stream_factory` documentation was corrected to state the actual invariant:

- Gateway always passes `Some`.
- The field is never mutated after construction.
- `None` is test-only.
- `None` is not a valid boot state.
- The docs now cross-reference `GraphQlCompiler::compile` so the constructor and compiler explanations stay aligned.

## Validation Gates

Completed validation:

- `72/72` library tests passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `state_manager.rs` was `rustfmt` clean.
- `cargo check --workspace --all-targets` passed.

Known unrelated formatter issue:

- `cargo fmt --check` still reports pre-existing drift at `openapi.rs:376`.
- That file was deliberately left unchanged because the drift predated this change.

## Remaining Follow-Up

Option 3 remains open as a separate scoped change:

- Drop `Option<SubStreamFactory>` from `GraphQlCompiler::compile` and `StateManager::new_with_gates`.
- Let the Rust type system enforce the production invariant directly.

Current state after this change:

- The only removed API path that could inject `None` is gone.
- The remaining function signatures still allow `Option<SubStreamFactory>`.
- The invalid production state is harder to reach and documented clearly, but not yet impossible at the type level.

# Citations

1. stdin
2. manual:Flint Forge/p15-v1.0-production-readiness