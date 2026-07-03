# Prometheus Feynman Learning Agent — Master Document Index

## Project Overview

The **Prometheus Feynman Learning Agent (PFLA)** is a cross-platform, AI-native learning application that enables users to master any subject using the Feynman Technique — orchestrated by an intelligent agent layer and built on cutting-edge local-first architecture. The platform extends into a **three-sided learning marketplace** where students learn, experts create **digital coaching personas**, and certified students become **masters** who earn money coaching others via live text, async video, and WebRTC video conferencing.

**Core Technologies**:
- **Frontend**: React 19 + Vite 7 + shadcn/ui (Base UI) + Tailwind CSS v4 + TanStack Router
- **Agent UI**: AG-UI (CopilotKit) + A2UI (Google) protocols for dynamic, AI-generated surfaces
- **Backend**: Rust Axum server with embedded static asset serving via `build.rs`
- **Data**: Flint-Forge Postgres cloud backend + PGlite WASM local database + ElectricSQL bidirectional sync
- **Desktop/Mobile**: Tauri 2 wrapper with shared Rust core and web frontend
- **AI Integration**: MCP client for external tool access, Vercel AI SDK + assistant-ui for streaming
- **State Management**: `@prometheus-ags/prometheus-entity-management` npm 3.0.0-alpha for reactive entity graphs
- **Continuous Improvement**: Karpathy Loop-inspired experiment engine for autonomous pedagogical optimization
- **Video Conferencing**: `flint-realtime-fabric` WebRTC SFU/MCU + signaling + recording + transcription
- **Marketplace**: Stripe Connect for revenue sharing, digital coach catalog, master certification pipeline, Creator Studio for persona building

---

## Document Set

### Base Platform Documents (P0–P10)

| # | Document | File Path | Purpose |
|---|---|---|---|
| 1 | **System Architecture** | `docs/prometheus-feynman-learning-agent-architecture.md` | Comprehensive architecture document covering all layers: client (React/PGlite/Tauri), edge (Axum API/AG-UI/A2UI/MCP), data (Flint-Forge Postgres/ElectricSQL), and agent (Feynman Loop/Karpathy Loop). Includes deployment diagrams, data schema, security architecture, and technology stack summary. |
| 2 | **Functional Specification** | `docs/prometheus-feynman-learning-agent-functional-spec.md` | Detailed functional requirements, user stories, API specifications (Feynman Loop, A2UI Surface, MCP Invocation), data contracts, state machines, error handling, accessibility, and localization. Covers all 11 feature areas from Goal Management to Subscription. |
| 3 | **Implementation Plan** | `docs/prometheus-feynman-learning-agent-implementation-plan.md` | Phased implementation roadmap (10 phases, ~41 weeks). Includes monorepo structure, dependency graph, team composition, risk mitigation, and solo/small-team adaptation guidance. Each phase has exit criteria and estimated effort. |
| 4 | **Business Model & Monetization** | `docs/prometheus-feynman-learning-agent-business-model.md` | Freemium-to-subscription strategy with three tiers (Free/Plus/Pro) and Enterprise. Covers value proposition, target personas, TAM/SAM/SOM, unit economics, LTV/CAC, revenue projections, go-to-market strategy, competitive positioning, pricing experiments, and subscription enforcement technical design. |

### Marketplace Addenda (P11–P20)

