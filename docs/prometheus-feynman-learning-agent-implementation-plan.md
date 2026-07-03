# Prometheus Feynman Learning Agent — Implementation Plan

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |
| **Related Documents** | `prometheus-feynman-learning-agent-architecture.md`, `prometheus-feynman-learning-agent-functional-spec.md` |

---

## 1. Implementation Philosophy

This plan follows a **risk-driven, phase-gated** approach. Each phase delivers a vertically-sliced, end-to-end functional increment. We prioritize:

1. **Core learning loop first**: The Feynman loop engine is the unique value proposition. It must work before polish.
2. **Local-first data second**: Offline capability is a key differentiator. PGlite + ElectricSQL sync is critical path.
3. **Agent-native UI third**: A2UI/AG-UI integration is the "wow factor" but depends on the loop and data layers.
4. **Platform expansion fourth**: Tauri desktop/mobile, Karpathy Loop, and business model gating are layered on the solid foundation.
5. **Marketplace fifth**: The Coach Catalog, Creator Studio, Master Certification, and Video Conferencing (Phases P11–P20) are built after the base platform is stable. See the **Implementation Plan Addendum** (`prometheus-feynman-learning-agent-implementation-plan-addendum.md`) for the marketplace build plan.

**Estimated total effort**: 6-8 months for a team of 4-5 engineers (2 Rust, 2 frontend, 1 infra/DevOps) for the base platform (P0–P10). The marketplace addendum (P11–P20) adds approximately 10-12 months for a team of 6-8 engineers. Solo or small team: 18-24 months for the full platform including marketplace.

**Related Documents**: `prometheus-feynman-learning-agent-implementation-plan-addendum.md` (Marketplace: Coach Catalog, Creator Studio, Master Certification, Video Conferencing, Revenue Engine).

---

## 2. Phase Overview

| Phase | Name | Duration | Goal | Deliverable |
|---|---|---|---|---|
| **P0** | Foundation & Scaffold | 4 weeks | Working monorepo, build pipeline, CI/CD, database schema. | `cargo check` green, `pnpm build` green, GitHub Actions CI. |
| **P1** | Local-First Data Layer | 4 weeks | PGlite in browser, ElectricSQL sync, shared schema. | Offline-capable CRUD for goals and concepts. |
| **P2** | Feynman Loop Core | 6 weeks | Full Feynman loop engine: explain, grade, gaps, recurse, mastery. | `/learn/feynman-loop` API works end-to-end with mock LLM. |
| **P3** | Agent UI & Surfaces | 4 weeks | AG-UI SSE streaming, A2UI surface rendering, assistant-ui integration. | Agent can render dynamic forms and cards during a loop. |
| **P4** | MCP Client & Tools | 3 weeks | Rust MCP client, tool discovery, tool invocation, sandboxed execution. | Agent can search the web and execute code during a loop. |
| **P5** | LLM Integration & Grading | 3 weeks | Real LLM integration (OpenAI/Anthropic), sycophancy correction, transfer problem generation. | Production-grade grading with real LLM. |
| **P6** | Flint-Forge Integration | 3 weeks | Connect to Flint-Forge Postgres, real-time events, auth/identity, reflection engine. | Multi-tenant cloud backend with RLS. |
| **P7** | Tauri Desktop & Mobile | 4 weeks | Tauri 2 wrapper, native capabilities, code signing, updater. | Signed installers for Windows/macOS/Linux. |
| **P8** | Karpathy Loop & Analytics | 3 weeks | LVS computation, experiment framework, Quarry reflection integration. | Autonomous improvement dashboard for admins. |
| **P9** | Subscription & Business | 3 weeks | Stripe integration, feature gating, tier enforcement, billing dashboard. | Freemium-to-Plus-to-Pro flow working. |
| **P10** | Polish, Performance, Launch | 4 weeks | Load testing, security audit, accessibility pass, documentation, marketing site. | Public beta launch. |

---

## 3. Phase Details

### Phase 0: Foundation & Scaffold (Weeks 1-4)

**Goal**: Establish a working monorepo with all build tools, CI/CD, and the basic database schema. All subsequent phases depend on this.

#### P0.1 Monorepo Structure

```
prometheus-feynman-learning-agent/
├── Cargo.toml                     # Workspace root
├── Cargo.lock
├── rust-toolchain.toml            # Pin to 1.85+
├── package.json                   # pnpm workspace root
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── .github/
│   └── workflows/
│       ├── ci.yml                 # Rust clippy, tests, frontend build
│       ├── release-tauri.yml      # Tauri build + code signing
│       └── deploy-web.yml         # Docker build + deploy to ECS/K8s
├── crates/
│   ├── pfla-core/                 # Domain types, errors, shared traits
│   ├── pfla-api/                  # Axum server, routes, handlers
│   ├── pfla-feynman/              # Feynman loop engine
│   ├── pfla-mcp/                  # MCP client implementation
│   ├── pfla-sync/                 # ElectricSQL sync bridge
│   └── pfla-auth/                 # JWT, RLS context, identity
├── ui/                            # React 19 + Vite 7 SPA
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── routes/                # TanStack Router
│   │   ├── components/            # shadcn/ui + custom
│   │   ├── surfaces/              # A2UI surface renderers
│   │   ├── stores/                # prometheus-entity-management + zustand
│   │   ├── lib/
│   │   │   ├── pglite.ts          # PGlite initialization
│   │   │   ├── electric.ts        # ElectricSQL sync setup
│   │   │   └── ag-ui.ts         # AG-UI event bus
│   │   └── hooks/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.ts
│   └── tsconfig.json
├── tauri/                         # Tauri 2 wrapper (optional for web-only)
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── build.rs               # Embeds Vite build into binary
│   │   └── src/
│   │       ├── main.rs
│   │       └── lib.rs
│   └── package.json
├── docs/
│   ├── architecture.md
│   ├── functional-spec.md
│   └── api/
│       └── openapi.yaml
├── migrations/                    # SQLx migrations
│   └── 001_initial_schema.sql
└── scripts/
    ├── scaffold.sh                # One-command dev setup
    ├── dev-web.sh                 # Run web dev server + API
    └── dev-tauri.sh             # Run Tauri dev mode
```

