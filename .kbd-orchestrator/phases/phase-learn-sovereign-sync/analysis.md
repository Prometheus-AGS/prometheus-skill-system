# Analysis — phase-learn-sovereign-sync
**Produced by:** /kbd-analyze
**Date:** 2026-06-28
**Analyst:** Claude Sonnet 4.6

---

## 0. Scope

This analysis extends the assessment's 18 gaps with four operator-requested
topics:

1. **Orchestrator agent** with full MCP client functionality
2. **liter-llm** for multi-provider LLM access
3. **Advanced skill activation** (intent classification, hybrid, keyword,
   semantic similarity) — patterned on cherry-studio/UAR and opencode
4. **AG-UI endpoint** with A2UI + task schema for skill management and
   inter-node access; Tauri cross-platform SDK for desktop/mobile/web builds

All analysis is evidence-backed from reference codebases, crate docs, and
mid-2026 crate versions.

---

## 1. Orchestrator Agent with MCP Client

### 1.1 Landscape

The UAR (Universal Agent Runtime, `cherry-studio/vendor/universal-agent-runtime`)
provides a blueprint for an orchestrator that:

- Is itself an Axum HTTP process (actor model via `uar/runtime/actor/agent_actor.rs`)
- Runs graph-based execution (`uar/runtime/graph/nodes/agent_node.rs`)
- Exposes skills via REST + WebSocket
- Acts as MCP client to downstream MCP servers

The MCP client layer in our stack should use **rmcp 1.8.0** (official
`modelcontextprotocol/rust-sdk`, released 2026-06-23). This is the only
production-ready Rust MCP SDK with both stdio and streamable-HTTP transports.

### 1.2 Architecture Decision: Build-Adapt

**Verdict: ADAPT from UAR patterns, not copy.**

We do NOT fork UAR wholesale. We:
1. Port the `SkillService` + `SkillRegistry` + `SkillMatchingConfig` pattern
   into a new `sovereign-orchestrator` crate in `substrate/`
2. Wire it to rmcp 1.8.0 for MCP client connections to arbitrary servers
3. Wire liter-llm for LLM inference
4. Expose AG-UI + A2UI SSE endpoints (port from flint-gate references)

The UAR's actor model (using `tokio::sync::mpsc` + internal message bus)
is the right shape — but we adapt it, not copy 80k lines.

### 1.3 MCP Client Design (rmcp 1.8.0)

```rust
use rmcp::{ServiceExt, model::*, transport::SseClientTransport};

// Connect to an MCP server via streamable-HTTP
let transport = SseClientTransport::start("http://server:port/sse").await?;
let client = ().serve(transport).await?;

// List available tools
let tools = client.list_tools(Default::default()).await?;

// Call a tool
let result = client.call_tool(CallToolRequestParam {
    name: "search_memories".into(),
    arguments: Some(serde_json::json!({"query": "prometheus recent work"})),
}).await?;
```

