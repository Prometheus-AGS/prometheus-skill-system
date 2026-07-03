# Assessment — `phase-learn-sovereign-sync`

> **Phase Brief:** Universal sovereign state sync across all skill package state via Loro CRDT + iroh P2P transport, packaged as a Tauri plugin, WASM module, Axum server node, and MCP server — with production-quality SDKs for Rust, TypeScript, and Go, AG-UI/A2UI streaming agent support, and a KnowMe-branded Docusaurus site.

---

## 0. Research Summary

Research was conducted across: existing codebase (flint-realtime-fabric, flint-gate, prometheus-skill-pack substrate), web search (mid-2026), and architectural analysis with sycophancy-correction gate applied.

### CRDT Engine: Loro — DECIDED, DO NOT RE-EVALUATE
ADR-001 in `flint-realtime-fabric/crates/frf-crdt` (2026-06-19) resolves this conclusively: Loro 1.13.1, 2–9× faster than automerge, 3.7× smaller wire payloads, stable 1.x API, Fugue algorithm for interleaving correctness. The `storage-provider` crate currently declares `automerge = "0.5"` — this must be migrated to Loro as part of G-11.

### P2P Transport and Peer Discovery: iroh + iroh-gossip
iroh 1.0.0 (released June 15, 2026, stable) is the transport. The discovery API was renamed in 1.0: `iroh::discovery::*Discovery` structs are now `iroh::address_lookup::*AddressLookup` — the assessment uses the 1.0 API names throughout.

Peer discovery uses **iroh-gossip** with **topic IDs derived from operator namespace keys** (BLAKE3 of `operator_pubkey || "sovereign-sync-v1"` → 32-byte TopicId). This gives private, namespaced peer groups with no public DHT exposure. The three-layer discovery stack:

