# Prometheus Feynman Learning Agent — System Architecture

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |

---

## 1. Executive Summary

The **Prometheus Feynman Learning Agent (PFLA)** is a cross-platform, AI-native learning application that enables users to master any subject using the Feynman Technique — explain, identify gaps, re-study, and re-explain — orchestrated by an intelligent agent layer. The system is built as a **Rust Axum** web application with an embedded **React 19 / Vite 7** frontend, compiled via `build.rs` and served as static assets. It exposes an **AG-UI** endpoint with **A2UI** support for AI-generated dynamic interfaces, acts as an **MCP client** to leverage external tool ecosystems, and integrates the **Flint-Forge** local-first data architecture (Postgres backend + PGlite frontend + ElectricSQL sync). A **Tauri 2** wrapper provides native desktop and mobile applications from the same codebase.

The platform is architected as a **three-sided learning marketplace**: students learn via the Feynman loop, experts create **digital coaching personas** (grounded in their corpus, optionally fine-tuned with LoRA) through the **Creator Studio**, and certified students become **masters** who earn money coaching others via live text, async video, and WebRTC video conferencing. The marketplace is built on `flint-realtime-fabric` for real-time video infrastructure, with a **Master Certification** pipeline that transforms students into earning coaches through the Karpathy Loop.

The architecture is designed around three core principles:
1. **Local-First Data**: PGlite in-browser + ElectricSQL sync ensures the user's learning data is always available, even offline.
2. **Agent-Native UI**: AG-UI/A2UI protocols enable the AI agent to dynamically generate interactive learning surfaces rather than static chat.
3. **Continuous Improvement**: The Karpathy Loop-inspired feedback mechanism enables the learning agent to iteratively improve its pedagogy based on learner outcomes, and the marketplace adds a master certification pipeline that turns students into teachers.

**Related Documents**: This architecture document is the foundation for the marketplace addenda covering the **Coach Catalog**, **Creator Studio**, **Master Certification**, and **Video Conferencing** infrastructure (`prometheus-feynman-learning-agent-architecture-addendum.md`).

---

## 2. System Context

### 2.1 Stakeholders

| Stakeholder | Concern |
|---|---|
| **Learners** | Intuitive, always-available learning interface that adapts to their knowledge gaps. |
| **AI Agent** | Real-time access to learner state, ability to render dynamic UI surfaces, tool access via MCP. |
| **Platform Operators** | Observable, scalable, multi-tenant backend with clear data isolation. |
| **Developers** | Type-safe, componentized architecture with clear separation between frontend, backend, and agent logic. |

### 2.2 External Systems

| System | Role | Integration |
|---|---|---|
| **Flint-Forge Postgres** | Primary backend database, real-time event bus, auth, reflection engine. | Direct SQL via `sqlx`, LISTEN/NOTIFY for real-time events. |
| **PGlite (WASM)** | Client-side embedded database for offline-first learning state. | `@electric-sql/pglite` + `drizzle-orm/pglite` in browser. |
| **ElectricSQL Sync** | Bidirectional sync between cloud Postgres and local PGlite. | `pglite-sync` shape subscriptions. |
| **MCP Servers** | External tool access (search, calculation, code execution, knowledge retrieval). | Rust MCP client over stdio/HTTP/SSE. |
| **LLM Providers** | Feynman loop reasoning, explanation generation, grading. | OpenAI, Anthropic, local models via `flint-realtime-fabric`. |
| **Vercel AI SDK** | Frontend streaming, tool call UI, state management for agent interactions. | `@assistant-ui/react` + `ai` package. |

---

## 3. Architectural Drivers

### 3.1 Functional Requirements

| ID | Requirement | Priority |
|---|---|---|
| FR-01 | Learners can initiate a Feynman loop on any concept. | Must |
| FR-02 | The system generates plain-language explanations with analogies at novice, peer, and skeptic levels. | Must |
| FR-03 | The system grades explanations, identifies gaps, and recursively targets weak concepts. | Must |
| FR-04 | The agent can dynamically render UI surfaces (forms, cards, diagrams) via A2UI/AG-UI. | Must |
| FR-05 | Learning state persists offline via PGlite and syncs when online. | Must |
| FR-06 | The agent can invoke external tools via MCP (search, compute, code execution). | Must |
| FR-07 | Users can access the app via web, desktop (Windows/macOS/Linux), and mobile (iOS/Android). | Must |
| FR-08 | The system supports continuous improvement via Karpathy Loop-style experiment evaluation. | Should |
| FR-09 | The system supports a freemium-to-subscription business model with feature gating. | Should |

