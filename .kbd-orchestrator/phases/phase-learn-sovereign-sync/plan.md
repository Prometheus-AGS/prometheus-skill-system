# Plan — phase-learn-sovereign-sync
**Produced by:** /kbd-plan
**Date:** 2026-06-28
**Backend:** native KBD (openspec/ present but phase uses native changes)

---

## 0. Architectural Verdict (Sycophancy-Corrected)

Before the change list, this section records the vetted architectural decision
with its reasoning. A sycophancy-correction gate (adversarial review agent)
was applied to the proposed plan concept and returned **PARTIAL** with
specific corrections. Those corrections are incorporated below.

### 0.1 The Proposed Concept and What Was Wrong

The analysis proposed a `sovereign-orchestrator` that would:
- Port the full UAR SkillService (keyword + embedding + hybrid + LLM-intent
  algorithms) as a lightweight in-process copy
- Detect UAR presence at runtime and skip itself
- Be the single binary that handles all harnesses uniformly

**The gate identified three scope-creep patterns:**

1. **"Lightweight UAR-spec mode" = a second SkillService.** There is no such
   thing as a minimal port of a four-algorithm matching engine. The minimum
   viable standalone capability is a `SkillIndex` that reads SKILL.md
   frontmatter and builds a `name → path` map. 50 lines, not 2,000.

2. **Four matching algorithms in Phase A** is speculative generality. Keyword
   matching covers 95% of real invocation patterns. Embedding requires a
   bundled 100MB+ model or a network API call — both unacceptable for a CLI
   binary invoked per-skill-lookup. LLM-intent has no proven use case yet.

3. **"Same binary works everywhere"** hides that each harness has different
   startup path, config location, permission model, and MCP transport.
   Unifying these into one binary means building a platform abstraction layer
   — a significant scope item, not a single binary.

### 0.2 The Correct Architecture (Accepted)

The gate's recommended architecture, accepted as the plan's foundation:

```
prometheus-skill-pack/
  skills/                         # SKILL.md files
    sync/                         # Sync skills (Phase A)
    learn/                        # Learn domain skills (existing)
    ...
  substrate/
    sovereign-sync/               # The binary: MCP server + P2P sync daemon
      src/
        main.rs                   # Startup: --mode mcp|daemon|server
        skill_index.rs            # Reads SKILL.md files → name+desc map
        mcp_server.rs             # rmcp MCP server: tools + SKILL.md invocation
        sync_engine.rs            # iroh P2P + Loro CRDT merge
        rest_api.rs               # Axum HTTP for SDK and Tauri sidecar
        ag_ui.rs                  # Port from flint-gate
        a2ui.rs                   # Port from flint-gate
```

**The sovereign-sync binary has exactly one job per mode:**

| Mode | Job |
|------|-----|
| `--mode mcp` (default, stdio) | MCP server: expose skills as tools via rmcp |
| `--mode daemon` | Background P2P sync daemon (iroh + Loro) on localhost:7892 |
| `--mode server` | Axum HTTP server: REST API + AG-UI SSE + task schema |

**Discovery is decoupled from invocation:**
- Native skill harnesses (Claude Code, Kimi, Codex, OpenCode) read SKILL.md
  directly. The binary is **not needed** for these.
- Non-SKILL.md harnesses (BossFang, custom Tauri, Axum services) use the
  binary's MCP server or REST API.

### 0.3 UAR Integration: Clean Boundary, No Collision

**When used inside UAR:** prometheus-skill-pack skills are registered into
UAR's existing SkillService via the filesystem provider (UAR already scans
skill directories and builds its own registry from SKILL.md). No sovereign
SkillService runs. The sovereign binary runs as a sidecar MCP server
(stdio or HTTP) that UAR invokes as an external tool — not an embedded
component. Zero collision.

**How UAR avoids collision:** UAR's `SkillStorageProvider` registers each
skill by `skill_id` derived from the SKILL.md `name` field. If the sovereign
binary also exposes MCP tools with the same names, the host will see
duplicates. The fix: the MCP server prefixes all tool names with `sovereign:`
(e.g., `sovereign:sync-status`) to make them distinct from UAR-registered
skills. UAR-registered skills are plain name (e.g., `sync-status`). No
collision possible.