| # | Document | File Path | Purpose |
|---|---|---|---|
| 5 | **System Architecture Addendum** | `docs/prometheus-feynman-learning-agent-architecture-addendum.md` | Extends the base architecture with the Coach Catalog (A2A Agent Registry), Creator Studio (corpus grounding, LoRA fine-tuning, quality gates), Master Certification (three-pillar pipeline, W3C Verifiable Credentials), Live Coaching (WebRTC via `flint-realtime-fabric`), Revenue Share Engine (Stripe Connect), and Video Conferencing Infrastructure (SFU/MCU, signaling, recording). Includes schema additions, ElectricSQL sync shapes, deployment architecture, and integration with the `jesus-twin` methodology (reference only). |
| 6 | **Functional Specification Addendum** | `docs/prometheus-feynman-learning-agent-functional-spec-addendum.md` | Detailed functional requirements for the marketplace: Coach Catalog & Discovery (FEAT-012), Creator Studio (FEAT-013), Master Certification Pipeline (FEAT-014), Live Coaching Sessions (FEAT-015), Video Conferencing Infrastructure (FEAT-016), and Revenue & Marketplace Engine (FEAT-017). Includes user stories, functional requirements, UI surfaces (A2UI), API specifications, state machines, and edge cases. |
| 7 | **Implementation Plan Addendum** | `docs/prometheus-feynman-learning-agent-implementation-plan-addendum.md` | Phased implementation roadmap for marketplace features (10 phases, P11–P20, ~40 weeks). Covers Coach Catalog, Creator Studio (core + advanced with LoRA), Master Certification, Live Coaching (text + async), Video Conferencing (flint-realtime-fabric), Revenue Share, Mobile Video (Tauri), AI Augmentation, and Launch & Scale. Includes dependency graph, team composition, risk mitigation, and performance targets. |
| 8 | **Business Model Addendum** | `docs/prometheus-feynman-learning-agent-business-model-addendum.md` | Three-sided marketplace business model: Digital Coach Marketplace (usage fees, subscriptions, Creator Studio tiers), Live Coaching Marketplace (session fees, revenue share by master tier, AI premium), and Revenue Share Engine (Stripe Connect, payouts, fraud detection, tax handling). Includes 5-year financial projections, unit economics, network effects, competitive differentiation, go-to-market strategy, and key metrics. |

---

## Key Research Findings

### Local Skills Discovered
- **`feynman-loop` skill**: PMPO-driven learning cycle (Spec → Plan → Execute → Reflect) with recursion guards, horizontal escalation (novice→peer→skeptic), and three mastery closure criteria.
- **Artifact Refiner skill**: Provides scaffolding patterns for React 19 + Vite 8 + shadcn/ui + Tauri 2 + Axum, already validated in the prometheus-skill-pack repository.
- **Flint-Forge project**: Rust workspace with `fdb-*` crates for domain, auth, postgres, realtime, reflection, gateway. Provides PostgreSQL LISTEN/NOTIFY real-time event bus, ArcSwap hot-state patterns, and A2UI/AG-UI research.
- **`jesus-twin` project**: Reference architecture for creating a grounded, voice-faithful digital persona using RAG-first corpus grounding, coverage gate, multi-protocol surfaces (AG-UI, A2A, MCP, REST), and fine-tuning (Unsloth LoRA on Gemma 4). Used **only as an architectural methodology example** for the Creator Studio; no religious product is proposed.

### External Research
- **AG-UI (CopilotKit)**: Event streaming protocol for real-time agent-frontend communication. Supports text, tool calls, state sync, lifecycle events over SSE.
- **A2UI (Google)**: Declarative JSON protocol for AI-generated UI surfaces. Security-first, transport-agnostic, flat component tree with id references.
- **A2A (Google)**: Agent-to-Agent protocol for cross-agent collaboration. Coaches expose Agent Cards (capabilities, pricing, endpoint) via A2A registry; the PFLA orchestrator discovers and delegates.
- **PGlite + ElectricSQL**: WASM PostgreSQL in the browser (< 3MB gzipped), with live queries, sync shapes, and bidirectional cloud sync. Single-user limitation handled by design.
- **Tauri 2**: Desktop + mobile from one codebase. 5-10MB bundles vs 120MB+ Electron. Uses OS webview (WKWebView, WebView2, WebKitGTK).
- **Karpathy Loop**: Autonomous experiment loop (editable asset + scalar metric + time-boxed cycle). Applied to pedagogy as LVS (Learning Velocity Score) optimization. Also applied to the Master Certification pipeline.
- **@prometheus-ags/prometheus-entity-management**: Normalized, globally-reactive entity graph store for React. Replaces per-view cache models.
- **MCP (Model Context Protocol)**: Anthropic's open standard. Rust client implementation connects to web search, code execution, knowledge retrieval servers. MCP servers can be tools for coaches.
- **Vercel AI SDK + assistant-ui**: YC-backed (9.9K stars), production-grade AI chat primitives with adapters for AI SDK, LangGraph, LangChain, Mastra.
- **Unsloth**: Fast LoRA fine-tuning library (2x faster, 70% less memory). Used for Creator Studio voice fidelity fine-tuning.
- **mediasoup / pion**: WebRTC SFU/MCU libraries for video conferencing. Used in `flint-realtime-fabric`.
- **Whisper (OpenAI)**: ASR for real-time transcription and async video messaging.
- **Stripe Connect**: Marketplace payment infrastructure for revenue sharing with masters.
- **W3C Verifiable Credentials**: Standard for issuing cryptographically signed digital badges (Master Certification).