### 3.2 Quality Attributes

| Attribute | Target | Approach |
|---|---|---|
| **Performance** | < 200ms UI response, < 500ms agent round-trip | ArcSwap hot-state, PGlite local queries, edge-cached assets. |
| **Availability** | 99.9% (online), 100% (offline core) | Local-first PGlite + background sync. |
| **Scalability** | 10K concurrent learners | Horizontal scaling of stateless Axum services; Postgres read replicas. |
| **Security** | Zero trust, RLS-enforced, MCP sandboxed | Capability-based Tauri permissions, Postgres RLS, A2UI component allowlists. |
| **Maintainability** | Modular crates, skill-driven | Clean architecture per crate; PMPO skill lifecycle. |
| **Portability** | Web + Desktop + Mobile | Single React frontend, Tauri 2 wrapper, responsive design. |

### 3.3 Constraints

| Constraint | Implication |
|---|---|
| Rust 1.85+ (2021 edition) | Modern async, `async fn` in traits, `let else` patterns. |
| React 19 + Vite 7 + shadcn/ui (Base UI) | No Next.js; SPA with client-side routing (TanStack Router). |
| Tauri 2 mobile targets | iOS 9+, Android 8+ minimum; shared Rust core. |
| PGlite single-user/connection | Design around single-connection WASM Postgres; no concurrent writers. |
| MCP spec 2025-06 | Follow Anthropic MCP standard for tool discovery, invocation, and capability negotiation. |

---

## 4. System Decomposition

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                         │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────────────────┐  │
│  │   Web Browser       │  │   Tauri Desktop     │  │   Tauri Mobile (iOS/   │  │
│  │   (React 19 + Vite) │  │   (React + Rust)    │  │   Android)              │  │
│  │   ┌───────────────┐ │  │   ┌───────────────┐ │  │   ┌──────────────────┐ │  │
│  │   │  PGlite WASM  │ │  │   │  PGlite WASM  │ │  │   │  PGlite WASM     │ │  │
│  │   │  (IndexedDB)  │ │  │   │  (AppData FS) │ │  │   │  (AppData FS)    │ │  │
│  │   └───────────────┘ │  │   └───────────────┘ │  │   └──────────────────┘ │  │
│  │   ┌───────────────┐ │  │   ┌───────────────┐ │  │   ┌──────────────────┐ │  │
│  │   │ ElectricSQL   │ │  │   │ ElectricSQL   │ │  │   │ ElectricSQL      │ │  │
│  │   │ Sync Client   │ │  │   │ Sync Client   │ │  │   │ Sync Client      │ │  │
│  │   └───────────────┘ │  │   └───────────────┘ │  │   └──────────────────┘ │  │
│  └─────────────────────┘  └─────────────────────┘  └──────────────────────────┘  │
│           ▲                        ▲                        ▲                    │
│           │  HTTP/SSE/WS           │  Tauri IPC             │  Tauri IPC         │
│           └────────────────────────┴────────────────────────┘                    │
│                                       │                                          │
│                              ┌────────┴────────┐                                 │
│                              │   AG-UI Bridge  │  (A2UI surfaces, event stream) │
│                              │   (React/TS)    │                                 │
│                              └─────────────────┘                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
                                       │
┌──────────────────────────────────────┼──────────────────────────────────────────┐
│                              EDGE / API LAYER                                   │
│                              ┌─────────────────────────────────────────────┐    │
│                              │   Rust Axum Server (embedded static + API)  │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  Static Asset Router (build.rs)     │   │    │
│                              │   │  Serves Vite-built SPA from binary  │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  AG-UI SSE Endpoint                 │   │    │
│                              │   │  /api/v1/ag-ui/stream               │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  A2UI Surface Endpoint              │   │    │
│                              │   │  /api/v1/a2ui/surface             │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  MCP Client Bridge                  │   │    │
│                              │   │  /api/v1/mcp/invoke               │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  Feynman Loop API                   │   │    │
│                              │   │  /api/v1/learn/{loop,grade,plan}    │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  Auth / Identity (Flint-Forge)      │   │    │
│                              │   │  JWT validation, RLS context        │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              └─────────────────────────────────────────────┘    │
│                                       │                                         │
│                              ┌────────┴────────┐                                │
│                              │  Realtime Fabric  │                               │
│                              │  (flint-realtime) │                               │
│                              └─────────────────┘                               │
└──────────────────────────────────────┼──────────────────────────────────────────┘
                                       │