**Detection is simple:** The sovereign binary's MCP server checks for an env
var `UAR_SKILL_SERVICE_URL` at startup. If present, it operates in
**passthrough mode** (only exposes sync-specific tools: `sync_push`,
`sync_pull`, `sync_status`, `sync_peers`). If absent, it exposes all skills.

### 0.4 Skill Activation: Keyword Only in Phase A

Phase A ships **keyword matching only** for the sovereign MCP server's skill
lookup. This is sufficient because:
- Claude Code, Kimi, Codex, OpenCode perform their own skill activation from
  SKILL.md — the binary is not in the loop for these.
- For BossFang/LibreFang and Axum harnesses, keyword lookup on skill
  `name`+`description`+`triggers.keywords` is deterministic and fast.
- Embedding matching requires a local model (100MB+) or an API call per
  lookup. Neither is acceptable for Phase A.

Phase B can add embedding matching when there is a concrete case requiring it.
LLM-intent classification is **removed from the roadmap** until a use case
justifies it.

### 0.5 BossFang/LibreFang Integration: MCP over stdio

BossFang is an Agent OS with its own agent management and tool registry. The
correct integration boundary is **MCP server over stdio**:

1. No ABI coupling (a native Rust plugin would require versioning contracts
   we cannot control).
2. MCP is the lingua franca for all harnesses we target.
3. Duplicate-name risk is mitigated by the `sovereign:` prefix.

The sovereign-sync binary registers itself in BossFang's tool registry as an
MCP server, exposing `sovereign:sync_push`, `sovereign:sync_pull`, etc.

### 0.6 "Run Everywhere" Implementation (Actual, Not Aspirational)

| Harness | How skills are discovered | How sovereign binary is used |
|---------|--------------------------|------------------------------|
| Claude Code | `~/.claude/skills/` (SKILL.md) | Optional: launchd MCP server |
| Kimi Desktop | `~/.kimi-code/skills/` (SKILL.md) | Optional: MCP in config.toml |
| Codex Desktop | `~/.codex/skills/` (SKILL.md) | Optional: MCP in mcp.json |
| OpenCode Desktop | `~/.opencode/skills/` (SKILL.md) | Optional: MCP in config |
| Claude Desktop | `~/.claude/skills/` (SKILL.md) | MCP via `claude_desktop_config.json` |
| BossFang/LibreFang | Binary MCP server registration | MCP over stdio (primary) |
| Custom Tauri app | Binary as sidecar `externalBin` | localhost:7892 HTTP |
| Axum server harness | Binary as subprocess | REST API on :7892 |
| CLI harness | SKILL.md files directly | Binary not needed |
| UAR (inside cherry-studio) | UAR SkillService reads SKILL.md | Binary as external MCP sidecar |

**The install-skills-flat.sh script already handles the first 5 harnesses.**
Phase A extends it to register the sovereign binary's MCP server in each.

---

## 1. Changes (22 total)

Changes are grouped by dependency. Groups within the same tier can parallelize.

### Tier 0 — Foundation (must complete before everything else)

---

#### change-sync-001: Delete AutomergeEngine; implement LoroAdapter [library: cand-001]

**Gap:** G-11
**Priority:** blocking — everything in the CRDT stack depends on this
**Files:**
- `substrate/storage-provider/Cargo.toml` — replace `automerge = "0.5"` with `loro = "1.13"`
- `substrate/storage-provider/src/lib.rs` — remove `AutomergeEngine`; add `LoroAdapter`
- `substrate/storage-provider/src/loro_adapter.rs` — new file; port from `frf-crdt`
- `substrate/learner-model/Cargo.toml` — update dependency
- `substrate/learner-model/src/lib.rs` — update CRDT calls

**Key pattern from frf-crdt to port:**
```rust
use loro::LoroDoc;
pub struct LoroAdapter { doc: LoroDoc }
impl CrdtEngine for LoroAdapter {
    fn apply_delta(&mut self, delta: &[u8]) -> Result<(), CrdtError> { ... }
    fn export_updates_since(&self, version: &[u8]) -> Result<Vec<u8>, CrdtError> { ... }
    fn merge_into(&mut self, other: &[u8]) -> Result<(), CrdtError> { ... }
}
```

---

#### change-sync-002: SyncManifest schema + SyncDomain + PrivacyClass [build-required]