#### P0.2 Toolchain Setup

| Task | Tool | Notes |
|---|---|---|
| Rust workspace | `cargo` | `resolver = "2"`, `edition = "2021"`, `rust-version = "1.85"` |
| JS workspace | `pnpm` | `pnpm-workspace.yaml` with `ui/`, `tauri/` |
| Frontend build | `vite` | `vite.config.ts` with React 19, Tailwind v4, path aliases |
| Type checking | `tsc` | `noEmit` in CI, strict mode |
| Linting | `eslint` + `prettier` | shadcn/ui recommended config |
| Rust linting | `clippy` (pedantic) | `cargo clippy -- -D warnings` in CI |
| Rust formatting | `rustfmt` | `cargo fmt --check` in CI |
| Database migrations | `sqlx` | `sqlx migrate run`, `sqlx prepare` for offline compile |
| CI/CD | GitHub Actions | Parallel jobs: Rust check/test, frontend build/test, Tauri build |
| Pre-commit | `lefthook` or `husky` | Rust fmt, clippy, JS lint, commit message lint |

#### P0.3 Database Schema (Migrations)

Write `migrations/001_initial_schema.sql` with all core tables (learners, goals, concepts, artifacts, grades, retention_checks) as defined in the Architecture Document. Include:
- UUID primary keys, `timestamptz` timestamps.
- JSONB for flexible metadata (curriculum, recursion_floor, gaps).
- `pgvector` extension for semantic search over explanations (optional in P0, but include the extension enable).
- RLS policies for all tables.
- ElectricSQL compatibility: no `serial` (use `bigserial` or `uuid`), no `timestamp` without zone.

**P0 Exit Criteria**:
- `cargo check` passes for all crates.
- `pnpm build` passes for the UI.
- `docker-compose up` starts Postgres with schema applied.
- `sqlx prepare` succeeds (offline query verification).
- CI pipeline runs green on PRs.

---

### Phase 1: Local-First Data Layer (Weeks 5-8)

**Goal**: The app works offline. PGlite in the browser stores all learner data. ElectricSQL syncs to cloud Postgres when online.

#### P1.1 PGlite Integration (UI)

1. Install `@electric-sql/pglite`, `@electric-sql/pglite-sync`, `@electric-sql/pglite-react`, `drizzle-orm`, `drizzle-kit`.
2. Create `ui/src/lib/pglite.ts`:
   - Initialize PGlite with `dataDir: "idb://pfla-db"`.
   - Load extensions: `live`, `electricSync`.
   - Execute the full schema DDL on first run (or check schema version).
   - Export `pg` instance and `useLiveQuery` hook.
3. Create Drizzle ORM schema definitions in `ui/src/db/schema.ts`.
4. Create `ui/src/db/migrations.ts` to run PGlite migrations (same SQL as Postgres, but adapted for PGlite limitations).

#### P1.2 ElectricSQL Sync Setup

1. Deploy ElectricSQL sync service (or use Electric Cloud if available).
2. Configure shapes for learner data:
   - `goals`: `WHERE learner_id = $1`
   - `concepts`: `WHERE goal_id IN (SELECT id FROM goals WHERE learner_id = $1)`
   - `artifacts`: `WHERE goal_id IN (...)`
   - `grades`: `WHERE concept_id IN (...)`
3. In `ui/src/lib/electric.ts`:
   - `syncShapeToTable` for each shape.
   - Handle `onShapeError` and `onShapeData` callbacks.
   - Expose `syncStatus` (online / offline / syncing / error).
4. Add a sync status indicator in the UI (subtle top-right banner).

#### P1.3 Offline CRUD for Goals & Concepts

1. Implement UI screens:
   - **Goal List**: `useLiveQuery` to list goals from PGlite. Responsive grid.
   - **Goal Create**: Form with title, description, proficiency target. Writes to PGlite.
   - **Goal Detail**: Shows curriculum tree (read-only in P1). Status badges.
2. Implement API endpoints:
   - `POST /api/v1/goals` — validated, writes to Postgres, triggers Electric sync.
   - `GET /api/v1/goals` — list with RLS.
   - `GET /api/v1/goals/:id` — detail with concepts.
3. Verify offline behavior: disconnect network, create a goal, reconnect, verify sync.

#### P1.4 Conflict Resolution Strategy

1. Define conflict resolution rules:
   - `title`, `description`: user-wins (last-write-wins).
   - `status`, `mastery_score`: user-wins.
   - `created_at`, `mastered_at`: server-wins (immutable).
   - `curriculum`: server-wins if learner hasn't modified; user-wins if modified.
2. Implement custom conflict handler in ElectricSQL sync configuration.
3. Test conflict scenarios: modify same goal offline on two devices, reconnect, verify resolution.

**P1 Exit Criteria**:
- User can create, read, and update goals and concepts offline.
- Data syncs to cloud Postgres within 30 seconds of coming online.
- Conflicts resolve deterministically without data loss.
- Sync status indicator accurately reflects state.

---

### Phase 2: Feynman Loop Core (Weeks 9-14)

**Goal**: The full Feynman learning loop works end-to-end with a mock LLM. Explanation generation, grading, gap identification, recursion, and mastery closure are all functional.

#### P2.1 Loop Engine Architecture (Rust)

Create `crates/pfla-feynman/` with the loop engine:

```rust
// crates/pfla-feynman/src/lib.rs
pub mod spec;
pub mod plan;
pub mod execute;
pub mod reflect;
pub mod recurse;
pub mod closure;

use pfla_core::{Concept, Goal, Artifact, Grade};

pub struct FeynmanLoop {
    pub goal_id: uuid::Uuid,
    pub concept_id: uuid::Uuid,
    pub depth: u8,
    pub audience: Audience,
}

pub enum Audience { Novice, Peer, Skeptic }

impl FeynmanLoop {
    pub async fn run(&self, ctx: &LoopContext) -> Result<LoopResult, LoopError> {
        // 1. Spec: load concept state
        let concept = self.spec(ctx).await?;
        // 2. Plan: generate outline
        let outline = self.plan(&concept, ctx).await?;
        // 3. Execute: produce explanation (streamed)
        let explanation = self.execute(&outline, ctx).await?;
        // 4. Reflect: grade and identify gaps
        let grade = self.reflect(&explanation, ctx).await?;
        // 5. Recurse: spawn child loops if needed
        let child_results = self.recurse(&grade, ctx).await?;
        // 6. Closure: evaluate mastery criteria
        let closure = self.closure(&grade, &child_results, ctx).await?;
        Ok(LoopResult { explanation, grade, child_results, closure })
    }
}
```

#### P2.2 Mock LLM Adapter

1. Create a `MockLlmProvider` trait implementation that returns deterministic responses:
   - `generate_explanation(topic, audience)`: returns a pre-written explanation from a test corpus.
   - `grade_explanation(text, rubric)`: returns a mock grade with configurable gaps.
2. Wire the mock provider into the loop engine for deterministic testing.
3. Write comprehensive unit tests for each phase:
   - Spec: correct concept loaded, recursion floor respected.
   - Plan: outline contains core idea + analogies.
   - Execute: explanation follows outline structure.
   - Reflect: gaps identified when expected, no gaps when explanation is complete.
   - Recurse: child loops spawned for each gap, depth limit enforced, floor guard prevents recursion.
   - Closure: mastery only when all three criteria met.

#### P2.3 API Endpoints & Streaming

1. Implement `POST /api/v1/learn/feynman-loop`:
   - Accepts `concept_id`, `goal_id`, `depth`, `audience`.
   - Returns `201` with `loop_id`.
   - Starts a background task running the loop engine.
2. Implement `GET /api/v1/learn/feynman-loop/:loop_id/stream`:
   - SSE endpoint that streams loop events:
     - `event: phase_start` (phase name)
     - `event: outline_generated` (outline JSON)
     - `event: explanation_chunk` (text fragment)
     - `event: explanation_complete` (full artifact)
     - `event: grade_complete` (grade JSON)
     - `event: child_loop_spawned` (child loop_id, concept_id)
     - `event: closure_result` (mastery boolean)
3. Implement `POST /api/v1/learn/grade`:
   - Accepts `artifact_id` and `explanation_text`.
   - Runs the reflect phase synchronously (or queued).
   - Returns the grade result.
4. Implement `POST /api/v1/learn/retain`:
   - Accepts `concept_id` and `quiz_answers`.
   - Evaluates retention and updates `retention_checks`.

#### P2.4 Frontend Loop UI

1. Create screens:
   - **Loop Start**: Concept card with "Start Explain" button.
   - **Explain Phase**: Streaming text area with real-time explanation display. Pause/Resume button.
   - **Outline Preview**: Card with outline bullets, "Approve / Request Changes" buttons.
   - **Grade Result**: Score visualization, gap list, "Study Gap" buttons.
   - **Transfer Problems**: Two problem cards with input fields and submit.
   - **Mastery Badge**: Animated badge with concept name and score.
2. Wire to SSE stream: parse events and update UI state accordingly.
3. Use `useLiveQuery` to read loop progress from PGlite (in case of refresh).

#### P2.5 Recursion & Escalation UI

1. **Gap Drill Dashboard**: Kanban-style board showing gaps per concept with status.
2. **Child Loop Trigger**: Clicking "Study Gap" starts a child loop at depth + 1.
3. **Horizontal Escalation**: After novice mastery, UI shows "Escalate to Peer Level" and "Escalate to Skeptic Level" buttons.
4. **Recursion Tree Visualization**: A tree diagram showing the concept hierarchy and loop status (pending / explaining / grading / mastered).

**P2 Exit Criteria**:
- Full Feynman loop runs end-to-end via API with mock LLM.
- All unit tests for loop phases pass (≥ 80% coverage).
- Frontend displays all loop phases correctly with SSE streaming.
- Recursion and escalation work correctly (depth guards, floor guards).
- Mastery closure only when all three criteria are met.

---

### Phase 3: Agent UI & Surfaces (Weeks 15-18)

**Goal**: The agent can dynamically render interactive UI surfaces. AG-UI streaming is polished. A2UI surfaces are rendered using shadcn/ui primitives.

#### P3.1 AG-UI Event Bus (Frontend)

1. Create `ui/src/lib/ag-ui.ts`:
   - `AgUiEventBus` class using `EventTarget` or zustand store.
   - Event types: `text`, `tool_call`, `tool_result`, `state_sync`, `lifecycle`.
   - `subscribe(eventType, callback)` and `publish(event)` methods.
2. Create `ui/src/components/AgUiStream.tsx`:
   - Connects to SSE endpoint.
   - Parses incoming AG-UI events and routes them to the event bus.
   - Handles reconnection with exponential backoff.
   - Displays connection status (connected / reconnecting / error).

#### P3.2 A2UI Catalog & Renderer