The orchestrator agent:
1. Reads `~/.config/sovereign-sync/mcp-servers.json` (same format as
   Claude Desktop's `claude_desktop_config.json`) at startup
2. Spawns one rmcp client per server (stdio or HTTP transport)
3. Aggregates available tools into a unified tool registry
4. Exposes tools to the LLM via liter-llm's `tools` parameter

**Key integration point**: the orchestrator's MCP client pool maps directly
onto the skill activation pipeline — an MCP tool call IS a skill invocation
for server-hosted skills.

---

## 2. liter-llm Integration

### 2.1 What it is

`liter-llm` is a NAPI-RS Node.js binding for multi-provider LLM access.
Located at: `vendor/universal-agent-runtime/crates/prometheus-skill-system/tools/liter-llm/`

The `DefaultClient` class exposes 140+ providers via model-name prefix routing:
- `"anthropic/claude-sonnet-4-6"` → Anthropic API
- `"openai/gpt-4o"` → OpenAI API
- `"ollama/llama3.3"` → local Ollama
- `"bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0"` → AWS Bedrock
- Provider auto-detected from prefix before first slash

Key NAPI types (from `liter-llm-node/index.d.ts`):
```typescript
export class DefaultClient {
  chat(req: ChatRequest): Promise<ChatResponse>
  chatStream(req: ChatRequest): ChatStreamIterator  // async iterable
  embed(req: EmbedRequest): Promise<EmbedResponse>
  listModels(provider?: string): Promise<ModelList>
  // + imageGenerate, speech, transcribe, moderate, rerank, search, ocr
  // + file management, batch management, response management
}
export function allProviders(): string[]
```

### 2.2 Where it lives in our stack

liter-llm is a **Node.js layer**. The `sovereign-orchestrator` Rust binary
cannot use it directly — it is the TypeScript SDK and Node.js sidecar that
consume liter-llm. The Rust binary uses its own HTTP client against the
liter-llm proxy endpoint OR uses reqwest against provider APIs directly.

**Two-path strategy:**
1. **TypeScript SDK / Node.js sidecar**: use liter-llm's `DefaultClient`
   natively — zero extra work
2. **Rust binary (sovereign-orchestrator)**: expose liter-llm as an HTTP
   proxy process (Node.js `DefaultClient` → HTTP server on `127.0.0.1:7891`)
   so Rust calls it via reqwest. This mirrors the `surface-bridge` pattern
   (Axum on 7890).

OR, for the Rust path only, use **reqwest + per-provider auth** for the
initial implementation (Anthropic, OpenAI, Ollama) and defer the full
liter-llm proxy to Phase B. Phase A ships with direct API clients for
the three most common providers.

**Phase A verdict: Direct Rust API clients for Anthropic/OpenAI/Ollama;
liter-llm proxy wired in TypeScript SDK (full 140-provider access there).**

### 2.3 Skill Execution Config

The UAR's `SkillExecutionConfig` (already in our reference) is the
per-skill model override pattern:

```rust
pub struct SkillExecutionConfig {
    pub preferred_provider: Option<String>,  // "anthropic"
    pub preferred_model: Option<String>,     // "claude-opus-4-8"
    pub max_tokens: Option<usize>,
}
```

This maps directly to liter-llm's model-prefix routing. When a skill is
activated, the orchestrator overrides the session LLM config with the
skill's `execution_config`.

---

## 3. Advanced Skill Activation

### 3.1 Reference Analysis

**From UAR `SkillMatchingAlgorithm` (rust):**
Five algorithms, plug-in config:
```rust
pub enum SkillMatchingAlgorithm {
    Keyword,        // fast, no model
    Embedding,      // Burn-rs + USearch vector search
    Llm,            // LLM classification (stub in UAR → we implement)
    Hybrid,         // keyword + embedding merged, deduped
    LocalEmbedding, // on-device, no API
}
```

The `SkillService::match_skills` implementation (fully read) shows:
- `Keyword`: title/description substring match + keyword trigger scoring
- `Embedding` / `LocalEmbedding`: delegate to `VectorMatcher` (Burn-rs + USearch)
- `Llm`: currently a stub (falls back to keyword) — **we implement this**
- `Hybrid`: keyword + vector, deduplicated by `skill_id`, truncated to `top_k`

**From cherry-studio `SkillActivatedStreamPayload` (TypeScript):**
```typescript
interface SkillActivatedStreamPayload {
  activationMethod: 'intent_classification' | 'hybrid' | 'keyword_match' | 'similarity_score'
  similarityScore?: number
  selectionReason: string
  triggerTokens: string[]
  matchedKeywords: string[]
  contextManagementMethod: 'truncate' | 'summarize' | 'sliding_window'
}
```

This payload is what the frontend receives to render the skill-activation UI
block (`skill-activation-block.tsx`).

**From opencode LLM layer:**
- Provider auto-detection by model prefix
- Per-provider auth + credential chain
- Protocol adapters: anthropic-messages, bedrock-converse, gemini, openai-chat
- No skill activation logic — opencode delegates that to the harness

### 3.2 Activation Method Implementation Plan

**Phase A ships all four activation methods:**

| Method | Implementation | Data needed |
|--------|---------------|-------------|
| `keyword_match` | UAR keyword_match (adopt as-is) | Skill `triggers.keywords` |
| `similarity_score` | Burn-rs MiniLM embeddings + USearch index | Skill descriptions as embeddings |
| `hybrid` | keyword + similarity merged | Both |
| `intent_classification` | rmcp call to skill-classifier MCP OR liter-llm LLM call | Skill catalog as context |

**LLM intent classification prompt (when algorithm = `Llm`):**
```
Given this user query: "{query}"
And these available skills: {skill_list_json}

Classify the user's intent and select the best matching skill(s).
Return JSON: {"matches": [{"skill_id": "...", "score": 0.95, "reason": "..."}]}
```

This replaces the UAR stub `warn!("LLM matching not yet implemented")`.

**Activation metadata** (emitted in SSE `SkillActivatedStreamPayload`):
```rust
pub struct SkillActivationResult {
    pub skill: Skill,
    pub activation_method: ActivationMethod,
    pub similarity_score: Option<f32>,
    pub selection_reason: String,
    pub trigger_tokens: Vec<String>,
    pub matched_keywords: Vec<String>,
}

pub enum ActivationMethod {
    KeywordMatch,
    SimilarityScore,
    Hybrid,
    IntentClassification,
}
```

### 3.3 Embedding Backend

**Verdict: adopt Burn-rs + USearch (same as UAR)**

- `burn` 0.16.x + `burn-ndarray` backend: on-device inference, no ONNX runtime
- `usearch` 3.x: ANNS index, in-process, ARM64-native
- Model: `sentence-transformers/all-MiniLM-L6-v2` (22M params, 80ms on M1)
- Embeddings computed at skill registration, stored in `redb` alongside skill metadata

**Crate additions (sovereign-orchestrator):**
```toml
burn = { version = "0.16", features = ["ndarray"] }
burn-import = "0.16"
usearch = "3"
```

---

## 4. AG-UI Endpoint with A2UI and Task Schema

### 4.1 Protocol Reference

**From flint-gate `ag_ui.rs` (fully read):**
AG-UI (CopilotKit protocol) — 15 event types across 5 categories:
- Text: `TEXT_MESSAGE_START`, `TEXT_MESSAGE_CONTENT`, `TEXT_MESSAGE_END`
- Tools: `TOOL_CALL_START`, `TOOL_CALL_ARGS`, `TOOL_CALL_END`
- State: `STATE_SNAPSHOT`, `STATE_DELTA`, `MESSAGES_SNAPSHOT`
- Run: `RUN_STARTED`, `RUN_FINISHED`, `RUN_ERROR`
- Steps: `STEP_STARTED`, `STEP_FINISHED`

**From flint-gate `a2ui.rs` (fully read):**
A2UI — intent-driven SSE:
- `render_component` (scope: `a2ui:render`)
- `update_state` (scope: `a2ui:state`)
- `navigate` (scope: `a2ui:navigate`)
- `show_modal` (scope: `a2ui:modal`)
- `show_toast` (scope: `a2ui:toast`)
- `request_input` (scope: `a2ui:input`)
- `stream_content` (scope: `a2ui:stream`)

### 4.2 Task Schema for Skill Management

The AG-UI endpoint in `sovereign-orchestrator` exposes a **task schema** for
managing skill-supported functionality. This is a JSON schema that:
- Enumerates available skills as task types
- Defines input/output schemas per skill
- Enables Tauri apps to render appropriate input forms

```json
{
  "schema_version": "1.0",
  "tasks": [
    {
      "task_id": "learn-goal",
      "title": "Start a Learning Goal",
      "description": "Begin a Feynman learning loop for a topic",
      "input_schema": {
        "type": "object",
        "properties": {
          "topic": { "type": "string" },
          "kb": { "type": "string", "nullable": true }
        },
        "required": ["topic"]
      },
      "output_schema": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "plan_summary": { "type": "string" }
        }
      },
      "ag_ui_stream": true,
      "a2ui_intents": ["render_component", "request_input", "stream_content"]
    }
  ]
}
```

**Endpoint shape:**

```
GET  /sovereign/tasks/schema           → JSON task schema
POST /sovereign/tasks/{task_id}/run    → AG-UI SSE stream (CopilotKit protocol)
GET  /sovereign/tasks/{task_id}/status → run status (for polling)
```

**Inter-node access:** when a task is dispatched to a remote node, the
orchestrator proxies the AG-UI SSE upstream from the remote node's
`/sovereign/tasks/{task_id}/run` to the local caller. The iroh P2P transport
provides the underlying channel; the AG-UI SSE is multiplexed over QUIC.

### 4.3 Build vs Adopt

| Component | Verdict | Source |
|-----------|---------|--------|
| AG-UI event types | ADOPT | Port from flint-gate/src/stream/ag_ui.rs |
| A2UI event types | ADOPT | Port from flint-gate/src/stream/a2ui.rs |
| Task schema | BUILD | New — no reference exists |
| AG-UI → SSE bridge | ADAPT | Pattern from flint-gate SSE mux |
| AG-UI → inter-node proxy | BUILD | New — iroh QUIC + axum proxy |

---

## 5. Tauri Cross-Platform SDK Architecture

### 5.1 Platform Matrix

The operator requirement: SDKs for server, web client, desktop (macOS/Win/Linux),
mobile iOS, mobile Android — all Tauri-based.

| Platform | SDK Form | Tauri Role |
|----------|---------|-----------|
| Server | Rust crate (`sovereign-client`) | N/A (pure Rust HTTP client) |
| Web client | TypeScript npm package | N/A (browser fetch + SSE) |
| Desktop | Tauri 2.11.3 app | Shell for web client; sidecar for sync binary |
| iOS | Tauri 2.11.3 mobile (iOS target) | Same web client; sidecar via `tauri-plugin-shell` |
| Android | Tauri 2.11.3 mobile (Android target) | Same web client; sidecar via `tauri-plugin-shell` |

### 5.2 Architecture

**Pattern: n0's `iroh-examples/tauri-todos`**

The sovereign-sync binary runs as a Tauri **sidecar** (`externalBin` in
`tauri.conf.json`). The web frontend communicates via `tauri-plugin-shell`
commands and events, or via localhost HTTP (simpler and works on all
platforms).

```
┌─────────────────────────────────────────┐
│  Tauri App (Desktop / iOS / Android)    │
│  ┌─────────────────────────────────────┐│
│  │  WebView (TypeScript + liter-llm)  ││
│  │  • AG-UI SSE client                ││
│  │  • A2UI event renderer             ││
│  │  • Skill task UI                   ││
│  └─────────────────┬───────────────────┘│
│                    │ localhost:7892       │
│  ┌─────────────────▼───────────────────┐│
│  │  sovereign-sync sidecar (Rust)      ││
│  │  • sovereign-orchestrator           ││
│  │  • iroh P2P sync                   ││
│  │  • Loro CRDT state                 ││
│  │  • MCP client pool (rmcp)          ││
│  │  • Skill matching (Burn + USearch) ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

**No Tauri plugin for iroh or Loro exists** — we embed directly. The
sidecar binary includes both libraries statically.

### 5.3 SDK Package Layout

```
substrate/
├── sovereign-client/        # Rust SDK (server-to-server, no UI)
│   ├── src/lib.rs           # SovereignClient struct
│   └── Cargo.toml
├── sovereign-ts-sdk/        # TypeScript SDK (web + Node.js)
│   ├── src/
│   │   ├── client.ts        # SovereignClient class (fetch + SSE)
│   │   ├── ag-ui.ts         # AG-UI event parser
│   │   ├── a2ui.ts          # A2UI renderer
│   │   ├── tasks.ts         # Task schema + runner
│   │   └── liter-llm.ts     # DefaultClient re-export + routing
│   └── package.json
└── sovereign-go-sdk/        # Go SDK (HTTP REST, no P2P native layer)
    ├── client.go            # SovereignClient over HTTP
    └── go.mod
```

### 5.4 Phase A vs Phase B scope

**Phase A (this phase):**
- `sovereign-orchestrator` Axum binary with all endpoints
- `sovereign-client` Rust SDK
- AG-UI + A2UI + task schema endpoints
- iroh P2P + Loro CRDT sync
- Skill activation (all 4 algorithms)
- MCP client pool

**Phase B (next phase):**
- `sovereign-ts-sdk` npm package + liter-llm integration
- `sovereign-go-sdk`
- Tauri app shell (desktop + mobile)
- Docusaurus documentation site

---

## 6. Library Candidates Summary

See `library-candidates.json` for machine-readable verdicts.

### Adopt (no build required)

| Library | Version | Purpose | Evidence |
|---------|---------|---------|---------|
| `loro` | 1.13.x | CRDT engine | ADR-001, frf-crdt |
| `iroh` | 1.0.0 | P2P QUIC transport | iroh 1.0.0 stable, June 15 |
| `iroh-gossip` | 0.32.x | Epidemic peer discovery | TopicId pattern confirmed |
| `rmcp` | 1.8.0 | MCP client + server | Official SDK, June 23 2026 |
| `redb` | 4.1.0 | Embedded persistence | Crash-safe, simple API |
| `statig` | 0.4.x | Hierarchical FSM | Subsystem lifecycle |
| `burn` | 0.16 | On-device embedding inference | UAR reference pattern |
| `usearch` | 3.x | ANNS vector index | UAR reference pattern |
| `axum` | 0.8.8 | HTTP/SSE server | Already in flint-gate |
| `@number0/iroh` | ~1.0.x | TypeScript iroh NAPI | Correct npm package |
| `loro-crdt` | 1.13.6 | TypeScript Loro WASM | Correct npm package |
| `@modelcontextprotocol/sdk` | 1.29.0 | TypeScript MCP | Stable, confirmed |

### Build (no adoptable candidate)

| Gap | What to build | Complexity |
|-----|---------------|-----------|
| `IrohDocsAdapter` impl | Port stub to real iroh 1.0.0 API | Medium |
| `LoroAdapter` | Loro 1.13.x storage-provider impl | Medium |
| `sovereign-sync` binary | New crate aggregating all above | High |
| LLM intent classification | Implement UAR stub + liter-llm bridge | Medium |
| AG-UI endpoint | Port from flint-gate ag_ui.rs | Low |
| A2UI endpoint | Port from flint-gate a2ui.rs | Low |
| Task schema | New JSON schema + API | Medium |
| Inter-node AG-UI proxy | iroh QUIC + axum SSE proxy | High |
| `sovereign-client` Rust SDK | Thin HTTP client wrapping endpoints | Low |

### Reject / Defer

| Library | Reason |
|---------|--------|
| Signal Protocol / X3DH | REJECTED: iroh QUIC TLS 1.3 sufficient |
| Temporal.io Rust SDK | REJECTED: requires server cluster, prerelease |
| loro-go / iroh-go FFI | REJECTED: no public Go bindings exist |
| Tauri plugin for iroh | REJECTED: does not exist; embed directly |
| `iroh-js` (npm) | REJECTED: RPC client only; use `@number0/iroh` |
| AutomergeEngine | DELETE: YAGNI, never shipped |

---

## 7. Privacy Invariant Impact Analysis

All four operator-added topics were evaluated against the three invariants:

**"KB content is NEVER forwarded to external APIs — privacy guarantee applies to sync too"**
- Orchestrator agent: MCP client may call external MCP servers → KB content
  must be stripped before any MCP call to non-local servers. Enforced by
  content-grounding-kb.sh pattern (already documented).
- liter-llm: Routes to external providers. KB content MUST NOT be included
  in LLM prompts. Grounding layer strips KB snippets before sending to LLM.
- Inter-node sync: Loro CRDT deltas are encrypted at-rest via iroh node keys.
  Deltas never leave the operator's node network (BLAKE3 TopicId scoping).

**"Self-reported fluency NEVER closes a Feynman loop"**
- Unaffected by orchestrator, liter-llm, or AG-UI additions.
- learn-grade routes through sycophancy-correction; this is unchanged.

**"Never tell the learner they did well when they did not"**
- liter-llm provider switching does NOT bypass sycophancy gate.
- learn-grade must use a consistent evaluation model; per-skill model
  overrides (`SkillExecutionConfig`) are blocked for learn-grade by policy.

---

## 8. Open Questions (from Assessment)

### OQ-1: Accept Phase A / Phase B split?
**Recommendation: YES.** This analysis confirms Phase A is 18-22 changes.
Adding Tauri SDK, TypeScript SDK, Go SDK, and Docusaurus in the same phase
would be 30+ changes — exceeds safe KBD phase size.

### OQ-4: Delete AutomergeEngine outright?
**Recommendation: DELETE.** The UAR's `SkillService` pattern confirmed:
the correct pattern is to have multiple storage providers with clear
`SkillOrigin` (Builtin/User). AutomergeEngine was a dead abstraction.
Deleting it removes confusion and ~200 lines of dead code.

---

## 9. Decision Log Entries

```
2026-06-28 — liter-llm in Rust binary
Decision: Direct Rust API clients (Anthropic/OpenAI/Ollama) in Phase A.
liter-llm proxy wired in TypeScript SDK only (Phase A).
Rationale: NAPI-RS is Node.js only; adding a Node.js proxy process to the
Rust binary in Phase A adds complexity without immediate payoff.
Provenance: analysis

2026-06-28 — Skill activation: LLM method implementation
Decision: Implement the UAR stub (LLM algorithm falls back to keyword)
using liter-llm via HTTP proxy (from TypeScript sidecar) or direct API
call (from Rust binary). Both paths supported.
Provenance: analysis, UAR service.rs evidence

2026-06-28 — Tauri sidecar pattern confirmed
Decision: sovereign-sync binary as Tauri externalBin sidecar.
No Tauri plugin for iroh/Loro needed.
Provenance: n0 iroh-examples/tauri-todos reference, Tauri 2.11.3 docs

2026-06-28 — Burn + USearch for embedding-based skill matching
Decision: Adopt Burn-rs 0.16 + USearch 3.x. On-device MiniLM inference.
No cloud embedding API required (privacy-preserving).
Provenance: UAR SkillService reference, UAR VectorMatcher pattern
```