**Gap:** G-01, G-12
**Priority:** blocking — sync-service and IrohDocsAdapter depend on domain namespacing
**Files:**
- `substrate/storage-provider/src/sync_manifest.rs` — new file

```rust
pub enum SyncDomain {
    KbdOrchestrator,
    OpenSpec,
    SurrealMemory,
    LearnerModel,
    Custom(String),
}

pub enum PrivacyClass {
    LocalOnly,           // never transmitted, not even encrypted
    SyncEncryptedOnly,   // encrypted in transit; iroh node keys
    SyncPlaintext,       // no extra encryption needed (non-sensitive)
}

pub struct DomainConfig {
    pub domain: SyncDomain,
    pub storage_prefix: String,
    pub privacy: PrivacyClass,
    pub iroh_namespace_key: [u8; 32],  // derived per-domain
}

pub struct SyncManifest {
    pub operator_id: [u8; 32],
    pub domains: Vec<DomainConfig>,
}
```

**Privacy enforcement rule:** `PrivacyClass::LocalOnly` domains are never
placed in iroh sync payloads — this is where the KB content invariant is
enforced structurally. SurrealMemory palace prefix → LocalOnly.

---

### Tier 1 — sync-service core (after Tier 0)

---

#### change-sync-003: sovereign-sync crate scaffold [build-required, library: cand-009]

**Gap:** G-02
**Files:**
- `substrate/sovereign-sync/Cargo.toml` — new crate
- `substrate/sovereign-sync/src/main.rs` — clap CLI, mode dispatch
- `substrate/sovereign-sync/src/config.rs` — config file loading

**Binary modes:**
```
sovereign-sync [--mode mcp|daemon|server] [--config PATH] [--port PORT]
```

**Config format** (`~/.config/sovereign-sync/config.toml`):
```toml
[node]
skills_dir = "~/.claude/skills"
operator_id = "<hex>"

[peers]
bootstrap = ["<NodeId1>", "<NodeId2>"]

[server]
port = 7892
```

This change delivers only the scaffold and config loading. Subsequent
changes add functionality.

---

#### change-sync-004: IrohDocsAdapter implementation [library: cand-002]

**Gap:** G-10
**Files:**
- `substrate/storage-provider/src/iroh_docs.rs` — replace all `Err(Unavailable)` stubs

**Key iroh 1.0.0 API notes:**
- `iroh::EndpointAddr` (was `NodeAddr`)
- `iroh::EndpointId` (was `NodeId`)
- `iroh::address_lookup::DnsAddressLookup` (was `DnsDiscovery`)
- One iroh-docs namespace per `SyncDomain` (namespace key derived from
  `BLAKE3(operator_id || domain.to_bytes())`)

---

#### change-sync-005: iroh P2P endpoint + iroh-gossip peer discovery [library: cand-002, cand-003]

**Gap:** G-02
**Files:**
- `substrate/sovereign-sync/src/p2p.rs` — new file

**Three-layer discovery:**
```rust
// Layer 1: pkarr bootstrap (from config file)
let bootstrap_peers: Vec<EndpointId> = config.peers.bootstrap;

// Layer 2: iroh-gossip topic
let topic_id = blake3::hash(&[&operator_id, b"sovereign-sync-v1"].concat());
let gossip = Gossip::from_endpoint(endpoint.clone(), Default::default(), &router).await?;
gossip.subscribe(topic_id, bootstrap_peers).await?;

// Layer 3: mDNS LAN shortcut (feature = "address-lookup-mdns")
```

**statig FSM** for connection lifecycle:
`Disconnected → Bootstrapping → Connected → Syncing → Idle`

---

#### change-sync-006: Loro merge engine in sovereign-sync [library: cand-001]

**Gap:** G-02, G-06 (merge core)
**Files:**
- `substrate/sovereign-sync/src/crdt.rs` — new file

Port `frf-crdt`'s merge functions into the sync-service context:
- `apply_incoming_delta(domain: &SyncDomain, delta: &[u8])` — privacy-check
  then merge into Loro doc for that domain
- `export_outgoing_delta(domain: &SyncDomain, since_version: Vec<u8>)` —
  get changes to push
- `scan_manifest(manifest: &SyncManifest, base_path: &Path)` — walk the
  filesystem to find CRDT state files per domain