1. Define `A2UI Catalog` in `ui/src/surfaces/catalog.ts`:
   - Map A2UI component types to React components:
     - `Card` → `shadcn/ui Card`
     - `Column` → `div` with flex-col layout
     - `Text` → `p` or `span`
     - `TextField` → `shadcn/ui Input`
     - `ChoicePicker` → `shadcn/ui Select`
     - `Button` → `shadcn/ui Button`
     - `Markdown` → `react-markdown` with custom renderer
     - `CodeBlock` → `shadcn/ui` + `prism-react-renderer`
     - `Chart` → `recharts` or `chart.js` wrapper
2. Create `ui/src/surfaces/A2uiRenderer.tsx`:
   - Accepts a `surfaceId` and `components` array.
   - Recursively renders component tree using flat `id` references (not nested JSON).
   - Validates component types against catalog; unknown types render as fallback error card.
   - Handles `action` events: on button click, emits `action` message to the agent via WebSocket or POST.
3. Create `ui/src/surfaces/SurfaceManager.tsx`:
   - Maintains a registry of active surfaces (surfaceId → component tree + data model).
   - Handles `createSurface`, `updateComponents`, `updateDataModel`, `deleteSurface` messages.
   - Renders surfaces in designated layout regions (main chat, sidebar, modal overlay).

#### P3.3 Assistant-UI Integration

1. Install `@assistant-ui/react` and `@assistant-ui/react-markdown`.
2. Create a custom `AssistantUI` wrapper that integrates with the AG-UI event bus:
   - Maps `text` events to assistant-ui message streaming.
   - Maps `tool_call` / `tool_result` events to assistant-ui tool call cards.
   - Maps custom A2UI surfaces to `ThreadWelcome` or `Composer` components.
3. Customize the theme to match the shadcn/ui design system (dark mode default, warm tones, low saturation).

#### P3.4 Vercel AI SDK Integration

1. Install `ai` package (`npm install ai`).
2. Create a custom `useAgent` hook that wraps the Vercel AI SDK `useChat` or `useAssistant`:
   - Connects to the AG-UI SSE stream instead of a standard chat endpoint.
   - Handles tool call UI rendering via the AI SDK's `tool` renderer.
3. Integrate with the A2UI surface manager so that the AI SDK can request dynamic surfaces (e.g., a form for collecting structured input).

#### P3.5 Example Surfaces for Learning

Implement A2UI surfaces for the Feynman loop:
- `OutlinePreviewSurface`: Card with bullet list, approval buttons.
- `AnalogyCarouselSurface`: Horizontal scrollable cards with analogy text and complexity badge.
- `GapDrillSurface`: Tree view of gaps with study buttons.
- `TransferProblemSurface`: Problem card with input, timer, and submit button.
- `MasteryBadgeSurface`: Animated SVG badge with share button.
- `RetentionQuizSurface`: Single-question flashcard with flip animation.

**P3 Exit Criteria**:
- Agent can stream text, tool calls, and state updates via AG-UI.
- A2UI surfaces render correctly with all catalog component types.
- User interactions with A2UI surfaces correctly emit action events back to the agent.
- assistant-ui and Vercel AI SDK are integrated and functional.
- UI is responsive and accessible (WCAG 2.1 AA pass).

---

### Phase 4: MCP Client & Tools (Weeks 19-21)

**Goal**: The Axum server can connect to MCP servers, discover tools, and invoke them. The agent can use tools during a Feynman loop.

#### P4.1 MCP Client Crate (`pfla-mcp`)

1. Create `crates/pfla-mcp/`:
   - Define `McpClient` struct with connection configuration (stdio, HTTP, SSE).
   - Implement `initialize` handshake per MCP spec 2025-06.
   - Implement `tools/list` to discover available tools.
   - Implement `tools/call` to invoke a tool with JSON arguments.
   - Implement `resources/list` and `resources/read` for resource access.
   - Handle progress tokens and cancellation.
2. Use `tokio::process::Command` for stdio transport, `reqwest` for HTTP, `tokio_tungstenite` for SSE (over HTTP initially, upgrade to WebSocket if needed).

#### P4.2 MCP Server Configuration

1. Create `config/mcp-servers.json`:
   ```json
   {
     "servers": [
       {
         "id": "brave-search",
         "transport": "stdio",
         "command": "npx",
         "args": ["-y", "@modelcontextprotocol/server-brave-search"],
         "env": { "BRAVE_API_KEY": "${BRAVE_API_KEY}" }
       },
       {
         "id": "e2b-code",
         "transport": "http",
         "url": "https://e2b-mcp-server.example.com"
       }
     ]
   }
   ```
2. Load configuration at startup via `ArcSwap` for hot reload.
3. Validate configuration: check that all required env vars are set, commands are executable.

#### P4.3 Tool Invocation Flow

1. During the Feynman loop, the agent can request a tool call:
   - Agent emits AG-UI `tool_call` event with `tool_name`, `arguments`, `invocation_id`.
   - Frontend renders a `ToolCallCard` with a progress spinner.
   - Backend invokes the MCP server via `pfla-mcp`.
   - Backend emits AG-UI `tool_result` event with the result.
   - Frontend updates the card to show result or error.
2. Implement approval gating for destructive/expensive tools:
   - Configurable per-server: `require_approval: true/false`.
   - If required, the agent emits an A2UI `ApprovalSurface` with "Approve / Deny" buttons.
   - User approval is required before the backend actually invokes the tool.

#### P4.4 Tool Caching

1. Implement a short-term cache for tool results:
   - Cache key: `SHA256(server_id + tool_name + canonical_json(arguments))`.
   - TTL: 5 minutes for idempotent tools, 0 for non-idempotent.
   - Cache is stored in-memory (Redis optional for multi-instance deployments).
2. Invalidate cache on tool configuration change (ArcSwap watcher).

**P4 Exit Criteria**:
- MCP client connects to at least 2 configured servers (e.g., Brave Search, E2B Code).
- Tool calls are surfaced in the UI with progress and results.
- Approval gating works for configured tools.
- Tool caching reduces redundant calls.
- Error handling: failed tool calls show user-friendly error messages.

