---
id: change-hardening-001-iroh-docs-share-import
title: Iroh docs share/import sync regression
phase: phase-sovereign-sync-hardening
priority: HIGH
effort: M
agent: codex
status: planned
scope:
  - substrate/storage-provider/src/iroh_docs.rs
  - substrate/storage-provider/tests
---

# change-hardening-001 — Iroh docs share/import sync regression

## Context

The real `IrohDocsAdapter` now performs local iroh-docs reads, writes, deletes, and key listing. The remaining hardening gap is multi-node usability: a second adapter needs a supported way to import the same document namespace before sync can prove that writes converge across nodes.

## Scope

- Add a public share/export method for the adapter using iroh-docs share ticket APIs.
- Add a constructor or helper that imports a shared document ticket into a second adapter.
- Add a two-node regression test that writes on node A, imports/syncs on node B, and verifies the same key/value can be read.
- Document retry/timing expectations for the sync test helper.

## Non-Goals

- No custom ticket format if iroh-docs already provides one.
- No production daemon peer discovery changes.
- No broad refactor of `StorageProvider`.

## Validation

- `cargo test` in `substrate/storage-provider`
- Existing `substrate/sovereign-sync` tests still pass