**Privacy gate in apply_incoming_delta:**
```rust
if domain.config.privacy == PrivacyClass::LocalOnly {
    return Err(SyncError::PrivacyViolation(domain.name()));
}
```

---

#### change-sync-007: redb persistence for sync state [library: cand-005]

**Gap:** G-02
**Files:**
- `substrate/sovereign-sync/src/store.rs` — new file

Tables:
- `peers`: `EndpointId → last_seen_timestamp + metadata`
- `versions`: `domain_key → Loro version vector (bytes)`
- `sessions`: `session_id → sync_session_state`

Replaces any Temporal.io dependency (confirmed rejected).

---

### Tier 2 — Interfaces (after Tier 1, can parallelize within tier)

---

#### change-sync-008: rmcp MCP server (stdio mode) [library: cand-004]

**Gap:** G-03
**Files:**
- `substrate/sovereign-sync/src/mcp_server.rs` — new file

**Four sync tools + SkillIndex tools:**
```rust
#[tool(description = "Push local sync domains to peers")]
async fn sync_push(domains: Option<Vec<String>>) -> CallToolResult { ... }

#[tool(description = "Pull updates from peers")]
async fn sync_pull(domains: Option<Vec<String>>) -> CallToolResult { ... }

#[tool(description = "Get sync status for all domains")]
async fn sync_status() -> CallToolResult { ... }

#[tool(description = "List known peers")]
async fn sync_peers() -> CallToolResult { ... }
```

**SkillIndex (keyword-only, Phase A):**
```rust
pub struct SkillIndex {
    skills: Vec<SkillEntry>,  // loaded from SKILL.md files
}
impl SkillIndex {
    pub fn search(&self, query: &str) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| {
            s.name.to_lowercase().contains(&query.to_lowercase())
            || s.description.to_lowercase().contains(&query.to_lowercase())
            || s.keywords.iter().any(|k| query.to_lowercase().contains(k))
        }).collect()
    }
}
```

**UAR passthrough mode:** if `UAR_SKILL_SERVICE_URL` env var is set, only
expose the four sync tools (no skill index tools, no collision with UAR).

**Tool naming:** all tools prefixed `sovereign:` when `--prefix-tools` flag
is set (BossFang / collision-avoidance mode).

---

#### change-sync-009: AG-UI + A2UI streaming endpoint [library: cand-014, cand-015]

**Gap:** G-NEW-4
**Files:**
- `substrate/sovereign-sync/src/ag_ui.rs` — port from `flint-gate/src/stream/ag_ui.rs`
- `substrate/sovereign-sync/src/a2ui.rs` — port from `flint-gate/src/stream/a2ui.rs`
- `substrate/sovereign-sync/src/routes/agent.rs` — new Axum SSE route

**Endpoint:** `POST /sovereign/agent/run` → SSE stream of AG-UI events

**Task schema endpoint:**
```
GET  /sovereign/tasks/schema     → JSON task schema (enumerated skills)
POST /sovereign/tasks/{id}/run   → AG-UI SSE stream
GET  /sovereign/tasks/{id}/status
```

Task schema dynamically enumerates `SkillIndex` entries — each skill becomes
a task type with input/output schemas derived from the SKILL.md frontmatter.

**A2UI intents used:** `render_component` (sync dashboard), `update_state`
(progress), `stream_content` (sync logs), `request_input` (conflict resolution).

---

#### change-sync-010: REST API (Axum routes, daemon/server modes) [library: cand-009]

**Gap:** G-02, G-NEW-1 (SDK dependency)
**Files:**
- `substrate/sovereign-sync/src/routes/mod.rs`
- `substrate/sovereign-sync/src/routes/sync.rs`
- `substrate/sovereign-sync/src/routes/skills.rs`
- `substrate/sovereign-sync/src/routes/health.rs`

**Routes:**
```
GET  /health
GET  /api/v1/skills               → list all skills (SkillIndex)
GET  /api/v1/skills/search?q=...  → keyword search
POST /api/v1/sync/push
POST /api/v1/sync/pull
GET  /api/v1/sync/status
GET  /api/v1/sync/peers
```

This is the interface used by:
- Tauri sidecar (localhost:7892)
- Axum server harnesses
- Go SDK (HTTP REST client)
- sovereign-client Rust SDK

---