---

### Phase 5: LLM Integration & Grading (Weeks 22-24)

**Goal**: Replace the mock LLM with real providers. Production-grade grading with sycophancy correction.

#### P5.1 LLM Provider Abstraction

1. Create `crates/pfla-feynman/src/llm/`:
   - `LlmProvider` trait:
     ```rust
     pub trait LlmProvider: Send + Sync {
         async fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String, LlmError>;
         async fn stream(&self, prompt: &str, config: &GenerationConfig) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>;
     }
     ```
   - Implementations: `OpenAiProvider`, `AnthropicProvider`, `LocalLlamaProvider` (via `ollama` or `llama.cpp` HTTP API).
2. Configuration in `config/llm.yaml`:
   ```yaml
   default: openai
   providers:
     openai:
       api_key: "${OPENAI_API_KEY}"
       model: "gpt-4o"
       base_url: "https://api.openai.com/v1"
     anthropic:
       api_key: "${ANTHROPIC_API_KEY}"
       model: "claude-sonnet-4"
     local:
       base_url: "http://localhost:11434"
       model: "llama3.1:70b"
   ```

#### P5.2 Prompt Engineering for Feynman Loop

1. **Explanation Generation Prompt**:
   - System prompt: "You are a patient, brilliant tutor. Explain the concept to the specified audience level. Use analogies. No preamble."
   - Few-shot examples for each audience level (novice, peer, skeptic).
   - Dynamic prompt injection: include concept description, prerequisites, and learner's past artifacts for context.
2. **Grading Prompt**:
   - System prompt: "You are a rigorous, fair grader. Evaluate the explanation for completeness, accuracy, clarity, and depth. Identify specific misconceptions and gaps. Be honest — do not be overly positive."
   - Include the rubric in the prompt.
   - Output structured JSON matching the `Grade` schema.
3. **Transfer Problem Generation Prompt**:
   - System prompt: "Generate two novel problems that test the learner's ability to apply the concept in new contexts. Problems must not be solvable by memorization."
   - Output structured JSON with problem statement, expected answer, and scoring rubric.

#### P5.3 Sycophancy Correction

1. Implement a two-pass grading strategy:
   - Pass 1: Grade the explanation normally.
   - Pass 2: A separate "skeptic" prompt asks: "Is this grader being too generous? Are there real gaps that were missed?"
   - If Pass 2 identifies issues that Pass 1 missed, lower the overall score and add the gaps.
2. Alternative: Use a smaller, more critical model (e.g., `gpt-4o-mini` with strict instructions) as a second opinion.
3. Log all grading discrepancies for analysis by the Karpathy Loop.

#### P5.4 Streaming Explanation

1. Wire the LLM stream into the AG-UI SSE stream:
   - Each LLM token becomes an `ag_ui_event` with `kind: "text"`.
   - Frontend renders tokens as they arrive (character-by-character feel).
2. Handle buffering: if the LLM provider sends chunks, buffer to word boundaries for smoother rendering.
3. Handle backpressure: if the client is slow, drop non-critical UI updates (but never drop text tokens).

**P5 Exit Criteria**:
- Real LLM generates explanations and grades with structured JSON output.
- Sycophancy correction prevents false positives in grading.
- Streaming explanations feel responsive (< 100ms per token chunk).
- All LLM interactions are logged (prompt, response, latency, cost) for analytics.
- Fallback to local LLM if cloud provider is unavailable (configurable).

---

### Phase 6: Flint-Forge Integration (Weeks 25-27)

**Goal**: The app connects to the Flint-Forge backend for multi-tenant cloud features, real-time events, and identity.

#### P6.1 Flint-Forge Crate Reuse

1. Add Flint-Forge crates as path dependencies or git dependencies:
   - `fdb-auth` for JWT validation and GoTrue integration.
   - `fdb-postgres` for connection pooling and schema management.
   - `fdb-realtime` for LISTEN/NOTIFY event bus.
   - `fdb-reflection` for Quarry engine integration.
   - `fdb-gateway` for GraphQL hybrid routing.
2. Create adapter crates if needed (`pfla-fdb-auth-adapter`, etc.) to map Flint-Forge types to PFLA domain types.

#### P6.2 Authentication & Identity

1. Integrate `fdb-auth` into the Axum server:
   - `POST /api/v1/auth/signup` and `/signin` → delegate to GoTrue.
   - JWT validation middleware for all protected routes.
   - RLS context injection: set `app.current_user_id()` on Postgres connections.
2. Frontend integration:
   - Signup/signin screens with email/password and social login buttons.
   - JWT storage: `localStorage` for web, Keychain/Keyring for Tauri (via Tauri plugin).
   - Token refresh: automatic on 401 responses.

#### P6.3 Real-Time Event Bus

1. Use `fdb-realtime` to listen for database changes:
   - `LISTEN schema_changes` for DDL events (e.g., new curriculum versions).
   - `LISTEN goal_updates` for goal status changes from other devices.
   - Bridge to SSE: when a notification arrives, fetch full event from outbox and push to connected clients.
2. Frontend: subscribe to SSE channels per-learner. Update local PGlite state when events arrive.

#### P6.4 GraphQL Hybrid API

1. Expose `fdb-gateway` GraphQL endpoint at `/api/v2/graphql`:
   - Queries for complex relationships (e.g., "all goals with their concepts and latest grades").
   - Mutations for batch operations (e.g., "archive all completed goals").
   - Subscriptions for real-time updates (via WebSocket, backed by Postgres LISTEN/NOTIFY).
2. REST endpoints remain for simple CRUD and AG-UI streaming.