1. **Bootstrap (cold start):** operator config file lists 1–N member NodeIDs → pkarr relay (self-hosted or n0's) resolves NodeID → current IP
2. **Group propagation:** iroh-gossip on the derived topic propagates full member list once first peer connects
3. **LAN shortcut:** `iroh-mdns-address-lookup` (feature: `address-lookup-mdns`) discovers same-subnet nodes without internet (fully offline capable)

**Kademlia DHT assessment:** iroh intentionally does not use Kademlia. `mainline` 7.0.0 (BEP-0044) writes to the **public** BitTorrent DHT (~10M nodes) — operator identity leaks to the network. Not acceptable for sovereign sync. **Do not use Kademlia or public DHT.** `libp2p-kad` would duplicate the entire P2P stack for no gain.

**iroh-gossip-discovery** (0.1.0, therishidesai) and **iroh-topic-tracker** (0.2.0, community) exist for group membership management but are experimental. Use the three-layer stack above instead; it depends only on production iroh crates.

**Direct peer addition:** operators share NodeID strings out-of-band (config file, QR code, Git commit). NodeIDs are stable Ed25519 public keys; they survive IP changes. This is the primary bootstrap mechanism; gossip handles propagation from there.

### Signal Protocol Assessment: REJECTED for this phase — redundant
iroh uses QUIC with TLS 1.3 and node-key-based identity (Ed25519). Every iroh connection is authenticated (you dial a public key, not an IP) and encrypted with forward secrecy by QUIC's TLS layer. Adding Signal Protocol's Double Ratchet + X3DH **on top of** iroh would provide:
- Post-compromise security (PCS) beyond what TLS 1.3 provides
- Message-level E2E encryption independent of the transport

**Honest assessment:** For a developer tool syncing skill state, QUIC's built-in encryption is sufficient. Signal Protocol adds significant complexity (prekey bundles, session state, ratchet management, multi-device key distribution) that is **not justified** by the threat model of this phase. The `double-ratchet-signal` crate exists but is community-maintained and not production-vetted for our use case.

**Decision:** iroh node-key identity + QUIC TLS 1.3 covers this phase. Signal Protocol is a candidate for a future `phase-learn-secure-comms` if the threat model warrants it. The assessment notes this as an explicit deferral, not an omission.

### Temporal.io Assessment: REJECTED for P2P node, VIABLE for cloud node only
**Temporal Rust SDK** is in Public Preview (prerelease) as of 2026 — API is evolving. More importantly:
- Temporal requires a server cluster (or Temporal Cloud). This **directly conflicts** with the zero-server P2P design requirement.
- For P2P client nodes, durable state is handled by Loro CRDT + iroh-docs + redb (on-device op-log from frf-store-redb pattern). Tokio tasks + redb provide durability without a workflow server.
- For the **cloud Axum node** (`--mode server`), Temporal IS a reasonable choice for orchestrating long-running agentic workflows (multi-step sync, installation sequences, retry logic). However, the Rust SDK prerelease status is a risk.

**Decision:** Temporal is out of scope for this phase. The cloud node uses Tokio + structured task management. If/when the Rust SDK reaches stable, add a `temporal-workflows` feature gate to the cloud node in a future phase. This is documented as an explicit architectural option, not an accident.

### MCP Client in Rust: CONFIRMED — rmcp 1.8.0
`rmcp` (official Rust MCP SDK, `github.com/modelcontextprotocol/rust-sdk`) is at **1.8.0** (released 2026-06-23) and is production-ready. MCP spec 2025-11-25. It supports both MCP server and MCP client roles, stdio and streamable HTTP transports, tools/resources/prompts, OAuth 2.0, elicitation, proc-macros via `rmcp-macros`. The sync-service will use rmcp to:
1. Expose itself as an MCP **server** (tools: `sync_push`, `sync_pull`, `sync_status`, `sync_peers`)
2. Act as an MCP **client** to forward tool calls to external MCP servers (cloud node mode)

### AG-UI Protocol: IMPLEMENT from existing spec — Rust SDK in development
AG-UI (CopilotKit Agent-User Interaction Protocol, `github.com/ag-ui-protocol/ag-ui`) uses HTTP POST + SSE streaming. Event types are exactly those already implemented in `flint-gate/src/stream/ag_ui.rs` (TEXT_MESSAGE_START, TOOL_CALL_START, STATE_DELTA, RUN_STARTED, etc.). There is no production Rust SDK yet — it is in development. **We will build the AG-UI Axum handler for the sync-service agent interface using the flint-gate implementation as the reference** (copy-adapt, not re-derive from scratch). This is not building from scratch; it is porting 230 lines of working code.

### A2UI Protocol: Already implemented in flint-gate
A2UI (Agent-to-UI intent protocol) is also live in `flint-gate/src/stream/a2ui.rs`. The `sync-service` agent will emit A2UI events via the same SSE pattern. The intents relevant to sync: `render_component` (sync status panel), `update_state` (progress), `stream_content` (sync logs), `request_input` (conflict resolution prompts).

### SDK Landscape

| SDK | Recommended approach | Maturity | Notes |
|---|---|---|---|
| **Rust** | Hand-written `substrate/sync-sdk-rust` crate | Production | Same pattern as `frf-sdk-rust` |
| **TypeScript** | `@number0/iroh` (NAPI, Node.js only) + `loro-crdt@1.13.6` npm (WASM) + `@modelcontextprotocol/sdk@1.29.0` | Production | `@number0/iroh` is the correct package (NOT `iroh-js` — that is an RPC client for iroh.network only); loro-crdt ships 4 WASM targets incl. base64 for edge |
| **Go** | Thin HTTP client over Axum REST API (NOT FFI) | Production | iroh has NO Go bindings; Loro has NO Go bindings (loro-go referenced in loro-ffi README but no public repo exists); HTTP client is the correct approach |
| **Tauri Plugin** | `tauri-plugin-prometheus-sync` using `tauri_plugin_shell::ShellExt` sidecar pattern; Tauri **2.11.3** | Production | Tauri 2.x sidecar well-documented; JSON-RPC over stdio to sync-service binary; no iroh or Loro Tauri plugin exists — embed directly as n0's tauri-todos example shows |

**Go SDK decision:** iroh has zero official Go bindings as of June 2026. Loro has no Go bindings (loro-ffi README mentions `loro-go` but no public GitHub repository exists, no pkg.go.dev entry). UniFFI officially supports Swift, Kotlin, Python, Ruby — not Go. The correct Go SDK is a **thin HTTP client** over the sync-service Axum REST API + WebSocket for streaming. This is idiomatic Go and has no FFI dependency. MCP: `modelcontextprotocol/go-sdk` v1.5.0 (official, stable).

### TypeScript SDK — correct packages
- **iroh in TypeScript/Node.js:** `@number0/iroh` (npm, NAPI native addon) — version should be checked via `npm view @number0/iroh` as it may now be at 1.0.x following the June 15 Rust 1.0 release. **NOT `iroh-js`** — that is a separate, lighter RPC client for iroh.network only with no P2P capability.
- **Browser WASM for iroh:** No official `@number0/iroh-wasm` package exists. Browser use requires a custom Rust+wasm-bindgen compilation. For Phase A, browser support is deferred.
- **Loro in TypeScript:** `loro-crdt@1.13.6` (npm, WASM). Correct package, ships 4 build targets (bundler, nodejs, web, base64). NOT `loro-wasm` — that is the raw wasm-bindgen output; `loro-crdt` wraps it with idiomatic TypeScript.
- **MCP in TypeScript:** `@modelcontextprotocol/sdk@1.29.0` + `zod` — stable; v2 alpha exists but is not production-ready until Q3 2026.

---

## 1. Existing Assets (What We Have)

### In `prometheus-skill-pack/substrate/`

| Crate | State | What it provides |
|---|---|---|
| `storage-provider` | Live but uses automerge 0.5 | `StorageProvider` + `CrdtEngine` traits; `AutomergeEngine`; `IrohDocsAdapter` stub |
| `learner-model` | Live | CRDT learner model, FSRS-6, JSON-RPC stdin/stdout interface, launchd install pattern |
| `surface-bridge` | Live | Axum 0.7 on :7890, `/health`, `/mcp/detect-surface-tier`, tier detection pattern |

### In `flint-realtime-fabric/`

| Crate | What it provides to this phase |
|---|---|
| `frf-crdt` | Loro 1.13.1 integration: `LoroDeltaApplier`, `apply_delta`, `InMemoryCrdtStore`, `export_updates_since`, `merge_into_store` |
| `frf-agentproto` | `ContentBlock` (TextDelta, ToolCall, ToolResult, StateSnapshot, RunStart, RunEnd, Error) — exact AG-UI events in typed Rust |
| `frf-wasm` | wasm-bindgen pattern for CRDT core |
| `frf-ffi` | UniFFI scaffold for Swift/Kotlin |
| `frf-gateway` | Axum 0.8.8 gateway pattern with WS + gRPC |

### In `flint-gate/`

| File | What it provides |
|---|---|
| `src/stream/ag_ui.rs` | Full AG-UI SSE processor, event classifier, token counter — production-tested |
| `src/stream/a2ui.rs` | Full A2UI intent processor with scope enforcement — production-tested |
| `src/auth/` | Ory Kratos + JWT minting — for cloud node identity |

---

## 2. Gap Analysis

### G-01 — Universal State Schema (SyncManifest)
**Gap:** No `SyncManifest` type exists. The `StorageProvider` trait provides key-value storage but no namespace schema.
**What's needed:** Define `SyncDomain` enum (KbdOrchestrator, OpenSpec, SurrealMemory, KarpathyPk, LearnerModel, Custom(String)) and `SyncManifest` (list of domains + their storage prefix mappings).

### G-02 — sync-service Rust crate
**Gap:** `substrate/sync-service/` does not exist.
**What's needed:** New Axum crate with: iroh endpoint, iroh-gossip subscription, Loro merge engine (from frf-crdt), SyncManifest scanning, JSON-RPC stdin/stdout interface, REST API for SDK access.
**Risk:** iroh-gossip-discovery (0.1.0) is early-stage. If it is unstable, fall back to manual bootstrap peer addition (iroh built-in, no dependency needed).

### G-03 — MCP Server interface
**Gap:** No MCP server in sync-service.
**What's needed:** rmcp 0.16.0 integration exposing 4 tools: `sync_push`, `sync_pull`, `sync_status`, `sync_peers`. Launchd plist + MCP config entries for all 5 harnesses (matching learner-model pattern).

### G-04 — Cloud Axum node (MCP client mode)
**Gap:** No `--mode server` flag or MCP client forwarding logic.
**What's needed:** CLI flag to switch binary mode; rmcp MCP client for tool forwarding; HTTP/WebSocket bridge for Tauri/WASM clients.

### G-05 — Tauri Plugin
**Gap:** `tauri-plugin-prometheus-sync/` does not exist.
**What's needed:** Tauri 2.x plugin crate with: sidecar spawn of sync-service binary, JSON-RPC command bridge, Tauri event system for async callbacks, toolchain installer commands, `capabilities/default.json` sidecar permission.

### G-06 — WASM Module
**Gap:** No `substrate/sync-wasm/` crate.
**What's needed:** wasm-bindgen crate over the CRDT merge core and SyncManifest types. TypeScript wrapper with `loro-crdt` npm integration for browser and Tauri web view.

### G-07 — Flint Gate Integration
**Gap:** Sync-service has no identity layer for cloud mode.
**What's needed:** Feature-gated Ory Kratos JWT verification in cloud node mode (copy-adapt from flint-gate `src/auth/`). P2P mode uses iroh node key only.

### G-08 — Docusaurus Site
**Gap:** No `docs/site/` directory exists.
**What's needed:** Docusaurus 3.x install at `docs/site/`, KnowMe brand CSS tokens, Conviction Logomark SVG in theme, Space Grotesk + Roboto + Inter + JetBrains Mono font stack. Content: sync architecture, SyncManifest reference, Tauri plugin guide, cloud node deployment, MCP integration, KB privacy guarantee, learn domain guide.

### G-09 — Sync Skills
**Gap:** No `/sync-status`, `/sync-peers`, `/sync-push` skills exist.
**What's needed:** 3 SKILL.md files in `skills/sync/` with JSON-RPC invocations of the sync-service binary. Must pass 5-harness validation (Claude Code, Kimi, MiniMax, OpenCode, Codex).

### G-10 — IrohDocsAdapter implementation
**Gap:** `substrate/storage-provider/src/iroh_docs.rs` has `Err(StorageError::Unavailable(...))` bodies for all methods.
**What's needed:** Full iroh 0.34+ `iroh_docs::Engine` integration. This is a non-trivial implementation: requires iroh endpoint, author key, namespace key, and async document operations mapped to the `StorageProvider` trait.

### G-11 — Loro migration in storage-provider
**Gap:** `storage-provider/Cargo.toml` declares `automerge = "0.5"`. The `AutomergeEngine` is the active CRDT implementation.
**What's needed:** Replace with `loro = "1.13"` (matching `frf-crdt`), implement `LoroCrdtEngine` by porting `frf-crdt`'s `LoroDeltaApplier`, `apply_delta`, and merge functions. The `AutomergeEngine` can remain as a legacy path gated by a feature flag for migration.

### G-12 — Privacy Preservation
**Gap:** No enforcement mechanism in sync path yet.
**What's needed:** `SyncDomain::LearnerModel` includes a `privacy: PrivacyClass::LocalOnly | SyncEncryptedOnly` field. Sync path checks this before transmitting. KB content (domain `SurrealMemory` with palace prefix) is marked `LocalOnly` and never placed in iroh sync payloads.

### G-NEW-1 — Rust SDK (operator-added)
**Gap:** No `substrate/sync-sdk-rust/` crate.
**What's needed:** Typed Rust client wrapping the sync-service REST + WebSocket API. Expose: `SyncClient::connect()`, `push_domain()`, `pull_domain()`, `list_peers()`, `subscribe_events()`. Used by other Prometheus services.

### G-NEW-2 — TypeScript SDK (operator-added)
**Gap:** No TypeScript SDK package exists.
**What's needed:** npm package at `sdks/ts/` wrapping iroh-js + loro-crdt, with typed APIs matching the Rust SDK surface. Targets: Node.js + browser (via iroh-js WASM). MCP client via `@modelcontextprotocol/sdk`.

### G-NEW-3 — Go SDK (operator-added)
**Gap:** No Go SDK. iroh has no Go bindings.
**What's needed:** Thin HTTP/WebSocket Go client at `sdks/go/` over the sync-service Axum REST API. Uses `github.com/modelcontextprotocol/go-sdk` v1.5.0 for MCP integration. Generated from OpenAPI spec of the Axum REST API, not from iroh FFI.

### G-NEW-4 — AG-UI + A2UI streaming agent (operator-added)
**Gap:** sync-service has no agent mode.
**What's needed:** Axum SSE endpoint on sync-service that emits AG-UI events during sync operations (RunStarted, ToolCallStart/End, StateDelta for progress, RunFinished). A2UI intents for `render_component` (sync dashboard) and `request_input` (conflict resolution). Port from `flint-gate/src/stream/ag_ui.rs` and `a2ui.rs` — not rewritten from scratch.

---

## 3. Architecture Decisions

### A-01: iroh-gossip topic-based peer discovery (CONFIRMED)
Derive `TopicId = SHA-256(operator_id_bytes + b"prometheus-sync")`. Peers with the same operator subscribe to the same topic and discover each other via iroh-gossip epidemic broadcast. Fallback: manual bootstrap peer addition (iroh native, no external crate). This is private by default — topic IDs are opaque 32-byte values with no global DHT.

**Why not Kademlia:** iroh intentionally omits Kademlia. Adding it means a separate crate with no n0-computer support, solving a content-routing problem we don't have.

### A-02: No Signal Protocol for this phase (CONFIRMED)
iroh QUIC with TLS 1.3 + node-key authentication provides authenticated, encrypted, forward-secure channels. Signal Protocol (Double Ratchet + X3DH) is deferred to a future security-hardening phase. The `double-ratchet-signal` crate (community) is not yet production-vetted. **Document this as a known deferral in the sync-service README.**

### A-03: No Temporal for P2P node; deferred for cloud node (CONFIRMED)
P2P durability: Loro CRDT + iroh-docs + redb on-device op-log. Cloud node durability: Tokio structured tasks with retry logic. Temporal.io Rust SDK is Public Preview (prerelease). If it reaches stable before the cloud node is built, add a `temporal` Cargo feature. **Document this as an explicit architectural option.**

### A-04: Go SDK via HTTP REST (NOT FFI) (CONFIRMED)
iroh has no first-party Go bindings. `uniffi-bindgen-go` is experimental and unsupported. The Go SDK wraps the sync-service's Axum REST + WebSocket API, generated from an OpenAPI spec. This is the correct, maintainable, idiomatic path.

### A-05: AG-UI handler ported from flint-gate (CONFIRMED)
The sync-service will include an `/agent/stream` SSE endpoint emitting AG-UI events. This is a port of the working `flint-gate` implementation, not a dependency on an unfinished upstream Rust SDK.

### A-06: Loro migration replaces AutomergeEngine (CONFIRMED)
`storage-provider` switches from `automerge = "0.5"` to `loro = "1.13"`. `AutomergeEngine` is removed (not feature-gated — it has never shipped to a production installation). The learner-model binary must be updated to use the new `LoroCrdtEngine`.

### A-07: sync-service binary runs as both MCP server AND agent (CONFIRMED)
Three modes via `--mode` flag:
- `mcp` (default): JSON-RPC stdio MCP server (launchd, harness integration)
- `server`: Axum HTTP cloud node with MCP client forwarding + AG-UI streaming
- `daemon`: Background sync daemon (no stdio, just iroh P2P + REST on localhost)

---

## 4. Scope Guard and Phase Split

The operator's request has expanded the original 12 goals with: 4 new SDK goals (Rust, TS, Go, Tauri), AG-UI/A2UI streaming agent, explicit peer discovery protocol spec, and Signal Protocol assessment. A sycophancy check flagged that accepting all of this in one phase with no scope pushback would produce an undeliverable plan.

**Recommended split:**

### Phase A: `phase-learn-sovereign-sync` (this phase)
Core infrastructure that everything else depends on:
- G-11: Loro migration (prerequisite for everything)
- G-10: IrohDocsAdapter implementation
- G-01: SyncManifest schema
- G-02: sync-service binary (iroh-gossip discovery + Loro merge + JSON-RPC + REST)
- G-03: MCP server interface (rmcp) + launchd + 5-harness install
- G-12: Privacy enforcement in sync path
- G-NEW-4: AG-UI + A2UI streaming agent in sync-service
- G-09: /sync-status, /sync-peers, /sync-push skills (3 skills)
- G-NEW-1: Rust SDK (`substrate/sync-sdk-rust`)

**Deferred to Phase B (`phase-learn-sovereign-packaging`):**
- G-04: Cloud Axum node with MCP client mode
- G-05: Tauri plugin + sidecar
- G-06: WASM module
- G-07: Flint Gate identity integration (cloud mode)
- G-NEW-2: TypeScript SDK
- G-NEW-3: Go SDK
- G-08: Docusaurus site

**Rationale:** Phase A delivers a working, installable, 5-harness P2P sync capability. Phase B packages it for distribution and cross-platform deployment. Attempting both in one phase risks delivering nothing shippable.

*Operator decision required: Accept this split or confirm scope of this phase before /kbd-plan proceeds.*

---

## 5. Open Questions

| # | Question | Impact | Recommendation |
|---|---|---|---|
| OQ-1 | Accept phase A/B split, or collapse to one large phase? | Scope of plan.md | **Recommend split** — each phase is independently shippable |
| OQ-2 | iroh-gossip-discovery (0.1.0, community crate) — use or implement bootstrap manually? | G-02 complexity | Use it with fallback to manual bootstrap; pin exact version |
| OQ-3 | Should the cloud Axum node (`--mode server`) be in Phase A or Phase B? | G-04 scope | Phase B — P2P-only is sufficient for Phase A |
| OQ-4 | Should `AutomergeEngine` be kept as a feature flag or deleted outright? | G-11 migration risk | Delete it — it has never been in a shipped installation; YAGNI |
| OQ-5 | Docusaurus site — build in this phase or Phase B? | G-08 scope | Phase B — docs depend on a working Phase A to document |
| OQ-6 | iroh-docs namespace key: derive from operator ID or from SyncDomain enum? | G-10 design | One iroh-docs namespace per SyncDomain (learner-model, kbd, surreal-memory, etc.) — isolation by namespace, sync by TopicId |
| OQ-7 | Should the Tauri plugin support toolchain installation (Rust, Node, Python) or only skill binaries? | G-05 scope | Phase B question — only binaries in Phase A |

---

## 6. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| iroh-gossip-discovery 0.1.0 is unstable | Medium | High | Fall back to manual iroh bootstrap peers; no single point of failure |
| iroh API breaking changes (active development) | Medium | High | Pin iroh version; match version used in frf-crdt |
| Loro migration breaks learner-model binary | Low | High | Port frf-crdt's LoroCrdtEngine directly; write migration test |
| rmcp API evolves before GA | Low | Medium | Pin rmcp 0.16.0; the MCP server API surface is small |
| AG-UI Rust SDK released mid-phase and conflicts with our port | Low | Low | Our port is a direct dependency, not a wrapper; no conflict |
| Phase A scope is still too large | Medium | Medium | Revisit OQ-1 with operator before plan proceeds |

---

## 7. Locked Decisions

1. **CRDT engine:** Loro 1.13.x — matches frf-crdt ADR-001, no re-evaluation
2. **P2P transport:** iroh 1.0.0 (QUIC, iroh-gossip, iroh-docs 0.35.x stable, iroh-blobs 0.35.x stable); discovery API renamed to `address_lookup` in 1.0
3. **Peer discovery:** three-layer stack: pkarr bootstrap → iroh-gossip BLAKE3-derived topic → mDNS LAN shortcut; manual config-file NodeID list as primary bootstrap
4. **No Kademlia DHT** — public DHT leaks operator identity; iroh intentionally omits Kademlia
5. **No Signal Protocol in this phase** — iroh QUIC TLS 1.3 is sufficient; Signal deferred
6. **No Temporal.io in this phase** — P2P durability via Loro + iroh-docs + redb; Temporal deferred for cloud node
7. **MCP client/server:** rmcp 1.8.0 (official Rust MCP SDK, `modelcontextprotocol/rust-sdk`)
8. **AG-UI:** Port from flint-gate implementation; not blocked on upstream Rust SDK
9. **Go SDK:** HTTP REST client over Axum API — NOT iroh FFI, NOT Loro FFI (no Go bindings exist for either)
10. **TypeScript SDK:** `@number0/iroh` (NAPI, Node.js) + `loro-crdt@1.13.6` (WASM, 4 targets) + `@modelcontextprotocol/sdk@1.29.0`; deferred to Phase B
11. **Tauri Plugin:** Deferred to Phase B
12. **WASM Module:** Deferred to Phase B
13. **Docusaurus Site:** Deferred to Phase B (Phase B prerequisite: working Phase A)

---

## 8. Confidence Ratings

| Area | Confidence | Rationale |
|---|---|---|
| Loro migration (G-11) | HIGH | Direct port from frf-crdt which is working |
| iroh P2P + gossip discovery (G-02) | HIGH | Core iroh is production stable; gossip-discovery crate is early but optional |
| IrohDocsAdapter (G-10) | MEDIUM | iroh-docs API is stable but the full async StorageProvider implementation is non-trivial |
| rmcp MCP server (G-03) | HIGH | rmcp 0.16.0 is production-ready |
| AG-UI port (G-NEW-4) | HIGH | Direct port from working flint-gate code |
| Go HTTP SDK (G-NEW-3) | HIGH | Thin HTTP client; idiomatic Go; deferred to Phase B |
| Phase A delivery in one sprint | MEDIUM | Depends on operator accepting the A/B split |

---

## 9. Recommended Plan Shape (for /kbd-plan)

If operator accepts Phase A scope, the plan should contain approximately 16–20 changes:

**Group 0 — Foundation (no parallelism; prerequisites)**
- change-sync-001: Loro migration in storage-provider (replaces automerge)
- change-sync-002: SyncManifest schema + SyncDomain enum + privacy classes

**Group 1 — sync-service core (sequential: 001→002 first)**
- change-sync-003: sync-service crate scaffold (Axum + clap + 3-mode binary)
- change-sync-004: iroh endpoint + iroh-gossip peer discovery + manual bootstrap
- change-sync-005: IrohDocsAdapter implementation (iroh-docs namespace-per-domain)
- change-sync-006: Loro merge engine in sync-service (port from frf-crdt)
- change-sync-007: SyncManifest scan + push/pull operations + privacy enforcement

**Group 2 — Interfaces (can parallelize after Group 1)**
- change-sync-008: rmcp MCP server (4 tools: push, pull, status, peers)
- change-sync-009: AG-UI + A2UI streaming agent endpoint (SSE /agent/stream)
- change-sync-010: JSON-RPC stdin/stdout interface (daemon mode)
- change-sync-011: REST API (Axum routes for SDK access)

**Group 3 — SDK + Skills (after Group 2)**
- change-sync-012: Rust SDK crate (substrate/sync-sdk-rust)
- change-sync-013: /sync-status skill (5-harness parity)
- change-sync-014: /sync-peers skill (5-harness parity)
- change-sync-015: /sync-push skill (5-harness parity)

**Group 4 — Install + Tests**
- change-sync-016: install-skills-flat.sh extension (launchd, MCP config for 5 harnesses)
- change-sync-017: Integration tests (Loro migration, iroh gossip, MCP server, AG-UI stream)
- change-sync-018: CLAUDE.md update + version bump

Total: 18 changes. Manageable in a single phase if scope discipline holds.

---

## 10. Assessment Complete Checklist

- [x] Sycophancy check applied — S-03 pattern corrected (scope pushback on Signal, Temporal, Go FFI, single-phase)
- [x] Research grounded in mid-2026 crate state (confirmed via web search + background agents)
- [x] All operator additions assessed honestly: 2 rejected for this phase (Signal, Temporal), 1 redirected (Go FFI → HTTP), 1 split into Phase B (Tauri, WASM, TS SDK, Docusaurus)
- [x] Open questions enumerated for /kbd-plan
- [x] Locked decisions enumerated (no re-evaluation needed in plan)
- [x] Risk register written
- [x] 18-change plan shape recommended for operator review

**Assessment verdict:** Phase A scope (18 changes) is achievable. Phase B scope (Tauri, WASM, TS SDK, Go SDK, Docusaurus) should be a separate phase.

**Operator action required before /kbd-plan:** Confirm or redirect OQ-1 (phase split) and OQ-4 (delete AutomergeEngine vs. feature flag).