┌──────────────────────────────────────┼──────────────────────────────────────────┐
│                              DATA / AGENT LAYER                                 │
│                              ┌─────────────────────────────────────────────┐    │
│                              │   Flint-Forge Postgres (cloud)              │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  fdb-app: Application Domain        │   │    │
│                              │   │  fdb-auth: Identity & Auth          │   │    │
│                              │   │  fdb-postgres: Schema/DDL           │   │    │
│                              │   │  fdb-realtime: LISTEN/NOTIFY bus    │   │    │
│                              │   │  fdb-reflection: Quarry Engine      │   │    │
│                              │   │  fdb-gateway: API Gateway           │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              └─────────────────────────────────────────────┘    │
│                              ┌─────────────────────────────────────────────┐    │
│                              │   Agent Runtime (Rust/Axum)                   │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  Feynman Loop Engine                │   │    │
│                              │   │  ├─ Spec/Plan/Execute/Reflect     │   │    │
│                              │   │  ├─ Recursion & Escalation          │   │    │
│                              │   │  └─ Mastery Closure Criteria        │   │    │
│                              │   │  Karpathy Loop Adapter              │   │    │
│                              │   │  ├─ Editable Asset (curriculum)     │   │    │
│                              │   │  ├─ Scalar Metric (learning rate)   │   │    │
│                              │   │  └─ Time-boxed Experiment           │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              └─────────────────────────────────────────────┘    │
│                              ┌─────────────────────────────────────────────┐    │
│                              │   MCP Server Ecosystem                      │    │
│                              │   ┌─────────────────────────────────────┐   │    │
│                              │   │  Web Search (Brave/Perplexity)        │   │    │
│                              │   │  Code Execution (e2b)               │   │    │
│                              │   │  Knowledge Retrieval (supabase)       │   │    │
│                              │   │  File System (local)                │   │    │
│                              │   └─────────────────────────────────────┘   │    │
│                              └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Layer Descriptions

#### 4.2.1 Client Layer (React 19 + Vite 7 + shadcn/ui)

The frontend is a single-page application (SPA) built with **React 19**, **Vite 7**, **Tailwind CSS v4**, and **shadcn/ui** (based on Base UI primitives). It uses **TanStack Router** for client-side routing and **@prometheus-ags/prometheus-entity-management** for normalized, globally-reactive entity graph state management (replacing per-view cache models like TanStack Query).

Key components:
- **AG-UI Renderer**: Subscribes to the SSE event stream and renders agent-generated UI surfaces (text, tool calls, forms, charts) using `assistant-ui` primitives.
- **A2UI Surface Renderer**: Parses A2UI JSON messages (`createSurface`, `updateComponents`, `updateDataModel`) and maps them to native shadcn/ui components (Card, Button, TextField, ChoicePicker, Markdown).
- **PGlite Data Layer**: Uses `@electric-sql/pglite` with `live` and `electricSync` extensions. Local schema mirrors the cloud Postgres schema via ElectricSQL shapes.
- **Feynman Loop UI**: Dedicated screens for the Explain → Gap-Find → Re-Study cycle, with explanation input, grading visualization, and gap concept drill-down.
- **Tauri IPC Bridge**: In desktop/mobile builds, replaces HTTP calls with Tauri `invoke()` for file system access, system notifications, and native menus.

#### 4.2.2 Edge / API Layer (Rust Axum)

The Axum server is a dual-purpose binary: it serves the embedded React SPA and hosts the API endpoints. The SPA is compiled via `build.rs` (Vite production build) and embedded as static bytes using `rust-embed` or `include_dir`.