**P6 Exit Criteria**:
- User can sign up, sign in, and stay authenticated across sessions.
- RLS policies enforce data isolation per learner.
- Real-time updates from other devices are reflected in the UI within 2 seconds.
- GraphQL endpoint supports complex queries and subscriptions.
- Flint-Forge crates are integrated and tested.

---

### Phase 7: Tauri Desktop & Mobile (Weeks 28-31)

**Goal**: Native desktop and mobile applications built from the same codebase.

#### P7.1 Tauri Project Scaffold

1. In `tauri/` directory, run `npm create tauri-app@latest` with:
   - Frontend: React + TypeScript + Vite (reuse `ui/`).
   - Backend: Rust (reuse `crates/`).
2. Configure `tauri.conf.json`:
   - Window title, dimensions, resizable.
   - Updater endpoint: GitHub Releases JSON.
   - Deep link scheme: `pfla://` for share links.

#### P7.2 Embedded Axum Server

1. In `tauri/src-tauri/src/lib.rs`, create a background thread that starts the Axum server:
   - Bind to `127.0.0.1:0` (random free port).
   - Store the actual port in Tauri managed state.
   - Frontend discovers the port via Tauri command `get_api_port()`.
   - Frontend uses `http://127.0.0.1:{port}` for all API calls.
2. Use `include_dir` in `build.rs` to embed the Vite production build into the binary.
   - Serve static files from the embedded directory via Axum's `ServeDir`.
   - Alternatively, use Tauri's built-in asset protocol for serving the frontend.

#### P7.3 Tauri Commands

1. Implement Tauri commands for native capabilities:
   - `greet` (hello world → verify IPC works).
   - `get_api_port` → returns the loopback port of the embedded Axum server.
   - `get_app_data_dir` → returns the Tauri app data directory for PGlite filesystem persistence.
   - `show_notification` → native system notification for retention reminders.
   - `set_tray_menu` → system tray with quick actions (Start Loop, Sync Status, Quit).
2. Wire frontend to use Tauri commands when `window.__TAURI__` is present, fall back to HTTP otherwise.

#### P7.4 Mobile-Specific Adaptations

1. iOS:
   - Add `SafeArea` component to all screens.
   - Configure `Info.plist` for background sync (if needed).
   - Test on-device: iPhone SE (small screen), iPhone 15 Pro (notch), iPad (split view).
2. Android:
   - Add `AndroidManifest.xml` permissions for internet and notifications.
   - Handle back button navigation (pop router history).
   - Test on-device: Pixel 8, Samsung Galaxy S24.
3. Touch-optimized gestures:
   - Swipe right on a concept card to start a loop.
   - Pull-to-sync on the goal list.
   - Pinch-to-zoom on A2UI diagrams.

#### P7.5 Code Signing & Distribution

1. **macOS**:
   - Apple Developer ID certificate for signing.
   - Notarization via `notarytool` in CI.
   - DMG and .app.zip distribution.
2. **Windows**:
   - EV Code Signing certificate (or standard cert if EV is unavailable).
   - NSIS and MSI installers.
   - SmartScreen reputation building (submit to Microsoft for review).
3. **Linux**:
   - AppImage, DEB, and RPM packages.
   - No signing required, but GPG signing for DEB/RPM is recommended.
4. **iOS/Android**:
   - Apple App Store and Google Play Store developer accounts.
   - TestFlight / Play Console Internal Testing for beta distribution.

#### P7.6 Auto-Updater

1. Configure Tauri updater in `tauri.conf.json`:
   - `endpoints`: `https://api.prometheus-ags.com/updater/{{target}}/{{current_version}}`.
   - `pubkey`: Ed25519 public key for signature verification.
2. CI workflow generates update manifest and signs the update bundle.
3. Frontend checks for updates on startup and shows a "Update Available" banner.

**P7 Exit Criteria**:
- Tauri app builds for Windows, macOS, and Linux without errors.
- Desktop app runs the embedded Axum server and serves the React frontend.
- Native notifications work for retention reminders.
- Auto-updater checks for and installs updates successfully.
- Mobile apps (iOS/Android) build and run on-device (basic functionality, not full feature parity).

---

### Phase 8: Karpathy Loop & Analytics (Weeks 32-34)

**Goal**: The system autonomously improves pedagogy based on learner outcomes. Admins can view experiment results.

#### P8.1 LVS Computation

1. In `fdb-reflection`, add a scheduled job (or cron) that computes LVS per cohort:
   ```sql
   -- Quarry reflection query
   SELECT
     cohort_id,
     AVG(CASE WHEN status = 'mastered' THEN 1.0 ELSE 0.0 END) AS mastery_rate,
     AVG(depth) AS avg_recursion_depth,
     AVG(CASE WHEN retention_passed THEN 1.0 ELSE 0.0 END) AS retention_pass_rate,
     (mastery_rate * 0.4 + (1.0 / avg_recursion_depth) * 0.3 + retention_pass_rate * 0.3) AS lvs
   FROM learner_cohorts
   GROUP BY cohort_id;
   ```
2. Store LVS history in a time-series table (`lvs_snapshots`).
3. Frontend: Admin dashboard with LVS trend charts (per cohort, per goal, per concept).

#### P8.2 Experiment Framework

1. Define experiment schema:
   ```sql
   CREATE TABLE experiments (
     id UUID PRIMARY KEY,
     hypothesis TEXT NOT NULL,
     parameter TEXT NOT NULL, -- e.g., 'analogy_type', 'explanation_length'
     control_value TEXT NOT NULL,
     experiment_value TEXT NOT NULL,
     cohort_id UUID NOT NULL,
     start_time TIMESTAMPTZ NOT NULL,
     end_time TIMESTAMPTZ,
     control_lvs FLOAT,
     experiment_lvs FLOAT,
     p_value FLOAT,
     status TEXT DEFAULT 'pending', -- 'running', 'completed', 'committed', 'rejected'
     committed_at TIMESTAMPTZ
   );
   ```