---

## Quick Start Reference

### Architecture Highlights
- **Local-First**: PGlite in browser/Tauri with ElectricSQL sync. Works offline, syncs when online.
- **Agent-Native UI**: A2UI/AG-UI enables the AI to render forms, cards, diagrams, and charts dynamically — not just chat text.
- **MCP Client**: Rust Axum server connects to external tools (search, code execution, knowledge DB) via the Model Context Protocol.
- **Tauri Multi-Platform**: Desktop (Win/Mac/Linux) and mobile (iOS/Android) from the same React + Rust codebase.
- **Karpathy Loop**: Autonomous pedagogical experiments run nightly, improving learning outcomes based on measured data.
- **Video Conferencing**: WebRTC 1:1 and group sessions via `flint-realtime-fabric`, with AI augmentation (whisper suggestions, shared persona mode), recording, and transcription.
- **Marketplace**: Three-sided economy — students learn, experts create coaches, masters earn money teaching. Built on Stripe Connect, A2A Agent Registry, and W3C Verifiable Credentials.

### Monetization Highlights
- **Free**: 3 goals, novice-only, web-only, 3 free coaches — designed to create habit and trigger upgrade at retention moment.
- **Plus ($12.99/mo)**: Unlimited goals, all audiences, retention scheduling, artifact library, desktop app, offline mode, all free + basic paid coaches, text-only live coaching (2 hrs/mo).
- **Pro ($29.99/mo)**: Plus + Karpathy insights, custom MCP tools, priority LLM, API access, mobile app, premium coaches (fine-tuned LoRA), video coaching (10 hrs/mo), async messaging (5/mo), full master certification, AI premium included, recording included.
- **Enterprise ($50/user/mo or $10K/yr)**: Team goals, manager dashboards, SSO/SAML, on-premise option, custom curricula, unlimited video + group sessions, white-label coaches, dedicated success manager.
- **Creator Studio**: Free (prompt-only, 1 coach), Pro ($29/mo, LoRA fine-tuning, 5 coaches), Enterprise ($299/mo, unlimited, custom models, white-label, API access).
- **Live Coaching**: Masters set their own prices ($15–150/hour). Platform fee: 20–30% depending on master tier. AI premium: +$5 (whisper) or +$10 (shared) per session.

### Implementation Critical Path
**Base MVP**: P0 (Foundation) → P1 (Local-First Data) → P2 (Feynman Loop) → P5 (LLM Integration) → P10 (Launch) = **~21 weeks** for a functional MVP (web-only, no Tauri/Karpathy initially).

**Marketplace MVP**: P11 (Coach Catalog) → P12 (Creator Studio Core) → P15 (Live Coaching Text) → P17 (Revenue Engine) = **~17 weeks** after P5 completion.

**Full Platform with Video**: P0–P5 → P11–P13 → P15–P17 → P16 (Video Conf) → P19 (AI Augmentation) → P20 (Launch) = **~50–60 weeks** for a team of 6–8 engineers.

### File Locations
All documents are in the workspace directory:
```
/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/docs/
├── prometheus-feynman-learning-agent-architecture.md           # Base architecture
├── prometheus-feynman-learning-agent-functional-spec.md        # Base functional spec
├── prometheus-feynman-learning-agent-implementation-plan.md      # Base implementation plan
├── prometheus-feynman-learning-agent-business-model.md            # Base business model
├── prometheus-feynman-learning-agent-architecture-addendum.md    # Marketplace architecture
├── prometheus-feynman-learning-agent-functional-spec-addendum.md # Marketplace functional spec
├── prometheus-feynman-learning-agent-implementation-plan-addendum.md # Marketplace implementation plan
├── prometheus-feynman-learning-agent-business-model-addendum.md  # Marketplace business model
└── README.md                                                    # This index
```

---

*Master Index — Version 2.0.0 — 2026-07-01*