Key endpoints:
- `GET /` → Serves `index.html` from embedded assets.
- `GET /assets/*` → Serves hashed JS/CSS chunks with far-future caching headers.
- `GET /api/v1/ag-ui/stream` → SSE endpoint for AG-UI event streaming (text, tool calls, state sync, lifecycle).
- `POST /api/v1/a2ui/surface` → Accepts A2UI JSON messages from the agent runtime and broadcasts to connected clients.
- `POST /api/v1/mcp/invoke` → Proxies MCP tool calls from the frontend to configured MCP servers.
- `POST /api/v1/learn/feynman-loop` → Initiates or continues a Feynman loop.
- `POST /api/v1/learn/grade` → Submits an explanation for grading.
- `POST /api/v1/learn/retain` → Schedules or evaluates a retention check.

The server uses **ArcSwap** for hot-swappable configuration and router state, and **tokio::sync::watch** for broadcasting metadata changes to async tasks. The `fdb-gateway` crate from Flint-Forge provides GraphQL hybrid query routing.

#### 4.2.3 Data / Agent Layer (Flint-Forge + Postgres + ElectricSQL)

The backend is anchored by **Flint-Forge**, a Rust workspace providing:
- **fdb-domain**: Core domain types (Learner, Concept, Goal, Artifact, Grade).
- **fdb-postgres**: Postgres connection management, schema migration, and `pgvector` integration for semantic search over learning corpora.
- **fdb-realtime**: LISTEN/NOTIFY event bus with outbox pattern for reliable schema change propagation.
- **fdb-auth**: JWT-based identity and RLS policy enforcement.
- **fdb-reflection**: Quarry reflection engine — analyzes learner trajectories, identifies pedagogical patterns, and feeds the Karpathy Loop.
- **fdb-gateway**: API gateway with GraphQL hybrid routing (REST for simple CRUD, GraphQL for complex queries).
- **fke-runtime**: Container runtime for executing sandboxed learning experiments (e.g., code execution for CS concepts).

**ElectricSQL** provides the sync layer between cloud Postgres and local PGlite:
- Shapes are defined per-learner: `goals`, `concepts`, `artifacts`, `grades`.
- Sync is bidirectional: learner writes (e.g., new explanation) are queued locally and pushed when online; schema changes from the agent are pulled and applied locally.
- Conflict resolution uses last-write-wins (LWW) with vector clocks for critical learner state.

---

## 5. Key Component Deep-Dives

### 5.1 Feynman Loop Engine

The Feynman Loop Engine is a PMPO (Plan-Model-Plan-Operate) cycle implementation that maps the four Feynman steps onto agentic phases:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Spec      │ -> │   Plan      │ -> │   Execute   │ -> │   Reflect   │
│  (Pick)     │    │ (Structure) │    │  (Explain)  │    │  (Grade)    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       ^                                                        │
       └──────────────────── Recurse ──────────────────────────┘
