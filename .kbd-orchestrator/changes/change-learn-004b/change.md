---
id: change-learn-004b
title: "storage-provider trait crate"
type: design
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-001
---

# change-learn-004b: storage-provider trait crate

## Problem

The learner-model crate will need to swap storage backends (local filesystem,
automerge CRDT, Iroh Docs P2P) without changing business logic. Without an
abstraction layer, storage coupling will make testing and backend evolution
painful.

## Proposal

Create a `storage-provider` Rust crate that defines `StorageProvider` and
`CrdtEngine` traits. Ship a `LocalDirAdapter` (production-ready),
an `automerge-rs` `CrdtEngine` (production-ready), and an `IrohDocsAdapter`
stub with `unimplemented!()` bodies for future P2P work.

## Outcome

A trait boundary that `change-learn-005` can depend on, with a working local
adapter and a CRDT engine usable in tests and production.

## Tasks

- [x] Define `StorageProvider` trait with `read`, `write`, `merge`, `list`, and `watch` methods
- [x] Define `CrdtEngine` trait with `apply`, `merge_docs`, and `export_state` methods
- [x] Implement `LocalDirAdapter` (read/write JSON files from a configurable base path)
- [x] Implement `automerge-rs` `CrdtEngine` backed by the schema from change-learn-001
- [x] Add `IrohDocsAdapter` stub with `unimplemented!()` bodies and a TODO comment linking the Iroh Docs RFC