#### change-sync-011: MCP client pool (rmcp, mcp-servers.json) [library: cand-004]

**Gap:** G-NEW-1
**Files:**
- `substrate/sovereign-sync/src/mcp_client_pool.rs` — new file

Reads `~/.config/sovereign-sync/mcp-servers.json` (same format as Claude
Desktop `claude_desktop_config.json`). Spawns one rmcp client per server:

```rust
let transport = match server.transport {
    "stdio" => StdioClientTransport::from_command(&server.command, &server.args)?,
    "http"  => SseClientTransport::start(&server.url).await?,
    _ => bail!("unsupported transport"),
};
let client = ().serve(transport).await?;
```

Aggregates all tools into `MpcToolRegistry`. When the MCP server receives a
tool call that maps to an external server, the client pool forwards it.

**Privacy gate:** MCP calls to external servers strip KB-derived content
from arguments before forwarding. The KB content invariant is enforced here.

---

### Tier 3 — SDK + Skills (after Tier 2)

---

#### change-sync-012: sovereign-client Rust SDK [build-required, library: cand-009]

**Gap:** G-NEW-1
**Files:**
- `substrate/sovereign-client/Cargo.toml` — new crate
- `substrate/sovereign-client/src/lib.rs`

```rust
pub struct SovereignClient { base_url: Url, client: reqwest::Client }
impl SovereignClient {
    pub async fn connect(url: &str) -> Result<Self>
    pub async fn list_skills(&self) -> Result<Vec<SkillEntry>>
    pub async fn search_skills(&self, query: &str) -> Result<Vec<SkillEntry>>
    pub async fn sync_push(&self, domains: &[&str]) -> Result<SyncResult>
    pub async fn sync_pull(&self, domains: &[&str]) -> Result<SyncResult>
    pub async fn sync_status(&self) -> Result<SyncStatus>
    pub async fn stream_task(&self, task_id: &str, input: serde_json::Value)
        -> Result<impl Stream<Item = AgUiEvent>>
}
```

---

#### change-sync-013: /sync-status skill [build-required]

**Gap:** G-09
**Files:**
- `skills/sync/sync-status/SKILL.md` — new skill

```yaml
---
name: sync-status
description: Show current P2P sync status for all domains, peer count, and last sync timestamp
version: '1.0.0'
license: MIT
metadata:
  author: travis-james
  category: sync
  tags: [sync, p2p, status, sovereign]
---
```

Instructions invoke the sovereign-sync binary via JSON-RPC or the MCP server.
Must pass 5-harness validation (Claude Code, Kimi, MiniMax, OpenCode, Codex).

---

#### change-sync-014: /sync-peers skill [build-required]

**Gap:** G-09
**Files:**
- `skills/sync/sync-peers/SKILL.md` — new skill

Lists all known peers with their EndpointId, last-seen timestamp, and
domain sync coverage.

---

#### change-sync-015: /sync-push skill [build-required]

**Gap:** G-09
**Files:**
- `skills/sync/sync-push/SKILL.md` — new skill

Triggers a push of specified domains (or all domains if none specified) to
all connected peers. Emits AG-UI stream events for progress.

---

### Tier 4 — Install + Tests + Docs (after Tier 3)

---

#### change-sync-016: install-skills-flat.sh extension [build-required]

**Gap:** G-03 (harness install), G-09 (5-harness parity)
**Files:**
- `scripts/install-skills-flat.sh` — extend existing script

Additions:
1. Compile `substrate/sovereign-sync` binary (if Rust toolchain available)
2. Install binary to `~/.local/bin/sovereign-sync` (or `~/.cargo/bin/`)
3. Register launchd plist (macOS) for `--mode daemon` background service
4. Inject MCP server entry into:
   - `~/Library/Application Support/Claude/claude_desktop_config.json`
   - `~/.kimi-code/config.toml` (Kimi Desktop)
   - `~/.config/OpenCode/mcp.json` (OpenCode)
   - `~/.codex/mcp.json` (Codex)
   - MiniMax: skip (no MCP config support confirmed)
5. Write `~/.config/sovereign-sync/config.toml` (first-run defaults)

**UAR co-existence:** install script detects `UAR_SKILL_SERVICE_URL` or
UAR process on port 8080. If detected, sets `--prefix-tools sovereign:` in
the launchd plist and MCP config entries to avoid collision.