```

**Spec Phase**: Load the concept state from the learner model (PGlite or Postgres). Determine target audience (`novice`, `peer`, `skeptic`). Check recursion floor guards.

**Plan Phase**: The agent generates an explanation outline (core idea, analogies, anticipated challenges). The outline is presented via A2UI as an interactive preview for user steering.

**Execute Phase**: The agent writes the full explanation in plain language, following the outline. The explanation is rendered via AG-UI streaming events. The user can confirm, correct, or extend before grading.

**Reflect Phase**: `learn-grade` examines the explanation for omissions, misconceptions, and transfer problem correctness. Sycophancy correction prevents false positives. If gaps exist, child loops are spawned. If mastery closure criteria are met (score ≥ 0.7, transfer problems solved, retention scheduled), the artifact is persisted.

**Recursion**: Vertical recursion drills into gap concepts (max depth 3). Horizontal escalation re-runs the same concept at `peer` and `skeptic` levels. Recursion floor guards prevent infinite loops on foundational concepts.

### 5.2 AG-UI / A2UI Integration

The AG-UI and A2UI protocols are complementary layers:

| Layer | Protocol | Role | Transport |
|---|---|---|---|
| **Event Streaming** | AG-UI | Real-time text, tool calls, state sync, lifecycle signals | SSE (`/api/v1/ag-ui/stream`) |
| **Surface Generation** | A2UI | Declarative JSON describing UI components to render | WebSocket or POST + SSE push |

**AG-UI Event Types** (emitted by the Rust backend, consumed by the React frontend):
- `text`: Incremental text stream from the LLM.
- `tool_call`: Agent invokes an MCP tool; frontend renders a tool card.
- `tool_result`: Tool result received; frontend updates the card.
- `state_sync`: Agent state update (e.g., "grading in progress").
- `lifecycle`: `run_start`, `run_end`, `error`, `interrupt`.

**A2UI Surface Types** (generated by the agent, rendered by the frontend):
- `createSurface`: Declares a new surface region (e.g., "explanation-card", "gap-drill-panel").
- `updateComponents`: Flat component tree with `id` references. Component types: `Card`, `Column`, `TextField`, `ChoicePicker`, `Button`, `Markdown`, `Chart`.
- `updateDataModel`: JSON Pointer data binding (`/booking/date → "2026-07-01"`).
- `deleteSurface`: Cleanup.

The frontend maintains an **A2UI Catalog** — a whitelist of renderable component types mapped to shadcn/ui primitives. The agent can only reference types in this catalog, ensuring security (no arbitrary HTML/JS injection).

### 5.3 MCP Client Bridge

The Axum server acts as an **MCP client** connecting to a configurable set of MCP servers:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Axum Server   │────▶│  MCP Client     │────▶│  MCP Server 1   │
│   (Rust)        │     │  (Rust)         │     │  (Web Search)   │
│                 │     │                 │────▶│  MCP Server 2   │
│                 │     │                 │     │  (Code Exec)    │
│                 │     │                 │────▶│  MCP Server 3   │
│                 │     │                 │     │  (Knowledge DB) │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

The MCP client is implemented as a Rust crate (`forge-cli` or a dedicated `mcp-client` crate) using the Model Context Protocol spec. It supports:
- **Tool Discovery**: `tools/list` request to enumerate available tools per server.
- **Tool Invocation**: `tools/call` with JSON-RPC 2.0 over stdio or HTTP.
- **Capability Negotiation**: `initialize` handshake with protocol version, capabilities, and client info.
- **Progress & Cancellation**: Token-based progress reporting and cancellation of long-running tool calls.

Tool results are serialized into AG-UI `tool_result` events and streamed to the frontend.

### 5.4 Karpathy Loop Adapter

The Karpathy Loop is a continuous improvement mechanism for the learning agent's pedagogy. It applies three primitives to the learning domain:

1. **Editable Asset**: The `curriculum.json` and `program.md` files that define the learning plan for a goal. The agent modifies these based on learner outcomes.
2. **Scalar Metric**: The **Learning Velocity Score (LVS)** — a composite metric combining concept mastery rate, recursion depth average, and retention pass rate. Computed automatically from learner data.
3. **Time-boxed Cycle**: Each pedagogical experiment (e.g., "try analogy type X for concept Y") runs for a fixed 24-hour window with a cohort of learners. Results are compared against the baseline.

The `fdb-reflection` Quarry engine analyzes learner trajectories, computes LVS, and generates experiment proposals. The agent can autonomously run experiments (with human approval for major changes) and commit winning strategies to the curriculum.

### 5.5 Tauri 2 Multi-Platform Wrapper

The Tauri 2 wrapper reuses the entire React frontend and Rust backend codebase:

```
┌─────────────────────────────────────────────────────────┐
│                      Tauri App                          │
│  ┌─────────────────────┐  ┌─────────────────────────┐ │
│  │   WebView (React)   │  │   Rust Core (Axum)      │ │
│  │   ┌───────────────┐ │  │   ┌───────────────────┐   │ │
│  │   │  PGlite WASM  │ │  │   │  Axum Router      │   │ │
│  │   │  (IndexedDB)  │ │  │   │  (embedded)       │   │ │
│  │   └───────────────┘ │  │   └───────────────────┘   │ │
│  │   ┌───────────────┐ │  │   ┌───────────────────┐   │ │
│  │   │  ElectricSQL  │ │  │   │  Tauri Commands     │   │ │
│  │   │  Sync Client  │ │  │   │  (fs, notify, menu)│   │ │
│  │   └───────────────┘ │  │   └───────────────────┘   │ │
│  └─────────────────────┘  └─────────────────────────┘ │
│           ▲                        │                  │
│           └──────── Tauri IPC ─────┘                  │
└─────────────────────────────────────────────────────────┘
```

In the Tauri build, the Axum server runs as a **background thread** within the Rust process, bound to `localhost` on a random free port. The WebView communicates with it via the same HTTP/SSE/WS interfaces as the browser, but over loopback. Tauri commands provide native capabilities (file system, notifications, system tray, deep links) that the web app accesses through a capability-gated IPC layer.

**Mobile-specific adaptations**:
- Safe-area insets via CSS environment variables (`env(safe-area-inset-top)`).
- Touch-optimized gestures for swipe-to-recurse, pull-to-sync.
- Offline mode UI: sync status indicator, queue length badge.

---

## 6. Data Architecture

### 6.1 Local-First Schema (PGlite + Postgres)

The learner's local database (PGlite) and the cloud Postgres share the same schema, synced via ElectricSQL shapes:

```sql
-- Core learning domain
CREATE TABLE learners (
    id UUID PRIMARY KEY,
    auth_id UUID REFERENCES auth.users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    proficiency_target TEXT, -- 'novice' | 'peer' | 'expert'
    subscription_tier TEXT DEFAULT 'free' -- 'free' | 'plus' | 'pro'
);

