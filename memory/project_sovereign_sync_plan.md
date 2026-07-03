---
name: project-sovereign-sync-plan
description: 'phase-learn-sovereign-sync created 2026-06-28; universal state sync via Loro CRDT + iroh P2P, Tauri plugin, WASM, Axum cloud nodes, KnowMe Docusaurus site'
metadata:
  type: project
---

Phase `phase-learn-sovereign-sync` created 2026-06-28. Stage: assessment_ready.

**Why:** Extend prometheus-skill-pack with sovereign universal state sync so all skill state (KBD orchestrator, artifacts, surreal-memory, pk-\* skill data, learn domain learner model) can be synced across devices and nodes, fully P2P with no central server.

**Scope (12 goals):**

- G-01: Universal SyncManifest covering all skill state domains
- G-02: substrate/sync-service Rust crate (Axum + Loro + iroh P2P + gossip)
- G-03: MCP server node interface (launchd + all 5 harness MCP configs)
- G-04: Cloud Axum node with MCP client support
- G-05: Tauri 2.x plugin with sidecar for ALL skill binaries + toolchains
- G-06: WASM module (shared Rust core, wasm-bindgen pattern from frf-wasm)
- G-07: flint-gate integration (Ory Kratos identity for cloud/server mode)
- G-08: KnowMe-branded Docusaurus documentation site
- G-09: /sync-status, /sync-peers, /sync-push skills with 5-harness parity
- G-10: IrohDocsAdapter implementation (stub at substrate/storage-provider/src/iroh_docs.rs)
- G-11: Loro adoption for prometheus-skill-pack (matches frf-crdt ADR-001)
- G-12: Privacy guarantee preservation through sync (KB content never forwarded)

**Key decisions already made (do NOT re-evaluate):**

- CRDT engine: **Loro 1.13.1** — decided in frf-crdt ADR-001 (2026-06-19); 2-9× faster than automerge, 3.7× smaller, Fugue algorithm, stable 1.x API
- P2P transport: **iroh** (QUIC, iroh-docs, iroh-blobs) — no WebRTC needed; iroh is purpose-built for this
- Identity (cloud mode): **Ory Kratos / Oathkeeper** via flint-gate

**Key repos to read at assess time:**

- `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric` — workspace structure, frf-crdt (Loro), frf-wasm, UniFFI pattern, Axum 0.8.8 gateway
- `/Users/gqadonis/Projects/know-me/flint-gate` — Kratos auth, AG-UI stream handler, Axum proxy patterns
- `substrate/storage-provider/src/iroh_docs.rs` — the stub to implement
- `/Users/gqadonis/Projects/know-me/branding/` — KnowMe brand assets for Docusaurus site

**How to apply:** When starting /kbd-assess for this phase, read position-reminder.txt first. The key_context block in current-waypoint.json has all critical paths. Do NOT re-research Loro vs automerge — the decision is final.