---

#### change-sync-017: Integration tests [build-required]

**Gap:** G-12 (privacy enforcement), test coverage
**Files:**
- `substrate/sovereign-sync/tests/integration/` — new directory
- `substrate/storage-provider/tests/loro_migration_test.rs`

Tests:
1. Loro migration: old automerge bytes → new Loro doc (migration path)
2. Privacy enforcement: `LocalOnly` domain rejected at sync boundary
3. iroh gossip: two-node topic subscription, peer discovery
4. MCP server: rmcp stdio client connects, lists tools, calls `sync_status`
5. AG-UI stream: `POST /sovereign/agent/run` emits `RUN_STARTED` → `RUN_FINISHED`
6. SkillIndex: keyword search returns correct SKILL.md matches
7. UAR passthrough: with `UAR_SKILL_SERVICE_URL` set, only sync tools exposed
8. BossFang collision avoidance: `sovereign:` prefix applied when `--prefix-tools` set

---

#### change-sync-018: Workspace Cargo.toml + version bump + CLAUDE.md [build-required]

**Gap:** general
**Files:**
- `Cargo.toml` (workspace root if exists) or `substrate/Cargo.toml`
- `CLAUDE.md` — add sovereign-sync section
- `package.json` — version bump to `1.5.0`
- `plugin.json` — version bump

CLAUDE.md additions:
- `substrate/sovereign-sync/` crate description
- Modes of operation table
- UAR co-existence guide
- BossFang MCP integration instructions
- Port usage (7892 daemon/server, 7890 surface-bridge, 7891 reserved)

---

### Changes Summary

| # | ID | Gap | Tier | Complexity | Library |
|---|---|---|---|---|---|
| 1 | change-sync-001 | G-11 | 0 | M | cand-001 (loro) |
| 2 | change-sync-002 | G-01, G-12 | 0 | M | — |
| 3 | change-sync-003 | G-02 | 1 | L | cand-009 (axum) |
| 4 | change-sync-004 | G-10 | 1 | M | cand-002 (iroh) |
| 5 | change-sync-005 | G-02 | 1 | M | cand-002,003 |
| 6 | change-sync-006 | G-02 | 1 | M | cand-001 |
| 7 | change-sync-007 | G-02 | 1 | L | cand-005 (redb) |
| 8 | change-sync-008 | G-03 | 2 | M | cand-004 (rmcp) |
| 9 | change-sync-009 | G-NEW-4 | 2 | M | cand-014,015 |
| 10 | change-sync-010 | G-02 | 2 | M | cand-009 |
| 11 | change-sync-011 | G-NEW-1 | 2 | M | cand-004 |
| 12 | change-sync-012 | G-NEW-1 | 3 | L | — |
| 13 | change-sync-013 | G-09 | 3 | L | — |
| 14 | change-sync-014 | G-09 | 3 | L | — |
| 15 | change-sync-015 | G-09 | 3 | L | — |
| 16 | change-sync-016 | G-03, G-09 | 4 | M | — |
| 17 | change-sync-017 | G-12 | 4 | M | — |
| 18 | change-sync-018 | — | 4 | L | — |

**Total: 18 changes.** Phase A scope confirmed. Phase B (TypeScript SDK, Go SDK, Tauri app shell, Docusaurus) is a separate phase.

---

## 2. Gaps Deferred to Phase B

| Gap | Reason |
|-----|--------|
| G-04 (cloud Axum node, MCP client mode) | Requires deployment infrastructure beyond Phase A scope |
| G-05 (Tauri plugin) | Binary sidecar pattern confirmed; full plugin is Phase B |
| G-06 (WASM module) | Phase B |
| G-07 (Flint Gate identity integration) | Cloud node only; Phase B |
| G-NEW-2 (TypeScript SDK) | Depends on Phase A REST API being stable |
| G-NEW-3 (Go SDK) | Same dependency |
| G-08 (Docusaurus site) | Phase B |
| Embedding matching (Burn+USearch) | Phase B; no justified Phase A use case |
| LLM-intent classification | Removed from roadmap; no proven use case |

---

## 3. Change Ordering Rationale

1. **Loro migration first (001)** because every other change depends on the
   CRDT engine. Touching Cargo.toml first surfaces dependency conflicts early.