CREATE TABLE goals (
    id UUID PRIMARY KEY,
    learner_id UUID NOT NULL REFERENCES learners(id),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'active', -- 'active' | 'completed' | 'archived'
    recursion_floor JSONB DEFAULT '[]',
    curriculum JSONB, -- generated curriculum tree
    created_at TIMESTAMPTZ DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE concepts (
    id UUID PRIMARY KEY,
    goal_id UUID NOT NULL REFERENCES goals(id),
    parent_id UUID REFERENCES concepts(id),
    title TEXT NOT NULL,
    description TEXT,
    depth INT DEFAULT 0,
    audience TEXT DEFAULT 'novice', -- 'novice' | 'peer' | 'skeptic'
    status TEXT DEFAULT 'pending', -- 'pending' | 'explaining' | 'grading' | 'gaps' | 'mastered'
    mastery_score FLOAT,
    created_at TIMESTAMPTZ DEFAULT now(),
    mastered_at TIMESTAMPTZ
);

CREATE TABLE artifacts (
    id UUID PRIMARY KEY,
    concept_id UUID NOT NULL REFERENCES concepts(id),
    goal_id UUID NOT NULL REFERENCES goals(id),
    depth INT NOT NULL,
    audience TEXT NOT NULL,
    explanation_text TEXT NOT NULL,
    grade_id UUID,
    overall_score FLOAT,
    transfer_scores FLOAT[],
    retention_scheduled BOOLEAN DEFAULT FALSE,
    child_artifacts UUID[],
    created_at TIMESTAMPTZ DEFAULT now(),
    closed_at TIMESTAMPTZ
);

CREATE TABLE grades (
    id UUID PRIMARY KEY,
    artifact_id UUID NOT NULL REFERENCES artifacts(id),
    concept_id UUID NOT NULL REFERENCES concepts(id),
    overall_score FLOAT NOT NULL,
    misconceptions_absent FLOAT NOT NULL,
    gaps JSONB, -- array of gap concept references
    transfer_problems JSONB, -- array of problem + expected answer + score
    graded_by TEXT, -- 'agent' | 'human' | 'hybrid'
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE retention_checks (
    id UUID PRIMARY KEY,
    concept_id UUID NOT NULL REFERENCES concepts(id),
    artifact_id UUID NOT NULL REFERENCES artifacts(id),
    scheduled_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    score FLOAT,
    passed BOOLEAN
);

-- Real-time sync tracking
CREATE TABLE sync_metadata (
    table_name TEXT PRIMARY KEY,
    last_sync_at TIMESTAMPTZ,
    shape_handle TEXT
);
```

### 6.2 ElectricSQL Shape Configuration

```typescript
// Shapes define what data syncs between cloud and edge
const learnerShapes = [
  {
    table: 'goals',
    where: "learner_id = '${learnerId}'",
  },
  {
    table: 'concepts',
    where: "goal_id IN (SELECT id FROM goals WHERE learner_id = '${learnerId}')",
  },
  {
    table: 'artifacts',
    where: "goal_id IN (SELECT id FROM goals WHERE learner_id = '${learnerId}')",
  },
  {
    table: 'grades',
    where: "concept_id IN (SELECT id FROM concepts WHERE goal_id IN (SELECT id FROM goals WHERE learner_id = '${learnerId}'))",
  },
];

// Sync initialization
await pg.electric.syncShapeToTable({
  shape: { url: `${BASE_URL}/v1/shape`, params: learnerShapes },
  table: 'local_sync_queue',
  primaryKey: ['id'],
});
```

### 6.3 Row-Level Security (RLS)

All tables are protected by RLS policies enforced at the Postgres level:

```sql
ALTER TABLE goals ENABLE ROW LEVEL SECURITY;
ALTER TABLE concepts ENABLE ROW LEVEL SECURITY;
ALTER TABLE artifacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE grades ENABLE ROW LEVEL SECURITY;

CREATE POLICY goals_owner ON goals
    FOR ALL USING (learner_id = auth.uid());

CREATE POLICY concepts_owner ON concepts
    FOR ALL USING (goal_id IN (SELECT id FROM goals WHERE learner_id = auth.uid()));
```

The Tauri/Web app sends a JWT in the `Authorization` header. The Axum server validates the JWT and sets `app.current_user_id()` for the Postgres connection, which RLS policies use via `auth.uid()`.

---

## 7. Deployment Architecture

### 7.1 Web Deployment

```
┌────────────────────────────────────────────────────────────┐
│                    Cloud Infrastructure                       │
│  ┌─────────────┐   ┌─────────────┐   ┌────────────────────┐│
│  │   CDN       │   │   Axum      │   │   Postgres (RDS)   ││
│  │  (static)   │◄──│   Servers   │◄──│  + ElectricSQL     ││
│  │             │   │  (K8s/ECS)  │   │  + pgvector        ││
│  └─────────────┘   └─────────────┘   └────────────────────┘│
│                           │                                  │
│                    ┌──────┴──────┐                          │
│                    │  MCP Servers │                          │
│                    │  (external)  │                          │
│                    └─────────────┘                          │
└────────────────────────────────────────────────────────────┘
```

For the web build, the Vite output is uploaded to a CDN (Cloudflare R2 / S3). The Axum API server runs on Kubernetes or ECS. The `build.rs` embeds the SPA for the standalone binary, but in production, the CDN handles static assets while the Axum server handles API and SSE.

### 7.2 Desktop / Mobile Deployment

```
┌────────────────────────────────────────────────────────────┐
│                    Tauri Build Pipeline                       │
│  ┌─────────────┐   ┌─────────────┐   ┌────────────────────┐│
│  │   Vite      │   │   Rust      │   │   Tauri Bundler    ││
│  │   Build     │──▶│   Compile   │──▶│   (.dmg, .msi,     ││
│  │             │   │  (Axum lib) │   │   .app, .apk)      ││
│  └─────────────┘   └─────────────┘   └────────────────────┘│
│                           │                                  │
│                    ┌──────┴──────┐                          │
│                    │   Code Sign  │                          │
│                    │   Notarize   │                          │
│                    └─────────────┘                          │
│                           │                                  │
│                    ┌──────┴──────┐                          │
│                    │   Updater    │                          │
│                    │   (GitHub    │                          │
│                    │   Releases)  │                          │
│                    └─────────────┘                          │
└────────────────────────────────────────────────────────────┘
```

The Tauri build pipeline:
1. `pnpm build` → Vite production build (React SPA).
2. `cargo build` → Rust compiles with embedded static assets via `build.rs` + `include_dir`.
3. `tauri build` → Bundles into platform-specific installers.
4. Code signing (Apple notarization, Windows EV cert, Android Play Store signing).
5. Tauri updater checks GitHub Releases for new versions and auto-downloads.

---

## 8. Security Architecture

### 8.1 Threat Model

| Threat | Mitigation |
|---|---|
| **A2UI Injection** | Agent can only reference whitelisted component types. Frontend validates all A2UI JSON against schema before rendering. No raw HTML/JS execution. |
| **MCP Tool Abuse** | Tool invocation is gated by user approval for destructive actions. MCP servers run in isolated sandboxes (e2b). |
| **Data Leakage** | RLS policies enforce row-level isolation. JWTs are short-lived (15 min) with refresh tokens. |
| **Offline Tampering** | PGlite data is signed with a per-device HMAC key. Sync rejects tampered records. |
| **Supply Chain** | `cargo vet` for Rust dependencies, `pnpm audit` for JS. Reproducible builds via locked lockfiles. |

### 8.2 Capability Model (Tauri)

Tauri 2 uses a **capability-based permission model** (`capabilities/default.json`):

```json
{
  "identifier": "default",
  "description": "Default capabilities for the Feynman app",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-read-app-data",
    "fs:allow-write-app-data",
    "notification:default",
    "updater:default"
  ]
}
```

The frontend cannot access capabilities not explicitly granted. Runtime permissions (e.g., camera, microphone) require user consent.

---

## 9. Observability & Monitoring

| Layer | Tool | Metrics |
|---|---|---|
| **Rust Backend** | `tracing` + `tracing-subscriber` + OpenTelemetry | Request latency, SSE connection count, MCP tool call latency, Feynman loop duration. |
| **Frontend** | `web-vitals` + custom analytics | Time-to-first-paint, A2UI render time, offline/online transitions. |
| **Database** | Postgres `pg_stat_statements` + custom views | Query latency, sync lag, RLS policy evaluation time. |
| **Karpathy Loop** | Quarry reflection metrics | LVS per cohort, experiment win rate, curriculum drift. |

---

## 10. Technology Stack Summary

| Layer | Technology | Version |
|---|---|---|
| **Frontend** | React | 19.2.x |
| | Vite | 7.x |
| | Tailwind CSS | 4.x |
| | shadcn/ui | latest (Base UI primitives) |
| | TanStack Router | 1.x |
| | @assistant-ui/react | latest |
| | @prometheus-ags/prometheus-entity-management | 3.0.0-alpha |
| | @electric-sql/pglite | 0.3.x |
| | @electric-sql/pglite-sync | latest |
| | drizzle-orm | latest |
| **Desktop/Mobile** | Tauri | 2.x |
| | Rust | 1.85+ |
| **Backend** | Axum | 0.8.x |
| | Tokio | 1.x |
| | sqlx | 0.8.x |
| | tokio-postgres | 0.7.x |
| | arc-swap | 1.x |
| | async-graphql | 7.x |
| **Data** | PostgreSQL | 17+ |
| | ElectricSQL | latest |
| | pgvector | 0.4.x |
| | pgsodium (Vault) | latest |
| **Agent** | LLM (OpenAI/Anthropic/Local) | GPT-4o / Claude 3.5 / Flint LLM |
| | MCP | 2025-06 spec |
| | AG-UI / A2UI | v0.9 / v1.0 |
| | Feynman Loop | v1.0 (from prometheus-skill-pack) |
| **Infrastructure** | Docker / Kubernetes | latest |
| | Cloudflare / AWS | latest |
| | GitHub Actions (CI) | latest |

---

## 11. Open Questions & Risks

| ID | Question / Risk | Status |
|---|---|---|
| RQ-01 | PGlite single-user limitation: How does multi-tab usage work? | Investigate — possible to use `SharedWorker` for PGlite instance sharing. |
| RQ-02 | ElectricSQL sync performance for large artifact corpora (>100MB per learner). | Need benchmarking; may require shape sharding. |
| RQ-03 | MCP server authentication and rate limiting strategy. | Need design doc for credential management and quota enforcement. |
| RQ-04 | Tauri 2 mobile stability relative to Flutter/RN for production. | Evaluate — current plan is Tauri 2 with Flutter fallback if mobile proves unstable. |
| RQ-05 | `@prometheus-ags/prometheus-entity-management` 3.0.0-alpha API stability. | Track upstream; pin version and have migration plan. |
| RQ-06 | Karpathy Loop autoresearch for pedagogy — safety boundaries for autonomous curriculum changes. | Need human-in-the-loop gate for non-trivial changes. |
| RQ-07 | A2UI component catalog extensibility — how do 3rd-party plugins add new components? | Need plugin registry design. |

---

*End of Architecture Document*
