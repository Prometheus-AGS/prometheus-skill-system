# Research Report: AG-UI / A2UI / MCP App UI Frameworks

**Research Date:** 2026-07-01
**Topic:** Agent-User Interaction Protocols, Generative UI Specifications, and MCP App Design Patterns for the Prometheus Deep-Research Skill
**Researcher:** Orchestrator Sub-Agent

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [AG-UI Protocol: Deep Dive](#2-ag-ui-protocol-deep-dive)
3. [A2UI Protocol: Deep Dive](#3-a2ui-protocol-deep-dive)
4. [The AG-UI / A2UI Relationship](#4-the-ag-ui--a2ui-relationship)
5. [assistant-ui React Framework](#5-assistant-ui-react-framework)
6. [MCP App Design Patterns](#6-mcp-app-design-patterns)
7. [Exposing AG-UI from an MCP Server (Rust/Axum)](#7-exposing-ag-ui-from-an-mcp-server-rustaxum)
8. [Prometheus Ecosystem Context](#8-prometheus-ecosystem-context)
9. [Unified Deep-Research MCP App: Design Vision](#9-unified-deep-research-mcp-app-design-vision)
10. [Recommendations for Prometheus](#10-recommendations-for-prometheus)
11. [Appendix: Sources & Citations](#11-appendix-sources--citations)

---

## 1. Executive Summary

The agentic UI landscape has converged around three complementary open protocols in 2025-2026:

| Protocol | Layer | Maintainer | Purpose |
|----------|-------|------------|---------|
| **MCP** | Agent ↔ Tools & Data | Anthropic (open standard) | Connects agents to external tools, APIs, data sources |
| **A2A** | Agent ↔ Agent | Google / Linux Foundation | Connects agents to other agents for task delegation |
| **AG-UI** | Agent ↔ User | CopilotKit + partners | Connects agents to user-facing applications via event streaming |
| **A2UI** | Generative UI | Google (open source) | Declarative UI spec for agents to "speak UI" safely across trust boundaries |

**Key Insight:** AG-UI and A2UI are *complementary*, not competing. AG-UI is the **transport** (how agents and UIs communicate at runtime); A2UI is the **content format** (what UI to render). A unified deep-research skill should support both: AG-UI for streaming research events (tool calls, state updates, progress), and A2UI for rendering research artifacts (charts, entity graphs, citation panels, timelines) as native UI components.

**MCP Apps** (SEP-1865, Nov 2025) allow MCP servers to expose interactive UI resources via `ui://` URIs, rendered in sandboxed iframes. This is a different paradigm from AG-UI/A2UI — more focused on tool-specific embedded interfaces rather than streaming agent conversations.

---

## 2. AG-UI Protocol: Deep Dive

### 2.1 What is AG-UI?

AG-UI (Agent–User Interaction Protocol) is an **open, lightweight, event-based protocol** that standardizes how AI agents connect to user-facing applications. It was developed by CopilotKit and several agent framework partners, and is now supported by major frameworks including LangGraph, CrewAI, Mastra, Pydantic AI, Microsoft Agent Framework, and AG2 (AutoGen 2).

**Official Docs:** https://docs.ag-ui.com/introduction

### 2.2 Core Design Philosophy

AG-UI addresses the fundamental shift from traditional request/response APIs to **long-running, streaming, non-deterministic agent interactions**. Agents need to:

- Stream intermediate work across multi-turn sessions
- Control application UI non-deterministically
- Mix structured + unstructured I/O (text, voice, tool calls, state updates)
- Support user-interactive composition (sub-agents, human-in-the-loop)

### 2.3 Event Types (The Protocol Grammar)

The AG-UI specification defines approximately **26 event types** across 5 categories:

#### Lifecycle Events
| Event | Purpose |
|-------|---------|
| `RUN_STARTED` | Agent begins execution |
| `RUN_FINISHED` | Agent completes successfully |
| `RUN_ERROR` | Error occurred during execution |
| `STEP_STARTED` | A named step within the run begins |
| `STEP_FINISHED` | A named step completes |

#### Text Message Events
| Event | Purpose |
|-------|---------|
| `TEXT_MESSAGE_START` | Streaming text message begins |
| `TEXT_MESSAGE_CONTENT` | Incremental text content chunk |
| `TEXT_MESSAGE_END` | Text message stream ends |
| `THINKING_START` / `THINKING_END` | Reasoning/thinking boundaries |
| `THINKING_TEXT_MESSAGE_*` | Thinking content streaming |

#### Tool Call Events
| Event | Purpose |
|-------|---------|
| `TOOL_CALL_START` | Tool execution begins |
| `TOOL_CALL_ARGS` | Tool arguments are being streamed |
| `TOOL_CALL_END` | Tool call completes |
| `TOOL_CALL_RESULT` | Tool execution result |

#### State Management Events
| Event | Purpose |
|-------|---------|
| `STATE_SNAPSHOT` | Complete state snapshot of the agent |
| `STATE_DELTA` | JSON Patch (RFC 6902) delta update |
| `MESSAGES_SNAPSHOT` | All messages in the conversation |
| `ACTIVITY_SNAPSHOT` / `ACTIVITY_DELTA` | Activity/step state |

#### Special Events
| Event | Purpose |
|-------|---------|
| `RAW` | Pass-through from external systems |
| `CUSTOM` | Application-specific custom events |

### 2.4 Transport

AG-UI supports multiple transports:
- **SSE (Server-Sent Events)** — primary, works over standard HTTP
- **WebSockets** — for bidirectional, low-latency scenarios
- **HTTP streaming** — for simpler deployments

### 2.5 Framework Integrations (as of mid-2026)

| Framework | Status | Notes |
|-----------|--------|-------|
| LangGraph | ✅ Supported | First-party partnership |
| CrewAI | ✅ Supported | First-party partnership |
| Mastra | ✅ Supported | Community |
| Pydantic AI | ✅ Supported | Community |
| Microsoft Agent Framework | ✅ Supported | First-party (Python + .NET) |
| AG2 (AutoGen 2) | ✅ Supported | First-party with A2UIAgent |
| OpenAI Agent SDK | 🔄 In Progress | — |
| Cloudflare Agents | 🔄 In Progress | — |

**SDKs Available:** TypeScript (`@ag-ui/client`, `@ag-ui/core`), Python, Kotlin, Go, Dart, Java, Rust, .NET, Nim.

### 2.6 AG-UI Handshake with Other Protocols

AG-UI contributors have added "handshakes" allowing AG-UI to "front for" agents through MCP and A2A protocols. This means:
- An AG-UI client app can seamlessly use an MCP-capable agent
- An AG-UI client app can seamlessly use an A2A-capable agent
- AG-UI acts as the universal UI layer atop heterogeneous backend protocols

**Source:** https://docs.ag-ui.com/agentic-protocols

---

## 3. A2UI Protocol: Deep Dive

### 3.1 What is A2UI?

A2UI (Agent to UI) is an **open-source, declarative UI protocol** created by Google that allows AI agents to generate rich, interactive user interfaces that render natively across web, mobile, and desktop — **without executing arbitrary code**.

**Official Site:** https://a2ui.org/
**GitHub:** https://github.com/google/A2UI
**License:** Apache 2.0

### 3.2 Core Problem Solved

In multi-agent systems, agents often run remotely (different servers, organizations). They cannot directly manipulate your UI. The traditional approach was sending HTML/JS in sandboxed iframes, which is heavy, visually disjointed, and introduces security complexity.

A2UI provides a format that is **"safe like data, but expressive like code."**

### 3.3 Message Types (v0.9.1 Current / v1.0 Candidate)

The server-to-client protocol defines four primary message types:

| Message Type | Purpose |
|--------------|---------|
| `createSurface` | Signals client to create a new UI surface and begin rendering |
| `updateComponents` | Provides component definitions to add/update in a surface |
| `updateDataModel` | Provides new data for a surface's data model |
| `deleteSurface` | Removes a surface and its contents |

**Client-to-server messages:**
- `action` — User interaction with a component (e.g., button click)
- Capabilities/metadata exchange via transport-specific handshakes

### 3.4 Component Catalog (Basic)

A2UI defines a basic catalog of components. The agent can only use pre-approved components from the client's catalog — no arbitrary code execution.

**Core components:** Text, Button, Image, Row, Column, Card, List, TextField, Modal, Tabs, Slider, DateTimeInput, Checkbox, MultipleChoice, and more.

### 3.5 Transport Agnostic

A2UI is designed to be **transport-agnostic**. It can be delivered over:
- **A2A** (Agent-to-Agent) protocol
- **AG-UI** protocol (as a payload inside AG-UI events)
- **REST** / SSE / WebSockets (planned/feasible)
- **MCP Apps** (via `ui://` resource scheme, though this uses iframe-based HTML rendering)

The transport contract requires: reliable delivery, message framing, metadata support, and optional bidirectional capability.

### 3.6 Key Design Principles

1. **LLM-Friendly:** Flat component list (adjacency list) with ID references — easy for LLMs to generate incrementally without perfect nested JSON
2. **Progressive Rendering:** JSONL streaming — client starts rendering before the full UI is generated
3. **Framework-Agnostic:** Same payload renders on Angular, Flutter, React, Lit, native mobile
4. **Security-First:** Declarative data only — no executable code. Agents request components from the client's trusted catalog
5. **Separation of Concerns:** UI structure (components), application data (data model), and client rendering are cleanly separated

### 3.7 Ecosystem Status (as of mid-2026)

| Integration | Status |
|-------------|--------|
| AG-UI / CopilotKit | ✅ Day-zero compatibility |
| A2A Protocol | ✅ Native transport support |
| Google ADK | ✅ Native integration |
| AG2 (AutoGen 2) | ✅ A2UIAgent native |
| Vercel json-renderer | ✅ POC support |
| Oracle Agent Spec | ✅ Full support |
| Flutter (GenUI SDK) | ✅ Uses A2UI under the covers |
| React Native renderer | ✅ Community |
| Angular renderer | ✅ Official |
| Lit renderer | ✅ Official |

---

## 4. The AG-UI / A2UI Relationship

### 4.1 The Classic Analogy

| Layer | Protocol | Role | Analogy |
|-------|----------|------|---------|
| **Transport** | AG-UI | *How* agents and UIs communicate | HTTP/TCP for agent UIs |
| **Content** | A2UI | *What* UI to render | HTML for agent UIs |
| **Orchestration** | A2A | *Who* to collaborate with | DNS + RPC for agents |
| **Tools** | MCP | *What* tools to use | USB-C for AI |

### 4.2 Practical Integration Pattern

```
┌─────────────┐      AG-UI events      ┌─────────────┐
│   React     │ ◄─────────────────────► │  Agent      │
│   Frontend  │  (SSE/WebSocket)        │  Backend    │
└─────────────┘                         └─────────────┘
       │                                        │
       │ receives A2UI payload inside           │ generates A2UI
       │ AG-UI CUSTOM or STATE_SNAPSHOT         │ JSON as structured
       │ event                                  │ output from LLM
       ▼                                        ▼
┌─────────────────────────────────────────────────────┐
│  A2UI Renderer (React component)                     │
│  - Parses A2UI JSON                                  │
│  - Maps to native React components                   │
│  - Binds to data model                               │
│  - Handles actions back to agent                     │
└─────────────────────────────────────────────────────┘
```

**Key Quote from Google Developers Blog:**
> "A2UI defines *what* UI to render — it's a declarative spec for describing UI components. AG-UI defines *how* agents and UIs communicate at runtime — the event stream, state synchronization, and interaction lifecycle. They're complementary. An agent can use AG-UI to stream events to the frontend, and one of those events can carry an A2UI payload that describes a UI component to render."

**Source:** https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/

---

## 5. assistant-ui React Framework

### 5.1 Overview

**assistant-ui** is a React component library for building conversational AI interfaces. It provides headless UI primitives (Thread, Message, Composer, etc.) that can be connected to various backend runtimes.

**Website:** https://www.assistant-ui.com/
**NPM:** `@assistant-ui/react`

### 5.2 Runtime Adapters

assistant-ui supports multiple backend protocols via runtime adapters:

| Adapter | Package | Purpose |
|---------|---------|---------|
| AG-UI | `@assistant-ui/react-ag-ui` | Connect to any AG-UI-compliant agent |
| A2A | `@assistant-ui/react-a2a` | Connect to A2A v1.0 agents |
| LangGraph | `@assistant-ui/react-langgraph` | Connect to LangGraph SDK agents |
| OpenAI | `@assistant-ui/react-openai` | Direct OpenAI API integration |
| CopilotKit | built-in | Via CopilotKit runtime |

### 5.3 AG-UI Integration Pattern

```typescript
import { AssistantRuntimeProvider } from "@assistant-ui/react";
import { HttpAgent } from "@ag-ui/client";
import { useAgUiRuntime } from "@assistant-ui/react-ag-ui";

function Provider({ children }: { children: React.ReactNode }) {
  const agent = useMemo(
    () => new HttpAgent({ url: process.env.NEXT_PUBLIC_AGUI_AGENT_URL! }),
    [],
  );
  const runtime = useAgUiRuntime({ agent });

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      {children}
    </AssistantRuntimeProvider>
  );
}
```

**Key Features:**
- Multi-thread support with "New Thread" button
- Tool result rendering
- Client-side tool execution
- Interrupt handling (human-in-the-loop)
- Full streaming support

### 5.4 Relationship to AG-UI

assistant-ui is **a client implementation** of the AG-UI protocol. It consumes AG-UI events and renders them using React components. The `@assistant-ui/react-ag-ui` adapter specifically wraps an `@ag-ui/client` agent in an assistant-ui runtime.

**Important:** assistant-ui is one of several AG-UI client options. CopilotKit is the other major React client, and there are also official vanilla JS clients, Vue/Angular SDKs, and terminal-based clients.

---

## 6. MCP App Design Patterns

### 6.1 MCP Apps (SEP-1865)

In November 2025, the MCP community introduced the **MCP Apps Extension** (SEP-1865), standardizing how MCP servers can deliver interactive user interfaces to hosts.

**Key Design Decisions:**

1. **Pre-declared UI Resources:** UI templates are resources with the `ui://` URI scheme, registered in tool metadata:
   ```typescript
   // Server registers UI resource
   { uri: "ui://charts/bar-chart", name: "Bar Chart Viewer", mimeType: "text/html+mcp" }
   
   // Tool references it
   { name: "visualize_data", inputSchema: {...}, _meta: { "ui/resourceUri": "ui://charts/bar-chart" } }
   ```

2. **MCP Transport for Communication:** UI components communicate with hosts using the existing MCP JSON-RPC base protocol over `postMessage`.

3. **HTML + Iframe Sandboxing:** Initial spec supports only `text/html` content, rendered in sandboxed iframes. Future iterations may support declarative UI, external URLs, remote DOM, native widgets.

4. **Security-First:**
   - Iframe sandboxing with restricted permissions
   - Pre-declared templates (hosts can review before rendering)
   - Auditable JSON-RPC messages
   - User consent for UI-initiated tool calls

**Source:** https://blog.modelcontextprotocol.io/posts/2025-11-21-mcp-apps/

### 6.2 MCP App vs. AG-UI / A2UI Paradigm

| Aspect | MCP Apps | AG-UI + A2UI |
|--------|----------|--------------|
| **Primary Use Case** | Tool-specific embedded UIs | Streaming agent conversations |
| **Rendering Model** | Pre-built HTML in sandboxed iframe | Native components from declarative JSON |
| **Security Model** | Iframe sandbox | Component catalog whitelist |
| **Interactivity** | Full HTML/JS (within sandbox) | Limited to declared component catalog |
| **Streaming** | Not native (pull-based) | Native (SSE/WebSocket) |
| **Best For** | Complex tool UIs (charts, forms, dashboards) | Real-time agent chat, progress, reasoning |

### 6.3 Building a User-Friendly MCP App

Best practices for MCP app design (2025-2026):

1. **Pre-declare UI Resources:** Register `ui://` resources so hosts can prefetch and validate templates
2. **Separate Static from Dynamic:** Templates are static; tool results hydrate them
3. **Use Standard MCP Messages:** Don't invent custom protocols — use `postMessage` + JSON-RPC
4. **Progressive Enhancement:** Fall back to text/JSON when UI is unavailable
5. **Responsive Design:** HTML templates should work across host window sizes
6. **Accessibility:** Follow WCAG guidelines even in iframe contexts

### 6.4 Existing MCP UI Projects

| Project | Stack | Description |
|---------|-------|-------------|
| **mcp-web-ui** | Go + React | Web-based MCP host with multi-provider LLM support |
| **chat-mcp** | Electron + React | Cross-platform desktop chat app for MCP |
| **interact-mcp** | Python + Gradio | MCP server with web chat interface |
| **mcp-agents** | Python + Streamlit | LangGraph ReAct agent with MCP tool UI |
| **langchain-mcp-client** | Python + Streamlit | Multi-provider LLM MCP client |
| **MCP Web UI** (MegaGrindStone) | Go | Host with SSE streaming, persistent history |

---

## 7. Exposing AG-UI from an MCP Server (Rust/Axum)

### 7.1 The Architecture Challenge

A Prometheus deep-research skill needs to expose its capabilities through multiple interfaces simultaneously:
- **MCP interface** — for tool-based access (Claude, Cursor, etc.)
- **AG-UI interface** — for streaming research to a frontend
- **A2UI interface** — for generative research artifacts (charts, graphs, timelines)
- **REST/CLI interface** — for programmatic access and scripting

### 7.2 Rust/Axum Implementation Pattern

Based on the ecosystem's current best practices (Microsoft Agent Framework, rust-mcp-sdk, redmine-mcp, m3u8-mcp), the recommended architecture is:

```
┌─────────────────────────────────────────────────────┐
│                  Rust Axum Server                     │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐ │
│  │ /mcp        │  │ /agui       │  │ /a2ui        │ │
│  │ (MCP Tools) │  │ (AG-UI SSE) │  │ (A2UI SSE)   │ │
│  └─────────────┘  └─────────────┘  └──────────────┘ │
│  ┌─────────────────────────────────────────────────┐  │
│  │      Shared Research Engine (Core Logic)        │  │
│  │  - Planner, Search, Retriever, Verifier         │  │
│  │  - Knowledge Graph Builder, Citation Manager    │  │
│  │  - Report Generator, Artifact Exporter          │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 7.3 Key Implementation Details

**MCP Server (Axum):**
- Use `rust-mcp-sdk` or `rmcp` crate for MCP protocol implementation
- Support both stdio and Streamable HTTP (SSE) transports
- Port architecture: typically 3000 for gateway, configurable for MCP
- JSON-RPC 2.0 message format over HTTP POST + SSE

**AG-UI Endpoint (Axum):**
- HTTP POST endpoint that accepts user messages and returns SSE stream
- Server-Sent Events emit AG-UI JSON events (`RUN_STARTED`, `TEXT_MESSAGE_CONTENT`, `TOOL_CALL_START`, `STATE_SNAPSHOT`, `RUN_FINISHED`)
- Thread management via `threadId` parameter
- State snapshots for research progress (search queries, sources found, evidence collected)

**A2UI Endpoint (Axum):**
- SSE stream of A2UI JSONL messages (`createSurface`, `updateComponents`, `updateDataModel`)
- Can be multiplexed over the same AG-UI connection as `CUSTOM` events, or served as a separate endpoint
- Research-specific component catalog: CitationCard, SourceList, Timeline, EntityGraph, ConfidenceMeter, ContradictionPanel

### 7.4 Example: AG-UI Event Stream for Research

```json
{"type": "RUN_STARTED", "thread_id": "research-123", "run_id": "run-456"}
{"type": "STEP_STARTED", "step_name": "question_decomposition"}
{"type": "TEXT_MESSAGE_START", "message_id": "msg-1"}
{"type": "TEXT_MESSAGE_CONTENT", "message_id": "msg-1", "content": "Decomposing your query into 5 sub-questions..."}
{"type": "TEXT_MESSAGE_END", "message_id": "msg-1"}
{"type": "STEP_FINISHED", "step_name": "question_decomposition"}
{"type": "STEP_STARTED", "step_name": "web_search"}
{"type": "TOOL_CALL_START", "tool_call_id": "tc-1", "tool_call_name": "tavily_search"}
{"type": "TOOL_CALL_ARGS", "tool_call_id": "tc-1", "args": "{\"query\": \"AG-UI protocol specification 2026\"}"}
{"type": "TOOL_CALL_END", "tool_call_id": "tc-1"}
{"type": "TOOL_CALL_RESULT", "tool_call_id": "tc-1", "content": "{\"results\": [...]}"}
{"type": "STATE_SNAPSHOT", "snapshot": {"phase": "search", "queries_executed": 1, "sources_found": 12, "sources_verified": 0}}
{"type": "STEP_FINISHED", "step_name": "web_search"}
{"type": "RUN_FINISHED", "thread_id": "research-123", "run_id": "run-456"}
```

### 7.5 Rust Ecosystem for MCP + AG-UI

| Crate | Purpose |
|-------|---------|
| `rmcp` | Rust MCP SDK with Axum/Actix support |
| `rust-mcp-sdk` | Async SDK + framework for MCP servers/clients |
| `rust-mcp-axum` | Axum integration for MCP Streamable HTTP |
| `axum` | Web framework for HTTP/SSE endpoints |
| `tokio` | Async runtime |
| `tower-http` | CORS, middleware |
| `serde` + `serde_json` | Protocol serialization |
| `eventsource` | SSE client (for testing) |

**Reference Projects:**
- `redmine-mcp` (Rust + Axum + Tauri + React): https://github.com/yonaka15/redmine-mcp
- `m3u8-mcp` (Rust + Axum + Tauri + React): https://github.com/yonaka15/m3u8-mcp
- `ThinkWatch` (Rust + Axum + React): https://github.com/ThinkWatchProject/ThinkWatch

---

## 8. Prometheus Ecosystem Context

### 8.1 prometheus-entity-management

The `@prometheus-ags/prometheus-entity-management` npm package is a **normalized, globally-reactive entity graph store for React**, built on Zustand. It replaces TanStack Query's per-view cache model with a single application-wide entity graph.

**Key Features:**
- Entity normalization (like a Redux/Apollo cache)
- Global reactivity — all components observing the same entity update automatically
- Graph relationships between entities
- Built on Zustand for minimal boilerplate
- Supports GraphQL, REST, and custom adapters

**Relevance to Deep-Research UI:**
- The entity graph is perfect for managing research entities (sources, citations, claims, people, organizations, topics)
- A deep-research MCP app could use this to maintain a client-side knowledge graph that syncs with the backend via AG-UI `STATE_SNAPSHOT`/`STATE_DELTA` events
- Entities extracted during research can be normalized and queried by the UI

### 8.2 flint-platform-agent / flint-realtime-fabric

Prometheus-AGS maintains `flint-realtime-fabric` and related repositories. These appear to be part of the **Flint platform** — a real-time fabric for agent coordination. While specific public documentation is limited, the naming convention suggests:

- `flint-platform-agent` — likely a platform-level agent runtime or orchestrator
- `flint-realtime-fabric` — real-time event fabric for agent communication
- These may serve as the runtime layer beneath the Prometheus skill system

### 8.3 Existing Prometheus MCP Stack Integration

The Prometheus skill pack already includes several MCP integrations that a deep-research skill should leverage:

| MCP Server | Role in Research Workflow |
|------------|---------------------------|
| **tavily-mcp** | Web search and retrieval (primary search engine) |
| **sequential-thinking** | Reasoning chain / step-by-step analysis |
| **liter-llm** | LLM proxy + MCP tool server (model abstraction) |
| **surreal-memory** | Graph memory + knowledge persistence |
| **forge-rs** | Enrichment, reflection, template generation |
| **prometheus-knowledge** | Knowledge base query (Karpathy KB) |
| **sycophancy-correction** | Bias detection and correction |

---

## 9. Unified Deep-Research MCP App: Design Vision

### 9.1 What Would a Unified Deep-Research MCP App Look Like?

A world-class deep-research MCP app for Prometheus should combine **MCP (tools)**, **AG-UI (streaming)**, and **A2UI (generative artifacts)** into a single cohesive experience:

```
┌────────────────────────────────────────────────────────────────────┐
│                        Research Copilot UI                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │ Chat / Query    │  │ Research Status │  │ Knowledge Graph     │  │
│  │ Input           │  │ Panel           │  │ (A2UI Component)    │  │
│  │                 │  │ (Phase, Sources, │  │                     │  │
│  │                 │  │  Confidence)     │  │                     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ Research Report (streaming markdown + A2UI artifact panels)    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │  │
│  │  │ Timeline │ │ Citation │ │ Source   │ │ Confidence│         │  │
│  │  │ Panel    │ │ Panel    │ │ Card     │ │ Meter    │         │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘         │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
         │                              │
         │ AG-UI events (SSE)           │ A2UI surfaces (SSE)
         ▼                              ▼
┌────────────────────────────────────────────────────────────────────┐
│              Prometheus Deep-Research MCP Server (Rust/Axum)       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Research Engine                                             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │  │
│  │  │ Planner  │ │ Search   │ │ Evidence │ │ Knowledge    │  │  │
│  │  │          │ │ Planner  │ │ Collector│ │ Graph Builder│  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │  │
│  │  │ Verifier │ │ Conflict │ │ Citation │ │ Report       │  │  │
│  │  │          │ │ Resolver │ │ Manager  │ │ Generator    │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │ MCP Tools│ │ AG-UI    │ │ A2UI     │ │ REST/CLI │              │
│  │ Interface│ │ Endpoint │ │ Endpoint │ │ Interface│              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
└────────────────────────────────────────────────────────────────────┘
```

### 9.2 UI/UX Considerations for Research Workflows

Research workflows are **long-running, multi-phase, and evidence-intensive**. The UI must:

1. **Progress Transparency:** Show which phase is active (planning → searching → collecting → verifying → resolving → synthesizing → generating). AG-UI `STEP_STARTED`/`STEP_FINISHED` events map naturally to this.

2. **Source Provenance:** Every claim must be traceable to its source. A2UI `SourceCard` components should show URL, title, snippet, confidence score, and verification status.

3. **Live Evidence Collection:** As the agent finds sources, they should appear in the UI in real-time via `STATE_DELTA` events, not just at the end.

4. **Knowledge Graph Exploration:** Research entities (people, organizations, topics, claims) should be browsable as an interactive graph. A2UI can render this as a custom `EntityGraph` component.

5. **Contradiction Highlighting:** When sources conflict, the UI should visually surface contradictions with confidence scores. This is a unique research UX requirement.

6. **Human-in-the-Loop:** The user should be able to:
   - Add follow-up questions mid-research
   - Reject low-confidence sources
   - Redirect the research angle
   - Request deeper investigation on a specific claim
   AG-UI's interrupt/approval patterns support this.

7. **Persistent Knowledge Assets:** Research should produce durable artifacts (citations.json, knowledge_graph.json, embeddings, entity graph) that other agents can query. The UI should provide download/export for these.

8. **Multi-Platform Access:** The same research should be accessible via:
   - Claude Code / Cursor / Windsurf (MCP tools)
   - Kimi Code / Kimi Work (skill system)
   - Web UI (AG-UI + A2UI)
   - CLI (`prometheus research --topic "..."`)

### 9.3 A2UI Component Catalog for Research

A research-specific A2UI component catalog might include:

| Component | Purpose |
|-----------|---------|
| `CitationCard` | Displays a single citation with URL, title, snippet, confidence |
| `SourceList` | Sortable/filterable list of sources with verification status |
| `Timeline` | Chronological visualization of events discovered |
| `EntityGraph` | Interactive graph of entities and relationships |
| `ConfidenceMeter` | Visual indicator of claim confidence (0-100%) |
| `ContradictionPanel` | Side-by-side comparison of conflicting claims |
| `ResearchPhaseIndicator` | Progress bar showing active research phase |
| `QueryChip` | Display of sub-questions the agent is investigating |
| `EvidenceTable` | Tabular view of evidence with columns for source, claim, confidence |
| `ReportPreview` | Streaming preview of the final report |
| `ExportPanel` | Buttons to download knowledge assets in various formats |

---

## 10. Recommendations for Prometheus

### 10.1 Architecture Recommendation

The Prometheus deep-research skill should adopt a **multi-protocol, multi-surface architecture**:

```
┌─────────────────────────────────────────────────────────────┐
│                 Deep-Research Skill Core                    │
│                  (Rust — unified logic)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ MCP Tool     │  │ AG-UI Event  │  │ A2UI Surface │     │
│  │ Interface    │  │ Stream       │  │ Generator    │     │
│  │ (tools/list) │  │ (SSE)        │  │ (SSE)        │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Shared Pipeline:                                    │  │
│  │  Planner → Search → Collect → Verify → Resolve →     │  │
│  │  Synthesize → Cite → Export                          │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 10.2 Protocol Strategy

| Protocol | Use Case | Implementation Priority |
|----------|----------|------------------------|
| **MCP** | Tool-based access (Claude, Cursor, IDE plugins) | P0 — essential |
| **AG-UI** | Streaming research progress to web/native clients | P0 — essential |
| **A2UI** | Generative research artifacts (charts, graphs, timelines) | P1 — high value |
| **A2A** | Research agent delegation to specialist sub-agents | P2 — future |
| **MCP Apps** | Embedded tool UIs within existing MCP hosts | P2 — future |

### 10.3 Frontend Strategy

| Client | Technology | Use Case |
|--------|------------|----------|
| **Web UI** | React + assistant-ui + `@assistant-ui/react-ag-ui` | Primary research interface |
| **Desktop** | Tauri (Rust) + React + AG-UI client | Cross-platform native app |
| **IDE** | MCP tools only | Claude, Cursor, Windsurf, Roo |
| **CLI** | Rust binary (`prometheus research`) | Scripting, automation |

### 10.4 Integration with Existing Prometheus Stack

- **tavily-mcp:** Primary search retriever
- **sequential-thinking:** Decomposition and reasoning steps
- **surreal-memory:** Store intermediate research state (entity graph, session memory)
- **forge-rs:** Template-based report generation, reflection on quality
- **prometheus-knowledge:** Query existing knowledge base before starting new research
- **sycophancy-correction:** Validate that research doesn't just confirm user biases
- **liter-llm:** Model abstraction (route to appropriate LLM based on task complexity)

### 10.5 Knowledge Asset Strategy

The skill should emit **persistent knowledge objects** (not just disposable reports):

| Asset | Format | Purpose |
|-------|--------|---------|
| `citations.json` | Structured JSON | Machine-readable source metadata |
| `knowledge_graph.json` | JSON / Cypher | Entity-relationship graph |
| `embeddings.npz` | NumPy array | Semantic embeddings for RAG |
| `timeline.json` | JSON | Chronological event data |
| `contradictions.json` | JSON | Conflicting claims with scores |
| `confidence_scores.json` | JSON | Per-claim confidence ratings |
| `follow_up_questions.json` | JSON | Suggested next research directions |
| `source_cache/` | Directory | Cached raw source content |
| `search_trace.json` | JSON | Log of all search queries and results |
| `reasoning_trace.md` | Markdown | Human-readable reasoning log |
| `report.md` | Markdown | Final human-readable report |
| `report.docx` | Word | Formatted report for distribution |

These assets should be stored in a location accessible to other agents (via `surreal-memory` or filesystem) and emitted as AG-UI `STATE_SNAPSHOT` events so the UI can display and export them.

### 10.6 CLI Tooling

Following the pattern of existing Prometheus skills:

```bash
# Research CLI
prometheus-research --topic "AG-UI protocol landscape" --depth deep --output ./research-output/

# Interactive research session
prometheus-research --interactive --gui  # Opens AG-UI web interface

# Research resume
prometheus-research --resume research-session-id

# Research verification
prometheus-research --verify ./research-output/citations.json

# Export to various formats
prometheus-research --export-docx ./research-output/report.md
prometheus-research --export-pdf ./research-output/report.md
```

---

## 11. Appendix: Sources & Citations

### Primary Sources

1. **AG-UI Protocol Official Docs** — https://docs.ag-ui.com/introduction
2. **AG-UI Protocol GitHub** — https://github.com/ag-ui-protocol/ag-ui
3. **A2UI Official Site** — https://a2ui.org/
4. **A2UI GitHub (Google)** — https://github.com/google/A2UI
5. **A2UI Protocol Spec v0.9** — https://github.com/google/A2UI/blob/main/specification/v0_9/docs/a2ui_protocol.md
6. **A2UI v0.9 Announcement** — https://developers.googleblog.com/a2ui-v0-9-generative-ui/
7. **Introducing A2UI (Google Blog)** — https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/
8. **MCP Apps Extension (SEP-1865)** — https://blog.modelcontextprotocol.io/posts/2025-11-21-mcp-apps/
9. **assistant-ui AG-UI Runtime** — https://www.assistant-ui.com/docs/runtimes/ag-ui/overview
10. **assistant-ui React AG-UI NPM** — https://www.npmjs.com/package/@assistant-ui/react-ag-ui
11. **CopilotKit AG-UI Docs** — https://docs.copilotkit.ai/agentic-protocols/ag-ui
12. **Microsoft Agent Framework AG-UI Integration** — https://learn.microsoft.com/en-us/agent-framework/integrations/ag-ui/
13. **AG-UI + Microsoft Agent Framework Blog** — https://techcommunity.microsoft.com/blog/azuredevcommunityblog/building-interactive-agent-uis-with-ag-ui-and-microsoft-agent-framework/4488249
14. **AG-UI Benchmark (26 events)** — https://github.com/namastexlabs/agui-benchmark
15. **AG-UI Events Reference (Python)** — https://docs.agentwire.io/sdk/python/core/events
16. **AG-UI Events Reference (JS)** — https://docs.agentwire.io/sdk/js/core/events
17. **Six Agent Protocols (MindStudio)** — https://www.mindstudio.ai/blog/six-agent-protocols-ai-builders-2026
18. **Agent Protocol Stack (DZone)** — https://dzone.com/articles/mcp-vs-a2a-vs-agui
19. **Agent Protocol Stack (Dev.to)** — https://dev.to/jubinsoni/the-agent-protocol-stack-mcp-vs-a2a-vs-ag-ui-when-to-use-what-6dn
20. **AI Agent Protocol Guide (Ceaksan)** — https://ceaksan.com/en/ai-agent-protocols-mcp-a2a-ucp-ap2-a2ui-ag-ui
21. **Google Developer's Guide to AI Agent Protocols** — https://developers.googleblog.com/developers-guide-to-ai-agent-protocols/
22. **A2UI + ADK Tutorial** — https://atamel.dev/posts/2026/03-30_a2ui_with_adk/
23. **A2UI + AG-UI + CopilotKit Tutorial** — https://copilotkit.ai/blog/build-with-googles-new-a2ui-spec-agent-user-interfaces-with-a2ui-ag-ui
24. **Chainlit A2UI/AG-UI Feature Request** — https://github.com/Chainlit/chainlit/issues/2894
25. **AG2 AG-UI Integration** — https://docs.ag2.ai/latest/docs/user-guide/ag-ui/
26. **AG2 AG-UI Blog** — https://docs.ag2.ai/latest/docs/blog/2026/02/17/AG2-AG-UI-Protocol/
27. **Pydantic AI AG-UI Integration** — https://pydantic.dev/docs/ai/integrations/ui/ag-ui/
28. **LogRocket: Build Real AI with AG-UI** — https://blog.logrocket.com/build-real-ai-with-ag-ui/
29. **Codecademy: AG-UI Protocol** — https://www.codecademy.com/article/ag-ui-agent-user-interaction-protocol
30. **AG-UI + Mastra Tutorial** — https://blog.logrocket.com/build-real-ai-with-ag-ui/
31. **MCP-UI Technical Deep Dive** — https://workos.com/blog/mcp-ui-a-technical-deep-dive-into-interactive-agent-interfaces
32. **UX Design + MCP** — https://uxdesign.cc/why-ux-designers-should-care-about-model-context-protocol-24d34b02c1c9
33. **rust-mcp-sdk** — https://lib.rs/crates/rust-mcp-sdk
34. **Shuttle: SSE MCP Server in Rust** — https://www.shuttle.dev/blog/2025/08/13/sse-mcp-server-with-oauth-in-rust
35. **redmine-mcp (Rust + Axum + Tauri)** — https://github.com/yonaka15/redmine-mcp
36. **m3u8-mcp (Rust + Axum + Tauri)** — https://github.com/yonaka15/m3u8-mcp
37. **ThinkWatch (Rust + Axum + React)** — https://github.com/ThinkWatchProject/ThinkWatch
38. **Prometheus Skill System Docs** — https://github.com/Prometheus-AGS/prometheus-skill-system
39. **prometheus-entity-management (NPM)** — https://www.jsdelivr.com/package/npm/@prometheus-ags/prometheus-entity-management
40. **Prometheus-AGS GitHub** — https://github.com/Prometheus-AGS
41. **STEM Agent Multi-Protocol Paper** — https://arxiv.org/html/2603.22359v1

---

*Report generated for the Prometheus Skill Pack deep-research skill design initiative.*
