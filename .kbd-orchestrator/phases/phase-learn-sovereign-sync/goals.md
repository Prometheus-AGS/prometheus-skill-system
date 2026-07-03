# Goals — `phase-learn-sovereign-sync`

## Phase Intent

Implement universal sovereign state sync across all skill package state, integrating with `flint-realtime-fabric` (Loro CRDT, Axum gateway, multi-SDK) and packaging the entire capability as a Tauri application, WASM module, and plain Axum server node — with a KnowMe-branded Docusaurus documentation site.

## Goals

### G-01: Universal State Schema — Define what syncs
- Define a canonical `SyncManifest` that enumerates all skill-package state domains: KBD orchestrator state (current-waypoint, progress.json, phase artifacts), OpenSpec changes and artifacts, surreal-memory state (knowledge graph, scoped memories, task streams, mindmaps), karpathy pk-* skill data, learn domain learner model (mastery per concept, FSRS cards, gap records), and any other registered skill state.
- Every domain gets a CRDT-shaped representation using Loro (already adopted in `flint-realtime-fabric` via ADR-001).

### G-02: State Service — Axum server in this monorepo
- Add a `substrate/sync-service` Rust crate (Axum) to `prometheus-skill-pack` that handles: Loro CRDT merge for all skill state, P2P coordination using iroh (QUIC transport, `iroh-docs` for CRDT key-value, `iroh-blobs` for content-addressed artifacts), gossip protocol for discovering peers owned by the same operator or organization, and a JSON-RPC stdin/stdout interface (matching the `learner-model` pattern) for CLI/sidecar use.
- The service has no central server dependency. All discovery and sync is P2P via iroh.

### G-03: MCP Server Node — integrate sync-service as an MCP server
- The sync-service exposes an MCP server interface so any harness can invoke sync operations as MCP tool calls.
- The existing `install-skills-flat.sh` installs the sync-service as a launchd service on macOS (matching `surface-bridge` and `learner-model` patterns), registers it in the MCP config for Claude Code, Kimi, MiniMax, OpenCode, and Codex.

### G-04: Axum Cloud Node — server-mode with MCP client support
- The same binary, when run with `--mode server`, becomes a cloud-addressable Axum node that: accepts connections from Tauri/WASM clients, supports MCP client calls to external MCP servers (tool invocation forwarding), and bridges P2P iroh peers with HTTP/WebSocket access for cloud-hosted scenarios.

### G-05: Tauri Plugin — bundles ALL skills + toolchains
- Create a Tauri plugin (`tauri-plugin-prometheus-sync`) that: bundles the sync-service binary as a Tauri sidecar, installs ALL skill binaries and required toolchains (Rust, Node, Python) via the Tauri sidecar installation pattern, handles skill lifecycle (start/stop/update), and exposes skill invocation as Tauri commands.
- Supports agentic behavior: the Tauri app can spawn and supervise AI agent sessions with full skill access.
- Builds for macOS, Windows, Linux (desktop) and iOS, Android (mobile) via Tauri 2.x.

### G-06: WASM Module — web platform with shared Rust code
- Compile the CRDT merge core and state serialization to WASM (`frf-wasm` pattern from `flint-realtime-fabric`).
- The WASM module shares Rust source with the Axum service, ensuring identical merge semantics on web vs. native.
- Provide a TypeScript wrapper that integrates with the Tauri web view and standalone browser deployments.

### G-07: Flint Gate Integration — identity and P2P auth
- Integrate with `flint-gate` (Ory Kratos / Oathkeeper) for operator identity when running in cloud/server mode.
- P2P mode uses iroh's built-in node key (Ed25519) for identity — no external auth required for pure P2P.
- The gossip peer-discovery protocol uses a namespace derived from operator ID to find peers owned by the same organization.

### G-08: Docusaurus Documentation Site
- Create a `docs/site/` Docusaurus site at the root of `prometheus-skill-pack` (following the `prometheus-flint-gate` pattern).
- Apply KnowMe brand system: Ember accent (#E04E28 light / #FF6A3D dark), Conviction Logomark SVG, Space Grotesk / Roboto / Inter / JetBrains Mono type stack, design tokens from the brand guide.
- Document: the sync architecture, state manifest, Tauri plugin setup, cloud node deployment, MCP integration, KB privacy guarantee, and the full learn domain.

### G-09: Cross-Platform Parity for Sync Skills
- Add `/sync-status`, `/sync-peers`, and `/sync-push` skills that work identically across Claude Code, Kimi, MiniMax, OpenCode, and Codex (following the platform-parity patterns from `goal-loop-support` phase).
- Skills use the JSON-RPC stdin/stdout interface to the sync-service binary.

### G-10: IrohDocsAdapter — implement the stub
- Implement `substrate/storage-provider/src/iroh_docs.rs` (currently `unimplemented!()`) using iroh 0.34+ APIs.
- The `IrohDocsAdapter` becomes the default sync backend for the learn domain learner model, replacing the local-dir-only `LocalDirAdapter` for multi-device scenarios.

### G-11: CRDT Engine Decision for prometheus-skill-pack
- Adopt Loro (matching `frf-crdt` ADR-001) for all skill-pack CRDT state, replacing the `AutomergeEngine` stub in `substrate/storage-provider`.
- Document the decision in `.kbd-orchestrator/decisions/ADR-003-crdt-loro.md`.

### G-12: Privacy Guarantee Preservation
- The KB content-never-forwarded guarantee from `phase-learn-feynman` is preserved through sync: KB content stored in the learner model is synced via encrypted iroh channels but is NEVER forwarded to external APIs during sync.
- Sync state is encrypted at rest using the iroh node key.

## Operator Requirements (Carried from phase-learn-feynman)
- KB content is NEVER forwarded to external APIs — privacy guarantee applies to sync too.
- Self-reported fluency NEVER closes a Feynman loop — sync cannot be used to bypass mastery closure.
- The honesty rule applies to all sync state: do not flatten or summarize in ways that lose gap signals.

## Out of Scope
- Full Tauri app UI (that is a separate phase — this phase delivers the plugin and sidecar only).
- `learn-to-build` capstone bridge.
- Full Docusaurus content beyond architecture + API reference (blog posts, tutorials are future work).
- `automerge-repo` or any alternative CRDT engine evaluation (Loro is decided via FRF ADR-001).