2. Create the Improvement Agent:
   - A background service that queries Quarry for underperforming cohorts.
   - Generates experiment proposals via LLM (e.g., "Cohort X has low mastery rate. Try analogy type 'visual' instead of 'verbal'.").
   - Submits proposals to the admin dashboard for approval.
3. Implement experiment runner:
   - Randomly assigns learners to control or experiment group (stratified by proficiency).
   - Applies the experimental parameter to the experiment group.
   - Runs for 24 hours.
   - Computes LVS for both groups at the end.
   - Statistical test (t-test or Mann-Whitney U) for significance.

#### P8.3 Admin Dashboard

1. Create an admin-only route (`/admin/experiments`):
   - List of experiments with status, hypothesis, results.
   - "Approve" / "Reject" buttons for pending proposals.
   - Diff view for proposed curriculum changes.
2. Admin-only A2UI surfaces:
   - `ExperimentCardSurface`: Shows hypothesis, parameter, expected LVS delta, risk badge.
   - `LvsChartSurface`: Line chart of LVS over time for control vs experiment.
   - `CurriculumDiffSurface`: Side-by-side diff of proposed curriculum changes.

#### P8.4 Auto-Commit & Safety

1. Winning experiments (p < 0.05, LVS improvement > 5%) are auto-committed:
   - If < 20% of curriculum affected: auto-commit without human approval.
   - If ≥ 20% affected: require admin approval.
2. All commits are logged in `experiment_log` with git-style history.
3. Rollback capability: any committed experiment can be reverted within 7 days.

**P8 Exit Criteria**:
- LVS is computed and stored for all cohorts.
- Improvement Agent generates at least one experiment proposal per week.
- Admin dashboard shows experiments with approval/rejection workflow.
- Auto-commit works for small changes; large changes require human approval.
- Experiment history is immutable and auditable.

---

### Phase 9: Subscription & Business Model (Weeks 35-37)

**Goal**: The platform is monetized via a freemium-to-subscription model.

#### P9.1 Stripe Integration

1. Backend:
   - `POST /api/v1/billing/checkout` → creates Stripe Checkout session for Plus/Pro.
   - `POST /api/v1/billing/portal` → redirects to Stripe Customer Portal.
   - Webhook `POST /api/v1/billing/webhook` → handles `invoice.paid`, `invoice.payment_failed`, `customer.subscription.updated`.
   - On webhook, update `learners.subscription_tier` and `subscription_status`.
2. Frontend:
   - Pricing page with three tiers (Free, Plus, Pro).
   - Checkout button redirects to Stripe Checkout.
   - Billing settings page with "Manage Subscription" link to Stripe Portal.

#### P9.2 Feature Gating

1. Backend gating:
   - Middleware `require_subscription(tier)` for API routes.
   - Returns `403` with `code: "SUBSCRIPTION_REQUIRED"` if tier is insufficient.
   - Examples:
     - `POST /api/v1/goals` → gated: max 3 active for free.
     - `POST /api/v1/learn/feynman-loop` with `audience: peer` → requires Plus.
     - `POST /api/v1/learn/feynman-loop` with `audience: skeptic` → requires Pro.
     - `GET /api/v1/admin/experiments` → requires Pro + admin role.
2. Frontend gating:
   - Components show "Upgrade to Plus" badge when feature is unavailable.
   - Buttons are disabled with tooltip explaining the required tier.
   - Goal creation counter: "2/3 free goals used".

#### P9.3 Subscription Tiers

| Tier | Price | Features |
|---|---|---|
| **Free** | $0 | 3 active goals, novice audience, basic analogies, no retention scheduling, web only. |
| **Plus** | $9.99/mo | Unlimited goals, all audiences (novice/peer/skeptic), retention scheduling, artifact library, Tauri desktop app. |
| **Pro** | $19.99/mo | Plus + Karpathy Loop insights, custom MCP tools, priority LLM queue, 1:1 tutor sessions (future), mobile app. |

#### P9.4 Grace Period & Downgrades

1. On payment failure:
   - Retry 3 times over 7 days.
   - During grace period, user retains Plus/Pro features.
   - After grace period, tier downgrades to Free.
2. On downgrade:
   - Existing goals are preserved but read-only if over the free limit.
   - User can delete goals to get back under the limit, then edit again.
3. On upgrade:
   - Immediate access to new features (no waiting for next billing cycle).

**P9 Exit Criteria**:
- Stripe Checkout and Portal work end-to-end (test mode).
- Webhook correctly updates subscription tier in real time.
- Feature gating is enforced on both frontend and backend.
- Grace period and downgrade behavior work correctly.
- Pricing page is accessible and persuasive.

---

### Phase 10: Polish, Performance, Launch (Weeks 38-41)

**Goal**: The product is ready for public beta launch.

#### P10.1 Performance Optimization

1. **Frontend**:
   - Code splitting by route (TanStack Router lazy loading).
   - PGlite query optimization: add indexes, limit query complexity.
   - Bundle size audit: tree-shake unused shadcn/ui components, lazy-load heavy libraries (recharts, react-markdown).
   - Target: < 200KB initial JS, < 1s time-to-interactive on 4G.
2. **Backend**:
   - SSE connection pooling: limit concurrent connections per learner.
   - LLM request caching: cache identical prompts for 5 minutes.
   - Database query optimization: add composite indexes, use `EXPLAIN ANALYZE`.
   - Target: < 50ms p99 API latency, < 500ms p99 LLM round-trip.
3. **Sync**:
   - ElectricSQL shape optimization: filter aggressively, limit columns.
   - Delta sync: only sync changed rows, not full tables.

#### P10.2 Security Audit

