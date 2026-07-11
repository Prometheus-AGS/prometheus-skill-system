---
type: Reference
id: flint-forge-p3-5-ci-postgres-hardening-phase-setup
title: Flint Forge p3.5 CI Postgres Hardening Phase Setup
description: "Project:** Flint Forge - **Phase:** `p3.5-ci-postgres-hardening` - **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge` - **Captured:** `2026-07-03T20:08:28Z` - **Status:** `assessment_ready` - **Commit:** `8366033` - **Previous phase:** `p3-auth-rls-keto` - **Next act"
tags:
- flint-forge
- ci-postgres
- pgvector
- pg-graphql
- postgrest
- fdb-realtime
- cargo-test
- kbd-phase
links:
- flint-forge-p3-auth-rls-keto-reflection-and-merge-summary
- flint-forge-p3-c019-postgrest-engine-read-write-progress
- flint-forge-p3-c020-listen-change-source-pr-6-status
sources:
- stdin
- manual:Flint Forge/p3.5-ci-postgres-hardening
timestamp: 2026-07-03T20:15:04.857294+00:00
created_at: 2026-07-03T20:15:04.857294+00:00
updated_at: 2026-07-03T20:15:04.857294+00:00
revision: 0
---

## Context

- **Project:** Flint Forge
- **Phase:** `p3.5-ci-postgres-hardening`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/flint-forge`
- **Captured:** `2026-07-03T20:08:28Z`
- **Status:** `assessment_ready`
- **Commit:** `8366033`
- **Previous phase:** `p3-auth-rls-keto`
- **Next action:** `/kbd-assess p3.5-ci-postgres-hardening`

This phase closes the gap left by [Flint Forge p3 Auth/RLS/Keto Reflection and Merge Summary](/flint-forge-p3-auth-rls-keto-reflection-and-merge-summary.md): real-time and REST paths were proven manually against live Postgres in p3, but were not yet CI-gating. It also folds in carried-forward bookkeeping for delivered p3 work such as [Flint Forge p3-c019 PostgREST Engine Read/Write Progress](/flint-forge-p3-c019-postgrest-engine-read-write-progress.md) and [Flint Forge p3-c020 LISTEN Change Source PR #6 Status](/flint-forge-p3-c020-listen-change-source-pr-6-status.md).

## Phase Gate

`cargo test --workspace` must be green and meaningful with a database in CI:

- CI provisions Postgres with required extensions.
- DB-backed tests run automatically rather than only via manual `--ignored` runs.
- Pre-existing `fdb-gateway` test debt is cleared.
- Workspace clippy is clean with `-D warnings`.

## Goals

### G1 — CI Postgres service

Provision a CI database through `scripts/ci-check.sh` / Dagger:

- Postgres 18.
- `pgvector`.
- `pg_graphql`.
- Export `DATABASE_URL` for DB-backed tests.

This resolves **OQ-9**.

### G2 — Un-ignore live Postgres tests

Move live Postgres coverage from manual-only to CI-gating by removing `#[ignore]` or gating tests on `DATABASE_URL` presence.

Affected coverage:

- `fdb-realtime/tests/listen_live_pg.rs`.
- `fdb-reflection` pgvector tests.
- `fdb-reflection` meta-listener tests.
- Add DB-backed embedding REST path test:
  - Query shape: `select=*,child(*)`.
  - Expected behavior: correct nested JSON.
- Add/enable DB-backed coverage for `PgRest::execute`.

### G3 — Clear `fdb-gateway` test debt

Fix existing test/lint issues so the workspace can run reliably:

- Isolate `keto_sync_config_ignores_non_numeric_env` so it no longer flakes under parallel `set_var` use.
- Clear `uninlined_format_args` in `tests/a2ui_seed_test.rs`.

### G4 — Workspace clippy gate

Ensure this command passes end-to-end:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Known blocker at phase start:

- `hello-component` example crate triggers macro-generated `used_underscore_items` lint.
- Required fix: allow or annotate narrowly, not globally.

### G5 — Reconcile p3 bookkeeping

Record carried-forward p3 status accurately:

- Mark c019 PostgREST engine as delivered.
- Mark c020 LISTEN source as delivered.
- Mark c017 superseded by c020.
- Resolve or re-scope c018 against the merged introspection work.

## Transition Work Completed

The p3.5 phase was created manually because the helper script/environment root was unavailable.

Completed actions:

1. Committed and pushed p3 reflection artifacts to `main`:
   - `reflection.md`.
   - `handoffs/reflect.md`.
   - phase-state snapshot.
2. Confirmed the reconciliation decision as a plan mutation: close via `/kbd-new-phase`.
3. Confirmed next-phase focus: CI Postgres hardening plus test debt.
4. Ran `/kbd-new-phase` manually using step-by-step `jq` and atomic writes.
5. Created `p3.5-ci-postgres-hardening` with:
   - `goals.md` containing G1–G5.
   - skeleton `progress.json`.
6. Updated waypoint:
   - `previousPhase = p3-auth-rls-keto`.
   - `next_action = /kbd-assess`.
7. Updated `project.json activePhase`.
8. Committed and pushed the transition as `8366033`.

## Limitations and Follow-up Notes

- `phase:before` hook did not fire because `KBD_ORCHESTRATOR_ROOT` and shared hook libraries such as `hooks.sh` and `waypoint.sh` were not resolvable in the environment.
- The hook failure is treated as best-effort; committed state persisted.
- `progress.json` was not hand-edited to register c019/c020 or supersede c017/c018. That reconciliation is intentionally owned by p3.5 goal G5 rather than fabricated into the old phase's tracked state.
- The session footer still showed `p3-auth-rls-keto | 7/9` because it came from `position-reminder.txt` generated at session start. The committed waypoint correctly points to `p3.5-ci-postgres-hardening`.

## Current Repository State

- `main` contains:
  - all p3 code for embedding, PostgREST engine, LISTEN source, and live tests;
  - p3 reflection;
  - p3.5 phase transition.
- Active phase: `p3.5-ci-postgres-hardening`.
- Phase status: `assessment_ready`.
- No open PRs or in-flight work were reported.

## Next Step

Run:

```bash
/kbd-assess p3.5-ci-postgres-hardening
```

Expected assessment output:

- Confirm CI/test-debt gaps.
- Produce the change plan for DB-backed CI hardening.
- Include G5 bookkeeping reconciliation for c019/c020/c017/c018.

# Citations

1. stdin
2. manual:Flint Forge/p3.5-ci-postgres-hardening