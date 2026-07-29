---
type: Reference
id: prometheus-entity-sync-v4-scaffold-and-phase-goals
title: Prometheus Entity Sync v4 Scaffold and Phase Goals
tags:
- prometheus-entity-sync
- rust-workspace
- postgres-cdc
- pglite
- sync-engine
- openspec
- typescript-sdk
sources:
- stdin
- manual:prometheus-entity-management/phase-v4-prometheus-entity-sync
timestamp: 2026-07-16T19:35:29.204208+00:00
created_at: 2026-07-16T19:35:29.204208+00:00
updated_at: 2026-07-16T19:35:29.204208+00:00
revision: 0
---

## Context

- **Phase:** `phase-v4-prometheus-entity-sync`
- **Project:** `prometheus-entity-management`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`
- **Captured:** `2026-07-16T19:34:44Z`
- **Source context:** `manual:prometheus-entity-management/phase-v4-prometheus-entity-sync`

## Objective

Build `prometheus-entity-sync`: a **Rust-native, MIT-licensed, bidirectional sync engine** connecting Postgres to:

- PGlite in browser environments
- SQLite on mobile/desktop
- `pglite-oxide` for Tauri desktop

Target parity with PowerSync's feature set while avoiding license restrictions and exposing Rust, Dart, and TypeScript SDKs.

## P0 — Sync Server Core

Planned Rust crates and server components:

- `pes-core`
  - `SyncRule` types
  - `BucketOp`
  - LSN cursors
  - Entity change model
- `pes-rules`
  - TOML sync rules DSL parser
  - `BucketAssigner` for per-user bucket membership
  - Inputs: JWT claims plus lookup queries against Postgres
- `pes-oplog`
  - Per-bucket ordered op log
  - Checksum support
  - Backed by `frf-store-redb`
- `pes-snapshot`
  - Chunked initial sync from Postgres
  - Batch size: 10K rows
  - Uses `frf-postgres-cdc`
- `pes-protocol`
  - `PSyncV1` wire protocol
  - WebSocket transport
  - MessagePack binary framing
- `pes-gateway`
  - WebSocket server using `tokio-tungstenite`
  - Extends `frf-gateway`
- `pes-server`
  - Config file support
  - Health endpoint
  - Prometheus-format metrics
  - Docker image
- Integration requirements:
  - Use `frf-postgres-cdc` for WAL streaming; do **not** reimplement CDC.
  - Use `frf-crdt` / Loro for CRDT write path and conflict resolution.

## P1 — TypeScript Client SDK and PEM Integration

Planned packages:

- `@prometheus-ags/entity-sync-core`
  - Protocol client
  - Reconnect with exponential backoff
  - JWT refresh
  - LSN tracking
- `@prometheus-ags/entity-sync-pglite`
  - PGlite extension
  - `syncBucket()` applies delta operations to local PGlite
- `@prometheus-ags/entity-sync-react`
  - `useEntitySync()` hook
  - `useSyncStatus()` hook
- PEM integration:
  - `registerEntityTransport` integration
  - `prometheusSync(config)` transport factory
- Acceptance target:
  - PEM Vite example app demonstrates bidirectional sync of the `entities` table.

## P2 — Dart / Flutter SDK

Planned Dart package:

- `prometheus_entity_sync`
  - Pure Dart WebSocket client
  - No FFI dependency
  - SQLite backend via `drift`

## Completed Work: `v4-repo-scaffold`

`kbd-apply` completed for `v4-repo-scaffold`:

- **Tasks:** 14/14 completed
- **Verification:** passed
- **Archive status:** archived

### OpenSpec Layout Fix

A structural bug from the plan phase was corrected:

- Original layout nested all 14 changes under one umbrella directory:
  - `openspec/changes/2026-07-13-v4-prometheus-entity-sync/v4-*/`
- The `openspec` CLI only understands flat top-level change directories.
- Changes were flattened to:
  - `openspec/changes/v4-*/`
- Waypoint files were updated to document this requirement for future `/kbd-apply` runs.

## Current Repository State

New sibling repository:

- Path: `/Users/gqadonis/Projects/prometheus/prometheus-entity-sync`
- Git initialized
- First commit: `1c8d91a`

### Rust Workspace

An 8-crate Rust workspace is live and wired to FRF path dependencies:

- `pes-core`
- `pes-rules`
- `pes-oplog`
- `pes-snapshot`
- `pes-protocol`
- `pes-gateway`
- `pes-server`
- `pes-sdk-rust`

Validation status:

```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
```

Both commands pass cleanly.

### TypeScript Workspace

A 4-package pnpm workspace is live:

- `entity-sync-core`
- `entity-sync-pglite`
- `entity-sync-react`
- `entity-sync-tauri`

### Repository Assets

The scaffold includes:

- CI workflow
- README with architecture diagram
- MIT `LICENSE`
- `.gitignore`

### OpenSpec Archive

The scaffold change was archived to:

```text
openspec/changes/archive/2026-07-16-v4-repo-scaffold/
```

Archive used `--skip-specs`, which is appropriate because the change was infrastructure/scaffold-only and did not introduce a user-facing capability.

## Next Action

Run:

```bash
/kbd-apply v4-pes-core-types
```

Purpose: define shared domain types that unblock the rest of the critical path, including:

- `PgLsn`
- `SyncRule`
- `BucketAssignment`
- `Op`
- related core sync-domain models

# Citations

1. stdin
2. manual:prometheus-entity-management/phase-v4-prometheus-entity-sync