1. Penetration testing:
   - A2UI injection: attempt to send unauthorized component types, validate rejection.
   - MCP tool abuse: attempt to invoke tools without approval, validate gating.
   - Auth bypass: attempt to access other learners' data, validate RLS.
   - SQL injection: test all user inputs with sqlmap or similar.
2. Dependency audit:
   - `cargo audit` for Rust CVEs.
   - `pnpm audit` for JS CVEs.
   - `cargo vet` for supply chain trust.
3. Secrets management:
   - All secrets (API keys, JWT signing keys) in environment variables or Vault.
   - No secrets in source code or Docker images.

#### P10.3 Accessibility Pass

1. WCAG 2.1 AA audit using automated tools (axe-core, Lighthouse).
2. Keyboard navigation: all interactive elements are reachable via Tab.
3. Screen reader testing: AG-UI text events are announced, A2UI components have ARIA labels.
4. Color contrast: all themes pass 4.5:1 ratio.
5. Reduced motion: animations respect `prefers-reduced-motion`.

#### P10.4 Documentation

1. User documentation:
   - "Getting Started" guide (create first goal, run first loop).
   - "Understanding the Feynman Loop" explainer.
   - "Offline Mode" guide.
   - "Subscription FAQ".
2. API documentation:
   - OpenAPI spec for all REST endpoints.
   - GraphQL schema documentation.
   - AG-UI event schema documentation.
   - A2UI catalog documentation.
3. Developer documentation:
   - Architecture overview (this doc).
   - Contribution guide.
   - Local development setup guide.

#### P10.5 Marketing & Launch

1. Marketing site:
   - Landing page with hero video, feature grid, testimonials, pricing, CTA.
   - Built with the same React + Vite stack, deployed to Vercel/Cloudflare Pages.
2. Launch channels:
   - Product Hunt launch.
   - Hacker News "Show HN" post.
   - Reddit r/selfimprovement, r/learnprogramming, r/artificial.
   - X/Twitter thread by founder.
   - Newsletter to early access list.
3. Beta program:
   - Invite-only for first 1000 users.
   - Feedback form in-app (A2UI surface).
   - Discord community for power users.

**P10 Exit Criteria**:
- Performance targets met (frontend and backend).
- No critical or high-severity security issues.
- WCAG 2.1 AA compliance verified.
- Documentation is complete and accurate.
- Marketing site is live and functional.
- Public beta is launched with at least 1000 waitlist signups.

---

## 4. Dependency Graph

```
P0 (Foundation)
  │
  ├─► P1 (Local-First Data)
  │     │
  │     ├─► P2 (Feynman Loop) ─────┐
  │     │                            │
  │     └─► P6 (Flint-Forge) ──────┤
  │                                  │
  P3 (Agent UI) ◄──────────────────┤
  │     │                            │
  │     └─► P4 (MCP Tools) ────────┤
  │                                  │
  P5 (LLM Integration) ◄───────────┘
  │     │
  │     └─► P8 (Karpathy Loop) ────┐
  │                                │
  P7 (Tauri) ◄─────────────────────┤
  │                                │
  P9 (Subscription) ◄────────────┘
  │
  P10 (Launch)
```

**Critical Path**: P0 → P1 → P2 → P5 → P10. The shortest path to a functional MVP is 4 + 4 + 6 + 3 + 4 = 21 weeks (≈ 5 months).

---

## 5. Team Composition & Responsibilities

| Role | Count | Primary Responsibilities | Phases |
|---|---|---|---|
| **Rust Backend Engineer** | 2 | Axum server, Feynman loop engine, MCP client, Flint-Forge integration, Tauri Rust core. | P0-P2, P4-P6, P7, P8, P10 |
| **Frontend Engineer** | 2 | React 19 SPA, A2UI surfaces, AG-UI streaming, PGlite/ElectricSQL integration, Tauri webview. | P0-P3, P7, P9, P10 |
| **DevOps / Platform** | 1 | CI/CD, Kubernetes/ECS deployment, Postgres ops, ElectricSQL sync service, monitoring. | P0, P1, P6, P10 |
| **AI / Prompt Engineer** | 1 (part-time) | LLM prompt engineering, grading rubrics, sycophancy correction, Karpathy Loop experiments. | P2, P5, P8 |
| **Designer** | 1 (part-time) | UI/UX design, A2UI component design, marketing site, brand identity. | P3, P10 |

**Solo/Small Team Adaptation** (1-2 people):
- Focus on the critical path: P0 → P1 → P2 → P5 → P10 (web-only, no Tauri, no Karpathy Loop initially).
- Use managed services: Supabase for Postgres/Auth, Electric Cloud for sync, Vercel for hosting.
- Defer MCP client to Phase 4+ (use simple HTTP APIs for tools initially).
- Launch with Free + Plus tiers only; add Pro later.

---

## 6. Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ElectricSQL sync unreliability for large datasets | Medium | High | Implement manual sync fallback; shard shapes by goal; test with 10K+ artifacts. |
| Tauri 2 mobile instability | Medium | High | Ship desktop first; use Flutter as mobile fallback if Tauri proves unstable. |
| LLM API costs spiral | Medium | High | Implement aggressive caching; use local LLM for free tier; rate limit per user. |
| Sycophancy correction insufficient | Medium | High | Two-pass grading + human review sampling; iterate on prompts via Karpathy Loop. |
| A2UI adoption stalls | Low | Medium | Build custom renderer regardless; A2UI is a value-add, not a hard dependency. |
| Flint-Forge integration complexity | Medium | Medium | Start with simple REST API to Postgres; add Flint-Forge crates incrementally. |
| Subscription conversion low | Medium | High | A/B test pricing; offer annual discount; strong free tier to build habit. |
| MCP server ecosystem fragmentation | Low | Medium | Support only well-maintained servers; build own servers for critical tools. |

---

*End of Implementation Plan*