2. **SyncManifest schema second (002)** because the privacy invariant
   (`LocalOnly`) must be defined before sync_engine or IrohDocsAdapter uses it.

3. **Crate scaffold third (003)** so all subsequent changes have a home.

4. **IrohDocsAdapter + P2P + Loro merge + redb (004-007)** can parallelize
   after the scaffold exists; they have no internal dependency on each other.

5. **Interfaces (008-011)** require the core (004-007) complete. Within Tier 2,
   the MCP server (008) and REST API (010) can parallelize with AG-UI (009) and
   the MCP client pool (011).

6. **SDK + Skills (012-015)** require the REST API (010) to be testable.

7. **Install + Tests + Docs (016-018)** are the last tier; they validate
   everything before the phase closes.

---

## 4. First Change to Apply

**`change-sync-001`**: Delete AutomergeEngine in `substrate/storage-provider`;
implement `LoroAdapter` using `loro = "1.13"` patterns from `frf-crdt`.

Run first because it is the root dependency of the entire phase. If Loro
migration surfaces unexpected issues (e.g., version conflict with a shared
Cargo workspace), we want to discover that before writing any new code.

---

## 5. Open Questions Resolved

| OQ | Resolution |
|----|-----------|
| OQ-1: Phase split | CONFIRMED — Phase A = 18 changes as above; Phase B = packaging |
| OQ-2: iroh-gossip-discovery vs manual | Use iroh-gossip directly (no experimental community crate); manual bootstrap from config |
| OQ-3: Cloud node in Phase A | DEFERRED to Phase B |
| OQ-4: AutomergeEngine | DELETE (change-sync-001) |
| OQ-5: Docusaurus | Phase B |
| OQ-6: iroh-docs namespace key | One namespace per SyncDomain, derived from BLAKE3(operator_id || domain) |
| OQ-7: Tauri plugin scope | Phase B; Phase A only needs the binary |

---

## 6. Statement of Architectural Reasoning

The architecture arrived at by this plan is the **most elegant path** for the
following reasons:

**For skill files in native harnesses (Claude Code, Kimi, Codex, OpenCode,
Claude Desktop):** SKILL.md files are already the right abstraction. No binary
is needed. Install-skills-flat.sh copies them. This is already shipping and
proven across 5 harnesses.

**For non-SKILL.md harnesses (BossFang, custom Tauri, Axum services):** A
single Rust binary exposing an MCP server is the correct boundary. MCP is
the universal tool protocol that all modern AI harnesses understand. The
binary is small, statically compiled, and has no runtime dependencies. MCP
over stdio requires zero network configuration.

**For UAR co-existence:** The SKILL.md source-of-truth means UAR's
`SkillStorageProvider` loads the same skills the binary would expose —
there is no divergence. When the skill pack is used inside UAR, UAR drives
skill activation and the binary only serves sync-specific tools (with
`sovereign:` prefix). No code duplication, no registry collision.

**For P2P sync:** iroh + iroh-gossip + Loro CRDT is the right stack because
it is private (no public DHT), encrypted (QUIC TLS 1.3), and offline-capable
(mDNS LAN shortcut). redb provides durability without a server. This replaces
Temporal.io (which would require a server cluster and a prerelease Rust SDK).

**For BossFang/LibreFang:** MCP over stdio is the lowest-coupling integration
point. No ABI, no FFI, no versioning contract on our side. BossFang registers
the sovereign binary as an MCP server and invokes it like any other tool. The
`sovereign:` tool prefix ensures no name collision with BossFang's own tools.

**Advantages across all modes:**
- **No runtime dependencies**: The sovereign binary is a single statically-linked
  Rust binary. No Node.js, no Python, no Docker.
- **Offline-first**: P2P sync works without internet (LAN mDNS). Skills work
  without the binary (native SKILL.md harnesses).
- **Privacy-preserving**: KB content is enforced `LocalOnly` at the type level.
  No configuration mistake can sync KB content to remote peers.
- **Zero-collision with UAR**: prefix-tools mode and env-var detection ensure
  the sovereign binary never fights UAR's SkillService for tool namespace.
- **Incremental deployment**: Start with SKILL.md files only (zero binary needed).
  Add the binary for P2P sync. Add MCP registration for BossFang/Tauri.
  No step requires the others.
