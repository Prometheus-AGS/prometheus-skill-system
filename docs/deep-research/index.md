---
title: Prometheus Deep Research Skill — Master Specification
date: 2026-07-03
version: 1.0.0-draft
status: specification
authors: [Prometheus AGS Research Team]
---

# Prometheus Deep Research Skill — Master Specification

> **Status:** Draft specification for community review  
> **Version:** 1.0.0-draft (Phase 2 — Feynman/OKF/Threading/Long-running added)  
> **Date:** 2026-07-03  
> **Scope:** Architecture, feature specification, platform strategy, Feynman learning integration, Google OKF alignment, threaded research, and long-running process management for a world-class, harness-agnostic deep-research skill within the Prometheus Skill Pack ecosystem.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Strategic Context: Why Prometheus Needs Deep Research](#2-strategic-context-why-prometheus-needs-deep-research)
3. [Landscape Analysis: State of Deep Research 2026](#3-landscape-analysis-state-of-deep-research-2026)
4. [Platform Specifications & Portability Strategy](#4-platform-specifications--portability-strategy)
5. [UI Protocol Strategy: AG-UI + A2UI + MCP Apps](#5-ui-protocol-strategy-ag-ui--a2ui--mcp-apps)
6. [Knowledge Asset Architecture](#6-knowledge-asset-architecture)
7. [Recommended Architecture: The Prometheus Research Pipeline](#7-recommended-architecture-the-prometheus-research-pipeline)
8. [Feature Specification](#8-feature-specification)
9. [MCP Server Design](#9-mcp-server-design)
10. [AG-UI / A2UI Research App Design](#10-ag-ui--a2ui-research-app-design)
11. [Native CLI: `prometheus-research`](#11-native-cli-prometheus-research)
12. [Positioning & Competitive Differentiation](#12-positioning--competitive-differentiation)
13. [Integration with Existing Prometheus Stack](#13-integration-with-existing-prometheus-stack)
14. [Project Plan & Roadmap](#14-project-plan--roadmap)
15. [Conclusion & Next Steps](#15-conclusion--next-steps)
16. [References](#16-references)
17. [Feynman Learning Integration](#17-feynman-learning-integration)
18. [Google OKF Alignment & Karpathy Wiki Convergence](#18-google-okf-alignment--karpathy-wiki-convergence)
19. [Threaded/Concurrent Research Architecture](#19-threadedconcurrent-research-architecture)
20. [Long-Running Process Management](#20-long-running-process-management)
21. [Updated References](#21-updated-references)

---

## 1. Executive Summary

After exhaustive research across 50+ sources spanning open-source projects, academic papers, protocol specifications, and platform documentation, we conclude that **the deep-research agent landscape has matured beyond disposable reports to a new paradigm: persistent, queryable knowledge assets**. Every leading project — from GPT Researcher to LangGraph Open Deep Research to MiroThinker — still primarily emits static documents. The opportunity for Prometheus is to build the **first deep-research system that natively produces structured, extensible knowledge packages** while remaining installable and operable across any skill-aware harness.

This specification defines:
- A **10-stage research pipeline** (Planner → Search → Retrieve → Collect → Verify → Resolve → Graph → Cite → Report → Export) implemented as a portable skill.
- A **dual-nature delivery**: both a `SKILL.md`-based portable skill (for Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, Roo, Amp, Gemini) and a native Rust MCP server with AG-UI/A2UI streaming.
- A **`.research` package format** containing knowledge graphs, citations, embeddings, contradictions, confidence scores, and audit trails — not just a report.
- A **unified React + AG-UI research app** that can render as an MCP App, a standalone web UI, or a Tauri desktop app, using `prometheus-entity-management` for reactive state and `flint-platform-agent` for runtime coordination.

**Key differentiator:** Other agents produce reports. Prometheus Research produces **knowledge infrastructure** that other agents can query, extend, and cite.

**Phase 2 discoveries** (added after second research pass):
- **Feynman Learning Integration:** Deep research is a natural learning primitive. The 10-stage pipeline maps onto inquiry-based learning frameworks, and the Feynman loop serves as a quality gate between Report and Export. Research outputs auto-generate curriculum DAGs that feed into `learn-plan`.
- **Google OKF v0.1 Alignment:** Google's Open Knowledge Format (released June 12, 2026) formalizes the Karpathy LLM Wiki pattern into an interoperable standard. The `.research` package should adopt OKF as its base format while preserving research-specific extensions (`confidence`, `verification_status`, `research_stage`, etc.).
- **Threaded/Concurrent Research:** The pipeline supports parallel research threads with per-thread context isolation, deterministic map-reduce merging, and cross-thread citation discovery. The Feynman loop's recursion and horizontal escalation patterns map directly to depth-first and breadth-first research strategies.
- **Long-Running Process Management:** Research processes spanning hours to days use LangGraph-style checkpointing, KBD waypoint tracking, and the Karpathy Loop (focus→reflect→ingest) at micro and macro frequencies. **surreal-memory** serves as the unified knowledge layer — graph + vector + document + relational + time-travel in one system.

---

## 2. Strategic Context: Why Prometheus Needs Deep Research

The Prometheus Skill Pack already provides:
- **Process orchestration** (`kbd-process-orchestrator`, `pmpo-*` skills)
- **Entity management** (`prometheus-entity-skills`, `prometheus-entity-management` npm module)
- **MCP server generation** (`mcp-server` Rust skill)
- **Native agent scaffolding** (`native-agent` with A2A/AG-UI/A2UI/assistant-ui)
- **Knowledge infrastructure** (`surreal-memory`, `prometheus-knowledge`, `forge-rs`, `liter-llm`)
- **Bias correction** (`sycophancy-correction`)
- **Multi-platform distribution** (Claude, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax, Roo, Amp, Gemini)

**What is missing is a unified research capability** that ties these together. Deep research is the natural capstone skill — it exercises the planning, orchestration, retrieval, verification, graph-building, and knowledge-persistence capabilities of the entire stack. Without it, the skill pack has powerful primitives but no "killer app" that demonstrates their combined value.

Moreover, the user's architectural vision (SurrealDB, graph memory, PostgreSQL, CRDT sync, IPFS, MCP) demands a research skill that emits **persistent knowledge objects**, not disposable PDFs. This specification answers that demand.

---

## 3. Landscape Analysis: State of Deep Research 2026

### 3.1 Major Open-Source Projects

| Project | Maturity | Configurability | Benchmark Perf | MCP Support | Model Agnostic | Knowledge Assets | OSS |
|---------|----------|-----------------|----------------|-------------|----------------|------------------|-----|
| **GPT Researcher** | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ✅ Apache-2 |
| **LangGraph OpenDR** | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★★ | ★★★★★ | ★★★★☆ | ✅ MIT |
| **OpenResearcher** | ★★★☆☆ | ★★★☆☆ | N/A (training) | ★★☆☆☆ | ★★★☆☆ | ★★★★★ | ✅ |
| **MiroThinker** | ★★★★☆ | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ★★★☆☆ | ★★★☆☆ | ✅ Partial |
| **HuggingFace ODR** | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★★☆☆ | ✅ |
| **NVIDIA AI-Q** | ★★★★☆ | ★★★★☆ | ★★★★☆ | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★☆☆☆ |

### 3.2 Key Findings

1. **GPT Researcher** (27.9k stars, v3.5.0) is the most mature but not SOTA on benchmarks. It uses a Planner → Execution → Publisher pattern with parallel agents, supports local documents + web, and outputs PDF/Word/Markdown. It can be installed as a Claude Skill but lacks native MCP integration and persistent knowledge assets.

2. **LangGraph Open Deep Research** is the most configurable and MCP-native. It uses a LangGraph-based workflow with plan-and-execute + reflection, supports many model providers, and has production deployment options (LangGraph Studio, Platform, OAP). Its weakness is LangChain ecosystem dependency.

3. **MiroThinker** achieves SOTA open-source performance (74% BrowseComp, 82.7% GAIA) via "interactive scaling" — training agents for more tool calls rather than bigger models. It demonstrates that smaller models (30B) with good training can outperform much larger models. However, it is model-centric rather than framework-centric.

4. **OpenResearcher** focuses on trajectory synthesis for training data generation, not end-user research. It provides valuable insights: incorrect trajectories use nearly 2× more tool calls than successful ones, suggesting failure is due to inefficient search, not insufficient exploration.

5. **New entrants (2025-2026):** DeepVerifier (three-stage verification framework), NVIDIA AI-Q (enterprise-grade with citation verification), Temporal+Braintrust (durable execution via Temporal workflows), MiniMax Agent (built-in five-step Deep Research with Agent Teams), Kimi K2.6 (10,000+ word reports, 300 parallel sub-agents), Skywork (hierarchical multi-agent), REDSearcher/OpenSeeker/ASearcher (RL-trained search agents).

### 3.3 Common Weakness Across All Projects

Every open-source project we investigated shares the same weakness: **they produce a report, not a reusable knowledge asset**. None emit structured knowledge graphs, machine-readable citation databases, embeddings, contradiction matrices, or reasoning traces as first-class outputs. This is the gap Prometheus Research is designed to fill.

---

## 4. Platform Specifications & Portability Strategy

### 4.1 The Converging Standard: agentskills.io

The agent skills ecosystem has converged on a single open standard originally developed by Anthropic and released as the Agent Skills specification at [agentskills.io](https://agentskills.io). As of mid-2026, virtually every major platform supports this format.

**Core format:** A folder containing a `SKILL.md` file with YAML frontmatter + Markdown body. Progressive disclosure (metadata at startup → instructions on activation → scripts/references on demand).

**What makes a skill truly portable:**
```
Portable Skill = SKILL.md (instructions) + AGENTS.md (context) + MCP (tools)
```

### 4.2 Platform Directory Conventions

| Platform | Skill Directory | Plugin Format | Notes |
|----------|-----------------|---------------|-------|
| **Claude Code** | `~/.claude/skills/`, `.claude/skills/` | `.claude-plugin/plugin.json` | Deepest ecosystem; subagents, hooks, routines |
| **Codex CLI** | `~/.agents/skills/`, `.agents/skills/` | `.codex-plugin/plugin.json` + `agents/openai.yaml` | Enterprise plugin system (March 2026) |
| **OpenCode** | `.opencode/plugins/` (JS/TS) | `@opencode-ai/plugin` | Code-based plugins; does NOT natively read SKILL.md |
| **Cursor** | `.cursor/skills/` | `.cursor-plugin/plugin.json` | Also scans `.claude/skills/` and `.codex/skills/` |
| **Windsurf** | `.windsurf/skills/` | N/A | Project-only skills; no global library |
| **Kimi Code** | `.kimi-code/skills/`, `.agents/skills/`, `.skills/` | N/A | Does NOT read `.claude/skills/` |
| **Gemini/Antigravity** | `.gemini/skills/`, `.agents/skills/` | `gemini-extension.json` | `.agents/skills/` preferred for portability |
| **Roo Code** | `.roo/skills/`, `.agents/skills/` | `.roo-plugin/` | Mode-specific skills supported |
| **MiniMax/Mavis** | Desktop app | N/A | Hybrid skills + multi-agent teams |

### 4.3 Multi-Platform Plugin Manifest Strategy

Sophisticated projects now ship multiple plugin manifests wrapping the same core skill:

```
prometheus-deep-research/
├── skills/
│   └── deep-research/
│       └── SKILL.md          ← Universal skill core
├── .claude-plugin/
│   └── plugin.json             ← Claude Code manifest
├── .codex-plugin/
│   └── plugin.json             ← Codex manifest + agents/openai.yaml
├── .cursor-plugin/
│   └── plugin.json             ← Cursor manifest
├── .github/plugin/
│   └── plugin.json             ← Copilot manifest
├── .windsurf/skills/
│   └── deep-research/
│       └── SKILL.md            ← Windsurf project skill
├── .kimi-code/skills/
│   └── deep-research/
│       └── SKILL.md            ← Kimi Code skill
├── gemini-extension.json       ← Gemini/Antigravity
├── opencode-skills.tar.gz      ← OpenCode bridge (via SDK)
└── .mcp.json                   ← MCP server config (shared)
```

### 4.4 OpenCode Bridge Strategy

Since OpenCode does NOT natively load `SKILL.md` files, we provide two paths:
1. **SDK Bridge:** A small JS/TS plugin (`@prometheus-ags/opencode-deep-research`) that loads the SKILL.md instructions and exposes them as OpenCode hooks.
2. **MCP Path:** OpenCode supports MCP servers natively. The `prometheus-research` MCP server can be configured in `opencode.json` and invoked via tool calls.

---

## 5. UI Protocol Strategy: AG-UI + A2UI + MCP Apps

### 5.1 Protocol Stack

| Protocol | Layer | Maintainer | Purpose | Role in Research |
|----------|-------|------------|---------|------------------|
| **MCP** | Agent ↔ Tools | Anthropic | Tool discovery & invocation | Research tools (search, verify, graph) |
| **A2A** | Agent ↔ Agent | Google / LF | Task delegation | Sub-agent orchestration (future) |
| **AG-UI** | Agent ↔ Frontend | CopilotKit + partners | Real-time streaming events | Research progress, tool calls, state |
| **A2UI** | Generative UI | Google (Apache 2.0) | Declarative UI payloads | Research artifacts (charts, graphs, timelines) |

**Key insight:** AG-UI and A2UI are complementary, not competing.
- **AG-UI = Transport** (how agents and UIs communicate at runtime)
- **A2UI = Content** (what UI to render)
- **MCP = Tools** (what capabilities the agent can invoke)

### 5.2 AG-UI Protocol Deep Dive

AG-UI defines ~26 event types across 5 categories:
- **Lifecycle:** `RUN_STARTED`, `RUN_FINISHED`, `RUN_ERROR`, `STEP_STARTED`, `STEP_FINISHED`
- **Text Messages:** `TEXT_MESSAGE_START`, `TEXT_MESSAGE_CONTENT`, `TEXT_MESSAGE_END`, `THINKING_START/END`
- **Tool Calls:** `TOOL_CALL_START`, `TOOL_CALL_ARGS`, `TOOL_CALL_END`, `TOOL_CALL_RESULT`
- **State Management:** `STATE_SNAPSHOT`, `STATE_DELTA`, `MESSAGES_SNAPSHOT`, `ACTIVITY_SNAPSHOT/DELTA`
- **Special:** `RAW`, `CUSTOM` (for application-specific payloads)

**Transports:** SSE (primary), WebSockets, HTTP streaming.  
**SDKs:** TypeScript, Python, Rust, Go, Java, Kotlin, Dart, .NET, Nim.  
**Framework support:** LangGraph, CrewAI, Mastra, Pydantic AI, Microsoft Agent Framework, AG2 (first-party or community).

### 5.3 A2UI Protocol Deep Dive

A2UI is a declarative, generative UI specification allowing agents to "speak UI" as safe JSON data rather than executable code.

**Core message types:** `createSurface`, `updateComponents`, `updateDataModel`, `deleteSurface`  
**Client-to-server:** `action` (user interaction events)  
**Component catalog:** Text, Button, Image, Row, Column, Card, List, TextField, Modal, Tabs, Slider, DateTimeInput, Checkbox, MultipleChoice, and more.

**Design principles:**
1. LLM-friendly flat adjacency lists with ID references
2. Progressive rendering via JSONL streaming
3. Framework-agnostic (React, Angular, Flutter, Lit, native mobile)
4. Security-first: no executable code, only pre-approved components
5. Separation of UI structure, application data, and client rendering

**Relationship to AG-UI:** A2UI payloads can be carried inside AG-UI `CUSTOM` or `STATE_SNAPSHOT` events. The canonical pattern is: AG-UI streams events, and some events carry A2UI payloads for generative UI components.

### 5.4 MCP Apps (SEP-1865)

MCP Apps allow MCP servers to expose interactive UI via `ui://` URI resources, rendered in sandboxed iframes. This is different from AG-UI/A2UI:
- **MCP Apps:** Pre-built HTML templates, iframe sandboxing, tool-specific embedded interfaces
- **AG-UI/A2UI:** Streaming native components, real-time agent conversations

**Best for:** Complex tool UIs (charts, forms, dashboards)  
**Research use case:** Embedded citation browser, knowledge graph explorer, verification dashboard

### 5.5 Recommended Protocol Priority

| Protocol | Use Case | Priority |
|----------|----------|----------|
| **MCP** | Tool-based access (Claude, Cursor, IDE plugins) | P0 — essential |
| **AG-UI** | Streaming research progress to web/native clients | P0 — essential |
| **A2UI** | Generative research artifacts (charts, graphs, timelines) | P1 — high value |
| **A2A** | Research agent delegation to specialist sub-agents | P2 — future |
| **MCP Apps** | Embedded tool UIs within existing MCP hosts | P2 — future |

---

## 6. Knowledge Asset Architecture

### 6.1 The Shift from Reports to Knowledge Assets

| Dimension | Disposable Report | Knowledge Asset |
|-----------|-------------------|-----------------|
| **Format** | Markdown, PDF, DOCX | JSON package with graph, embeddings, traces |
| **Lifespan** | Single-session | Persistent, versioned, extensible |
| **Queryability** | Full-text search only | Graph traversal, semantic search, structured queries |
| **Citations** | Inline hyperlinks | Machine-readable `citations.json` with DOI, URL, confidence, access timestamp |
| **Extensibility** | Manual editing | Agent-mergable: other agents can append nodes, resolve conflicts, add evidence |
| **Verifiability** | Human-read only | Machine-checkable: source hashes, retrieval traces, confidence scores |
| **Reusability** | Copy-paste | Importable as MCP tool context or RAG corpus |

### 6.2 The `.research` Package Format

```
my-topic.research/
├── manifest.json              # Package metadata, schema version, provenance
├── report.md                  # Human-readable final report
├── report.pdf                 # Formatted document for distribution
├── citations.json             # Structured citation database
├── knowledge_graph.json       # Entity-relationship graph (JSON-LD compatible)
├── embeddings/                # Vector embeddings
│   ├── chunks-embeddings.npy
│   └── entity-embeddings.npy
├── entity_graph.json          # Canonical entity resolution
├── timeline.json              # Temporal events with confidence intervals
├── contradictions.json        # Detected conflicts with evidence for each side
├── confidence_scores.json     # Per-claim, per-source, per-entity confidence
├── follow_up_questions.json   # Open questions for future research
├── source_cache/              # Mirrored/raw source content with content hashes
│   ├── source-001/
│   │   ├── content.html
│   │   ├── content.hash
│   │   └── metadata.json
├── search_trace.json          # Every query issued, results retrieved, ranking
├── reasoning_trace.json       # LLM reasoning steps, tool calls, plan revisions
├── artifacts/                 # Generated exports: PDF, DOCX, PPTX, CSV
└── SKILL.md                   # How to use this research package as a skill context
```

**Properties:** Content-addressable (top-level hash), diffable (JSON components can be merged), self-describing (manifest + SKILL.md).

### 6.3 Machine-Readable Citation Schema

```json
{
  "version": "1.0.0",
  "schema": "prometheus-citation-v1",
  "citations": [
    {
      "id": "cite-001",
      "type": "web",
      "url": "https://example.com/article",
      "title": "Example Article",
      "authors": ["Alice Smith"],
      "published": "2025-06-15",
      "accessed": "2026-07-03T15:00:00Z",
      "retriever": "tavily-mcp",
      "source_hash": "sha256:abc123...",
      "confidence": 0.92,
      "claims_supported": ["claim-001", "claim-002"]
    }
  ]
}
```

### 6.4 Knowledge Graph Schema

```json
{
  "entities": [
    {"id": "ent-001", "type": "Organization", "name": "OpenAI", "canonical_id": "wikidata:Q217219"},
    {"id": "ent-002", "type": "Product", "name": "GPT-4", "canonical_id": null}
  ],
  "relations": [
    {"source": "ent-002", "relation": "DEVELOPED_BY", "target": "ent-001", "confidence": 0.98, "sources": ["cite-001"]}
  ],
  "provenance": "prometheus-research-v1"
}
```

### 6.5 Persistent Knowledge Object Tiers

| Tier | Object | Format | Query Interface | Lifetime |
|------|--------|--------|-----------------|----------|
| **L1** | Research Package | `.research.tar.zst` | File system / IPFS | Permanent (user-managed) |
| **L2** | Knowledge Graph Fragment | `knowledge_graph.json` + embeddings | MCP tool `research_query_graph` | Persistent (surreal-memory) |
| **L3** | Citation & Claim Database | `citations.json` + `verified_claims.json` | MCP tool `research_query_claims` | Persistent (surreal-memory) |

---

## 7. Recommended Architecture: The Prometheus Research Pipeline

### 7.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         UNIVERSAL DEEP RESEARCH SKILL                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  INPUT: Research query + optional documents + constraints                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 1: PLANNING                                                          │
│  ├── Planner: Decomposes query into sub-tasks                               │
│  ├── Question Decomposer: Generates atomic research questions                 │
│  └── Search Planner: Allocates search budget per question                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 2: EVIDENCE GATHERING                                                │
│  ├── Retriever (web/RAG/graph/MCP/API): Multi-source fetch                  │
│  ├── Evidence Collector: Extracts claims from sources                       │
│  └── Source Cache: Persists raw content for verification                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 3: VERIFICATION & SYNTHESIS                                        │
│  ├── Evidence Verifier: Checks claim-source alignment                       │
│  ├── Conflict Resolver: Detects and resolves contradictions                 │
│  ├── Knowledge Graph Builder: Entities → relations → graph                    │
│  └── Citation Manager: Links claims to verified sources                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 4: OUTPUT GENERATION                                                   │
│  ├── Report Generator: Markdown/PDF/DOCX with citations                       │
│  ├── Artifact Exporter: Knowledge package (JSON bundle)                       │
│  └── Knowledge Asset Publisher: Writes to surreal-memory                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 5: PROGRESS & UI (via AG-UI / A2UI)                                   │
│  ├── Status events: search, analysis, synthesis, complete                   │
│  ├── Artifact events: outline, sections, sources, graph                       │
│  └── Human-in-the-loop: approval, redirection, query add                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  OUTPUT: Report + Knowledge Package + Queryable KB Entry                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 10-Stage Pipeline Detail

#### Stage 1: Planner (Question Decomposer)
- **Input:** User query + optional constraints (depth, breadth, format, date range)
- **Process:** LLM generates an objective research plan with N sub-questions
- **Output:** `plan.json` — ordered sub-questions with priority, estimated sources, and success criteria
- **MCP Tools:** `sequential-thinking` (trace reasoning), `liter-llm` (route to appropriate model)

#### Stage 2: Search Planner
- **Input:** `plan.json`
- **Process:** For each sub-question, generate 3–5 targeted search queries; select retriever (web, graph, memory, API) based on question type
- **Output:** `search_plan.json` — queries mapped to retriever and priority
- **MCP Tools:** `tavily-mcp` (web), `prometheus-knowledge` (KB), `surreal-memory` (graph)

#### Stage 3: Retriever (Hybrid Multi-Source)
- **Input:** `search_plan.json`
- **Process:** Execute searches in parallel with concurrency limits; retrieve raw content; fetch full pages where needed
- **Output:** `raw_evidence/` — scraped content, API responses, graph subgraphs, with metadata
- **MCP Tools:** `tavily-mcp` (web), `prometheus-knowledge` (KB), `surreal-memory` (graph), custom API MCPs
- **Pattern:** MCP-Enhanced RAG — treat tool invocation as part of retrieval

#### Stage 4: Evidence Collector
- **Input:** `raw_evidence/`
- **Process:** Normalize formats (HTML → Markdown, JSON → flat); deduplicate by content hash; chunk into context windows; score relevance with embeddings + cross-encoder
- **Output:** `evidence_chunks.jsonl` — scored, chunked, deduplicated evidence
- **MCP Tools:** `liter-llm` (embedding model routing), `forge-rs` (enrichment)

#### Stage 5: Evidence Verifier
- **Input:** `evidence_chunks.jsonl` + `plan.json`
- **Process:** For each claim, check if evidence supports it (claim extraction → evidence matching → faithfulness scoring); if faithfulness < threshold, route back to retriever with refined query
- **Output:** `verified_claims.json` — claims with confidence scores and supporting evidence IDs
- **MCP Tools:** `sequential-thinking` (structured verification), `sycophancy-correction` (bias check)
- **Max Retries:** 2 loops before returning best attempt with confidence flag

#### Stage 6: Conflict Resolver
- **Input:** `verified_claims.json`
- **Process:** Detect contradictions (semantic similarity + logical negation detection); score evidence balance for each side; flag for human review if auto-resolution confidence < 0.8
- **Output:** `contradictions.json` + `resolved_claims.json`
- **MCP Tools:** `surreal-memory` (check prior knowledge), `forge-rs` (drift detection)

#### Stage 7: Knowledge Graph Builder
- **Input:** `resolved_claims.json` + `citations.json`
- **Process:** Extract entities (NER); resolve to canonical IDs (Wikidata, ORCID, company registries); infer relationships; build JSON-LD compatible graph
- **Output:** `knowledge_graph.json` + `entity_graph.json`
- **MCP Tools:** `surreal-memory` (persist graph), `prometheus-knowledge` (link to KB)

#### Stage 8: Citation Manager
- **Input:** `knowledge_graph.json` + `source_cache/`
- **Process:** Resolve DOIs; validate URLs (HEAD request, check for 404); format citations in requested style (APA, MLA, Chicago, IEEE, BibTeX); generate `citations.json`
- **Output:** `citations.json` + inline citation markers in `report.md`

#### Stage 9: Report Generator
- **Input:** All prior outputs
- **Process:** Synthesize into structured report: executive summary, methodology, findings per sub-question, contradictions/discussions, conclusion, references
- **Output:** `report.md` (primary) + `report.pdf`, `report.docx` (optional)
- **MCP Tools:** `liter-llm` (synthesis model)
- **Pattern:** Human-in-the-loop optional: pause before final write for user approval

#### Stage 10: Artifact Exporter
- **Input:** All research artifacts
- **Process:** Bundle into `.research` package; compress; compute content hash; optionally publish to IPFS or knowledge base
- **Output:** `my-topic.research.tar.zst` (content-addressable)
- **MCP Tools:** `prometheus-knowledge` (publish to KB), `surreal-memory` (index graph)

### 7.3 State Machine & Orchestration

The pipeline is implemented as a **state machine with checkpoints** after each stage. This enables:
- **Resumability:** If a stage fails, resume from the last checkpoint
- **Human-in-the-loop:** Pause at configurable gates (plan approval, conflict review)
- **Parallelism:** Stages 3–5 can run in parallel per sub-question
- **Observability:** Full trace via OpenTelemetry or LangSmith

---

## 8. Feature Specification

### 8.1 Core Features (P0)

| Feature | Description | Acceptance Criteria |
|---------|-------------|-------------------|
| **Multi-source retrieval** | Web (Tavily), RAG, graph DB, MCP APIs, local documents | Supports ≥3 source types simultaneously |
| **Question decomposition** | LLM-driven planner breaks queries into sub-questions | Produces 3–10 sub-questions with priorities |
| **Evidence verification** | Faithfulness scoring per claim with retry loops | ≥85% of claims verified with source alignment |
| **Contradiction detection** | Semantic + logical negation detection | Flags conflicts with confidence scores |
| **Knowledge graph generation** | Entity extraction, canonicalization, relation inference | Outputs JSON-LD compatible graph |
| **Structured citations** | Machine-readable citation database with validation | Supports APA, MLA, Chicago, IEEE, BibTeX |
| **Report generation** | Markdown synthesis with inline citations | Produces executive summary + findings per sub-question |
| **Knowledge package export** | `.research` bundle with all artifacts | Contains manifest + report + graph + citations + traces + cache |
| **MCP server** | stdio + SSE transports exposing research tools | Claude, Cursor, Codex, Windsurf can invoke tools |
| **AG-UI streaming** | Real-time progress events during research | Emits RUN_STARTED, STEP_STARTED, TOOL_CALL, STATE_SNAPSHOT, RUN_FINISHED |

### 8.2 Advanced Features (P1)

| Feature | Description |
|---------|-------------|
| **A2UI artifact rendering** | Generative UI components (CitationCard, SourceList, Timeline, EntityGraph, ConfidenceMeter) streamed to frontend |
| **Human-in-the-loop gates** | Pause at plan approval, conflict review, and final report approval |
| **Knowledge drift detection** | Periodic re-verification of claims against live sources; stale citation flagging |
| **Entity resolution service** | Wikidata, ORCID, Crunchbase APIs for canonical entity IDs |
| **Collaborative research** | Multiple agents can append to the same `.research` package with merge conflict resolution |
| **Visual research explorer** | React + assistant-ui web app for browsing `.research` packages as interactive knowledge graphs |
| **MCP App UI** | Embedded citation browser and knowledge graph explorer via `ui://` resources |
| **Cross-research citation** | Research packages can cite other packages (inter-package knowledge graph) |

### 8.3 Future Features (P2)

| Feature | Description |
|---------|-------------|
| **A2A sub-agent delegation** | Research agent delegates to specialist sub-agents (e.g., legal researcher, medical researcher) |
| **Federated research network** | Distributed research packages with reputation staking and community verification |
| **Autonomous research agent** | Self-directed research that schedules follow-up investigations based on drift detection |
| **Multimodal research** | Vision-enabled research (analyze images, charts, diagrams in source documents) |
| **Research DAO** | Community-verified research packages with on-chain provenance |

---

## 9. MCP Server Design

### 9.1 Tiered Tool Interface

**High-Level (Consumer):**
- `research_conduct(query, depth, breadth, output_formats)` → returns `research_id`
- `research_get_status(research_id)` → progress, current phase, estimated completion
- `research_get_results(research_id, format="package")` → returns `.research` package or `report.md`

**Low-Level (Builder/Advanced):**
- `research_plan(query)` → returns `plan.json` with sub-questions
- `research_search(query, sources=["web", "graph", "memory"])` → ranked evidence
- `research_verify(claim, evidence)` → confidence score + verification notes
- `research_resolve_conflict(claim_a, claim_b)` → resolution or escalation flag
- `research_build_graph(evidence)` → `knowledge_graph.json` fragment
- `research_export(research_id, format="citations.json")` → specific package component

### 9.2 Stateful Sessions

Use `session_id` to track multi-step research:
- Research state persists across tool calls
- Partial results can be queried and extended
- Supports long-running research with checkpoints

### 9.3 Resource Exposure

Expose intermediate artifacts as MCP resources:
- `research://{session_id}/outline`
- `research://{session_id}/sources`
- `research://{session_id}/evidence_graph`
- `research://{session_id}/report.md`

### 9.4 Rust/Axum Implementation

```rust
// Axum router with dual transports
let app = Router::new()
    .route("/mcp", post(mcp_handler))          // MCP JSON-RPC over SSE
    .route("/agui", get(agui_stream))          // AG-UI SSE event stream
    .route("/a2ui", get(a2ui_stream))          // A2UI SSE surface stream
    .route("/api/v1/research", post(api_research))
    .route("/api/v1/research/:id", get(api_status))
    .layer(SharedState::new(research_engine));
```

**Crates:** `rmcp` (Rust MCP SDK), `axum`, `tokio`, `tower-http`, `serde`, `eventsource`.

---

## 10. AG-UI / A2UI Research App Design

### 10.1 Multi-Protocol Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                        Research Copilot UI                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │ Chat / Query    │  │ Research Status │  │ Knowledge Graph     │  │
│  │ Input           │  │ Panel           │  │ (A2UI Component)    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │ Research Report (streaming markdown + A2UI artifact panels)      │  │
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

### 10.2 A2UI Component Catalog for Research

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

### 10.3 Frontend Strategy

| Client | Technology | Use Case |
|--------|------------|----------|
| **Web UI** | React + assistant-ui + `@assistant-ui/react-ag-ui` | Primary research interface |
| **Desktop** | Tauri (Rust) + React + AG-UI client | Cross-platform native app |
| **IDE** | MCP tools only | Claude, Cursor, Windsurf, Roo |
| **CLI** | Rust binary (`prometheus-research`) | Scripting, automation |

### 10.4 Integration with `prometheus-entity-management`

The `@prometheus-ags/prometheus-entity-management` npm module provides a Zustand-based normalized entity graph store. The research UI should:
- Use it as the client-side reactive data layer for research entities (sources, citations, claims, people, organizations, topics)
- Sync with the backend via AG-UI `STATE_SNAPSHOT` / `STATE_DELTA` events
- Normalize entities extracted during research so they can be queried and browsed across the UI

### 10.5 Integration with `flint-platform-agent`

The `flint-platform-agent` serves as the runtime harness for executing the deep research skill. It manages:
- Agent lifecycle and tool registration
- Event streaming and protocol routing
- Multi-agent coordination (for future A2A delegation)

---

## 11. Native CLI: `prometheus-research`

### 11.1 Subcommands

```bash
# Conduct research
prometheus-research conduct "quantum computing error correction 2025" \
  --depth 3 --breadth 4 --output ./quantum.research

# Query existing research
prometheus-research query ./quantum.research \
  --question "What are the latest surface codes?"

# Extend existing research with new evidence
prometheus-research extend ./quantum.research \
  --query "logical qubit experiments 2026"

# Export to different formats
prometheus-research export ./quantum.research \
  --format pdf --output ./quantum-report.pdf
prometheus-research export ./quantum.research \
  --format docx --output ./quantum-report.docx

# Verify claims against live sources
prometheus-research verify ./quantum.research \
  --claims claim-001,claim-002

# Check for knowledge drift (re-run vs. stored)
prometheus-research drift ./quantum.research \
  --threshold 0.8

# Run as MCP server (SSE + stdio)
prometheus-research mcp --port 8944 --transport sse
prometheus-research mcp --transport stdio

# Show real-time research progress (AG-UI stream)
prometheus-research watch --research-id research-abc123

# List research packages in knowledge base
prometheus-research list --kb ~/.prometheus/knowledge/

# Fork research into new investigation
prometheus-research fork ./quantum.research \
  --query "superconducting qubits specifically"

# Interactive TUI with progress bars
prometheus-research interactive --topic "..."
```

### 11.2 Configuration

```yaml
# ~/.prometheus/research.yaml
default_depth: 3
default_breadth: 4
model:
  planner: gpt-4o-mini
  synthesizer: claude-sonnet-4
  verifier: gpt-4o
retrievers:
  - tavily
  - surreal-memory
  - prometheus-knowledge
output_formats:
  - markdown
  - pdf
  - docx
mcp_servers:
  - tavily-mcp
  - sequential-thinking
  - liter-llm
  - surreal-memory
  - forge-rs
  - prometheus-knowledge
  - sycophancy-correction
```

### 11.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Verification failed |
| 3 | Contradiction detected |
| 4 | Network/search failure |

---

## 12. Positioning & Competitive Differentiation

### 12.1 Competitive Matrix

| Capability | GPT Researcher | LangGraph ODR | MiroThinker | **Prometheus Research** |
|------------|---------------|---------------|-------------|------------------------|
| Maturity | ★★★★★ | ★★★★☆ | ★★★★☆ | ★★★☆☆ (new) |
| Configurability | ★★★★☆ | ★★★★★ | ★★★☆☆ | ★★★★★ |
| Benchmark Perf | ★★★☆☆ | ★★★★☆ | ★★★★★ | ★★★★☆ (target) |
| MCP Native | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ★★★★★ |
| Model Agnostic | ★★★★★ | ★★★★★ | ★★★☆☆ | ★★★★★ |
| Knowledge Assets | ★★★☆☆ | ★★★★☆ | ★★★☆☆ | ★★★★★ |
| AG-UI / A2UI | ★☆☆☆☆ | ★★☆☆☆ | ★☆☆☆☆ | ★★★★★ |
| Multi-Platform Skill | ★★★☆☆ | ★★☆☆☆ | ★☆☆☆☆ | ★★★★★ |
| Verification Pipeline | ★★★☆☆ | ★★★★☆ | ★★★★★ | ★★★★★ |
| Open Source | ★★★★★ | ★★★★★ | ★★★★☆ | ★★★★★ |

### 12.2 Unique Value Propositions

1. **Knowledge Infrastructure, Not Just Reports** — The only deep-research system that natively emits structured, queryable, extensible knowledge packages.
2. **Harness Agnostic** — Works as a portable skill across 10+ platforms (Claude, Codex, OpenCode, Cursor, Windsurf, Kimi, Roo, Amp, Gemini, MiniMax) AND as a standalone MCP server + web app.
3. **Full-Stack Verification** — Multi-layer verification (faithfulness scoring, contradiction detection, sycophancy correction, knowledge drift) rather than single-pass generation.
4. **Real-Time Streaming UI** — AG-UI + A2UI progress visibility for long-running research (minutes to hours), with human-in-the-loop gates.
5. **Integrated Prometheus Ecosystem** — Native integration with surreal-memory, forge-rs, liter-llm, prometheus-knowledge, and sycophancy-correction — not bolted-on.
6. **Persistent Knowledge Graph** — Research findings become part of a growing, queryable knowledge base that other agents can cite and extend.

### 12.3 Target Users

- **AI-native researchers** who need more than a static report
- **Knowledge workers** building institutional knowledge bases
- **Agent developers** who need verifiable, citable research outputs for downstream reasoning
- **Multi-platform teams** who want one research skill that works across Claude, Cursor, Codex, and Kimi

---

## 13. Integration with Existing Prometheus Stack

### 13.1 MCP Integration Map

| Component | Port (SSE) | Role in Research Workflow |
|-----------|------------|---------------------------|
| **tavily-mcp** | stdio | Primary real-time web search and retrieval |
| **sequential-thinking** | stdio | Structured reasoning traces for planning and verification |
| **liter-llm** | stdio | Model routing — cheap models for summarization, powerful models for synthesis |
| **surreal-memory-server** | 23001 | Persistent knowledge graph storage and entity retrieval |
| **prometheus-knowledge** | 8942 | Karpathy KB queries — prevent redundant research |
| **forge-rs** | 8943 | Post-research enrichment, reflection, drift detection |
| **sycophancy-correction** | stdio | Bias detection and confirmation-bias correction |

### 13.2 Integration Patterns

**Tavily-MCP:** Use `tavily_search` for primary web retrieval; configure hybrid web + MCP research; cache raw results with content hashes.

**Sequential-Thinking:** Initialize a reasoning trace before each planning phase; log planner decisions; audit retriever compliance with plan.

**Surreal-Memory:** After each session, save `knowledge_graph.json` and `entity_graph.json`; future research queries `search_memory` to find prior related work.

**Prometheus-Knowledge:** Check KB for existing knowledge before starting new research; append findings rather than replacing; link new findings to prior knowledge.

**Forge-RS:** After report generation, run `forge reflect` for gaps/contradictions; `forge enrich` for additional entity extraction; `forge drift` to compare against existing KB.

**Sycophancy-Correction:** Before finalizing, run findings through sycophancy checker; flag over-validation of user assumptions; generate "devil's advocate" section with counter-evidence.

### 13.3 Skill Pack Integration

The deep-research skill should be integrated into the Prometheus Skill Pack as:
- A new top-level skill under `skills/process/deep-research/` (or `skills/research/deep-research/`)
- A new native binary `prometheus-research` built from a Rust workspace under `tools/prometheus-research/`
- A new React frontend under `site/research-ui/` or `tools/prometheus-research/frontend/`
- Plugin manifests for `.claude-plugin/`, `.codex-plugin/`, `.cursor-plugin/`, etc.

---

## 14. Project Plan & Roadmap

### 14.1 Phase 1: MVP (Weeks 1–6)

**Goal:** A working `prometheus-research` CLI + MCP server + portable SKILL.md

| Week | Deliverable | Owner |
|------|-------------|-------|
| 1 | Define `.research` package schema (JSON Schema) + `manifest.json` format | Architecture |
| 2 | Implement Rust core: Planner + Search Planner + Tavily retriever | Backend |
| 3 | Implement Evidence Collector + Verifier + Conflict Resolver | Backend |
| 4 | Implement Knowledge Graph Builder + Citation Manager + Report Generator | Backend |
| 5 | Implement MCP server (stdio + SSE) with high-level + low-level tools | Backend |
| 6 | Write `SKILL.md` + multi-platform plugin manifests + install scripts | Integration |

**MVP Acceptance Criteria:**
- Can run `prometheus-research conduct "topic" --output ./topic.research` end-to-end
- MCP server exposes `research_conduct`, `research_get_status`, `research_get_results`
- SKILL.md installs successfully on Claude Code, Codex, Cursor, Windsurf
- Output contains `report.md`, `citations.json`, `knowledge_graph.json`, `search_trace.json`

### 14.2 Phase 2: V2 — Streaming UI & Verification (Weeks 7–12)

| Week | Deliverable |
|------|-------------|
| 7–8 | Implement AG-UI SSE endpoint with full event stream |
| 9–10 | Build React + assistant-ui research frontend with progress visualization |
| 11 | Implement A2UI surface generator for research artifacts |
| 12 | Add knowledge drift detection (`prometheus-research drift`) + forge-rs integration |

**V2 Acceptance Criteria:**
- Web UI shows real-time research progress with phase indicators, source lists, and confidence meters
- A2UI renders CitationCards, SourceLists, and EntityGraphs during research
- `prometheus-research drift` detects and reports knowledge drift vs. live sources

### 14.3 Phase 3: V3 — Advanced Features (Weeks 13–20)

| Week | Deliverable |
|------|-------------|
| 13–14 | Entity resolution service (Wikidata, ORCID, Crunchbase APIs) |
| 15–16 | Collaborative research — multiple agents append to same package with merge resolution |
| 17–18 | MCP App UI (`ui://` resources) for embedded citation browser and graph explorer |
| 19–20 | Visual research explorer (interactive knowledge graph browser) + performance optimization |

**V3 Acceptance Criteria:**
- Entities resolved to canonical Wikidata/ORCID IDs across research sessions
- Multiple research runs can merge into a single evolving knowledge graph
- MCP App renders embedded knowledge graph explorer in Claude Desktop / Cursor

### 14.4 Phase 4: V4 — Federation & Autonomy (Future)

- **A2A sub-agent delegation:** Research agent delegates to specialist sub-agents
- **Federated research network:** Research packages cite other packages; inter-package knowledge graph
- **Autonomous research agent:** Self-directed scheduling of follow-up investigations
- **Research DAO:** Community-verified research with reputation staking

### 14.5 Resource Requirements

| Phase | Engineering (FTE) | Duration | Key Dependencies |
|-------|-------------------|----------|----------------|
| MVP | 1.0 | 6 weeks | Tavily API key, surreal-memory running, liter-llm configured |
| V2 | 1.0 | 6 weeks | React + TypeScript frontend, assistant-ui, AG-UI SDK |
| V3 | 1.0–1.5 | 8 weeks | Entity resolution APIs, MCP App sandbox testing |
| V4 | 2.0 | TBD | A2A protocol maturity, federated infrastructure |

---

## 15. Conclusion & Next Steps

### 15.1 Summary

This specification defines a world-class deep-research skill for the Prometheus ecosystem that:
1. **Produces knowledge assets, not just reports** — structured, queryable, extensible packages
2. **Works everywhere** — portable SKILL.md + MCP server + AG-UI/A2UI streaming
3. **Verifies everything** — multi-layer verification, contradiction detection, bias correction
4. **Integrates deeply** — native use of the entire Prometheus MCP stack
5. **Streams progress** — real-time AG-UI progress with generative A2UI artifacts

### 15.2 Immediate Next Steps

1. **Review this specification** with the Prometheus core team and community
2. **Approve the `.research` package schema** and JSON Schema definitions
3. **Scaffold the Rust project** using the existing `native-agent` skill template
4. **Set up the `tools/prometheus-research/` workspace** with Axum + MCP + AG-UI endpoints
5. **Implement the MVP Planner + Retriever** against Tavily MCP
6. **Draft the `SKILL.md`** and test installation across Claude Code, Codex, and Cursor
7. **Create a feature branch** (`feat/deep-research`) in the `prometheus-skill-pack` repo

### 15.3 Open Questions

1. Should the initial retriever support only Tavily, or also Brave, SerpAPI, and native search from day one?
2. Should the knowledge graph use SurrealDB's native graph features, or a separate Neo4j instance?
3. Should the React frontend be bundled into the MCP server (single binary), or deployed separately?
4. What is the exact API surface of `prometheus-entity-management` and `flint-platform-agent` for integration?
5. Should the skill support "research resume" across process restarts (via surreal-memory session persistence)?

---

## 16. References

### Deep Research Agents

1. GPT Researcher — [github.com/assafelovic/gpt-researcher](https://github.com/assafelovic/gpt-researcher) (27.9k stars, v3.5.0)
2. LangGraph Open Deep Research — [github.com/langchain-ai/open_deep_research](https://github.com/langchain-ai/open_deep_research)
3. OpenResearcher — [github.com/TIGER-AI-Lab/OpenResearcher](https://github.com/TIGER-AI-Lab/OpenResearcher)
4. MiroThinker — [github.com/MiroMindAI/MiroThinker](https://github.com/MiroMindAI/MiroThinker)
5. HuggingFace Open Deep Research (smolagents) — [huggingface.co/smolagents](https://huggingface.co/smolagents)
6. DeepVerifier — arXiv:2601.15808
7. NVIDIA AI-Q Deep Researcher — [docs.nvidia.com/aiq-blueprint](https://docs.nvidia.com/aiq-blueprint)
8. Temporal + Braintrust Deep Research — [braintrust.dev/cookbook](https://braintrust.dev/cookbook)
9. MiniMax Agent — [minimax-ai.chat](https://minimax-ai.chat)
10. Kimi K2.6 Agent — [kimi.com/help/agent](https://kimi.com/help/agent)

### Skill Platforms

11. Agent Skills Specification — [agentskills.io/specification](https://agentskills.io/specification)
12. Claude Code Docs — [docs.anthropic.com/claude-code](https://docs.anthropic.com/claude-code)
13. OpenAI Codex Plugin Docs — [developers.openai.com/codex/plugins](https://developers.openai.com/codex/plugins)
14. OpenCode Plugin Docs — [opencode.ai/docs/plugins](https://opencode.ai/docs/plugins)
15. Cursor Skills — [agensi.io/learn/cursor-rules-vs-skill-md](https://agensi.io/learn/cursor-rules-vs-skill-md)
16. Windsurf Cascade — [byteiota.com/windsurf-cascade-tutorial](https://byteiota.com/windsurf-cascade-tutorial)
17. Kimi Code Skills — [agensi.io/learn/kimi-code-skills-guide](https://agensi.io/learn/kimi-code-skills-guide)
18. Roo Code Skills — [roocodeinc.github.io/Roo-Code](https://roocodeinc.github.io/Roo-Code)

### UI Protocols

19. AG-UI Protocol — [docs.ag-ui.com](https://docs.ag-ui.com)
20. AG-UI GitHub — [github.com/ag-ui-protocol/ag-ui](https://github.com/ag-ui-protocol/ag-ui)
21. A2UI Official — [a2ui.org](https://a2ui.org)
22. A2UI GitHub (Google) — [github.com/google/A2UI](https://github.com/google/A2UI)
23. MCP Apps Extension (SEP-1865) — [blog.modelcontextprotocol.io](https://blog.modelcontextprotocol.io)
24. assistant-ui — [assistant-ui.com](https://assistant-ui.com)
25. CopilotKit AG-UI — [docs.copilotkit.ai](https://docs.copilotkit.ai)

### Architecture & Knowledge Graphs

26. Tetrate — MCP + RAG — [tetrate.io/learn/ai/mcp](https://tetrate.io/learn/ai/mcp)
27. AgentForge — [github.com/omkarbhad/agentforge](https://github.com/omkarbhad/agentforge)
28. CASCADE — arXiv:2512.23880v1
29. GraphAware — LLMs for Knowledge Graph — [graphaware.com/blog](https://graphaware.com/blog)
30. Neo4j — Context Engineering — [neo4j.com/blog/agentic-ai](https://neo4j.com/blog/agentic-ai)
31. ZBrain — Knowledge Graphs for Agentic AI — [zbrain.ai](https://zbrain.ai)
32. LongTracer — [github.com/ENDEVSOLS/LongTracer](https://github.com/ENDEVSOLS/LongTracer)
33. ai-memory MCP — [mcpservers.org/servers/alphaonedev/ai-memory-mcp](https://mcpservers.org)
34. Prometheus Skill Pack — [github.com/Prometheus-AGS/prometheus-skill-system](https://github.com/Prometheus-AGS/prometheus-skill-system)
35. Prometheus Entity Management — [npmjs.com/package/@prometheus-ags/prometheus-entity-management](https://npmjs.com/package/@prometheus-ags/prometheus-entity-management)

---

## 17. Feynman Learning Integration

The Prometheus deep-research skill is not an isolated tool — it is a **learning primitive** deeply integrated with the Feynman Learning Loop ecosystem. This section defines how research and learning are bidirectional, mutually reinforcing capabilities.

### 17.1 Research as a Learning Primitive

Inquiry-Based Learning (IBL) frameworks explicitly treat research as constructivist learning. The ACRL Framework for Information Literacy frames "Research as Inquiry" as one of its six core frames — identifying knowledge practices including formulating questions, determining scope, breaking complex questions into simple ones, monitoring gathered information for gaps, synthesizing ideas from multiple sources, and drawing reasonable conclusions. These are **exactly the stages of the Deep Research pipeline**.

**Integration:** The Deep Research pipeline should be exposed as a `learn-research` skill that is itself a learnable competency tracked by the learner model. Research methodology (question formulation, source evaluation, synthesis, citation) becomes a concept DAG that learners can master through the Feynman loop.

### 17.2 Research Findings → Curriculum Inputs

The `learn-plan` skill already builds concept DAGs in surreal-memory. The research pipeline can auto-generate these DAGs from its findings:

| Research Stage | Curriculum Output |
|---|---|
| Search → Retrieve | Raw concept list from sources |
| Collect → Verify | Filtered, validated concept set |
| Resolve → Graph | **Concept DAG with prerequisite edges** |
| Cite → Report | Annotated curriculum with source provenance |
| Export | `curriculum.json` + `concept-DAG.json` |

**Mechanism:** The Knowledge Graph Builder (Stage 7) extracts entities and infers prerequisite relations. These become `create_entity` and `create_relation` calls in surreal-memory, feeding directly into `learn-plan`'s topological sort. Research depth (number of sources, complexity of synthesis) correlates with concept difficulty — which `learn-plan` uses for time estimation (simple definition: 15–30 min; applied concept: 45–90 min; complex system: 2–4 hours).

### 17.3 Feynman Loop as Research Quality Gate

The Feynman Technique (explain → identify gaps → fill gaps → simplify) is a specific application of self-explanation and retrieval practice — two of the most evidence-backed learning strategies, each with a meta-analytic effect size of g = 0.55.

**Integration:** The Feynman loop should function as a **quality gate between the Report and Export stages** of the research pipeline:

```
Report → Feynman Explain → Grade → Gap? → Re-research (back to Search)
                    ↓ No gaps
              Export → .research package
```

When applied to research findings, the Feynman loop identifies:
- **Source gaps**: Cannot explain which source supports a claim → re-verify citation
- **Methodology gaps**: Cannot explain how a conclusion was reached → re-read methods
- **Synthesis gaps**: Cannot explain how two sources connect → build cross-reference
- **Contradiction gaps**: Cannot reconcile conflicting sources → flag for resolution
- **Recency gaps**: Cannot explain if findings are still current → check dates

### 17.4 `.research` Package → Karpathy Wiki

Both the `.research` package and the Karpathy LLM Wiki are **compounding artifacts** — cross-references already exist, contradictions already flagged, synthesis already reflects everything read. The integration is straightforward:

| `.research` Component | Wiki Destination | Action |
|---|---|---|
| `report.md` | `wiki/topics/<topic>.md` | Filed as a topic summary page |
| `knowledge_graph.json` | `wiki/entities/*.md` | Entity pages with cross-links |
| `citations.json` | `wiki/sources/*.md` | Source pages with metadata |
| `contradictions.json` | `wiki/contradictions/*.md` | Contradiction pages with resolution notes |
| `timeline.json` | `wiki/timeline.md` | Chronological event page |
| `search_trace.json` | `log.md` | Append to chronological log |
| `reasoning_trace.json` | `wiki/methodology.md` | Research methodology notes |

**Wiki maintenance:** After each research session, the `forge reflect` + `pk ingest` Karpathy Loop writes session lessons back to the knowledge base. The wiki index (`index.md`) is updated to include new pages, and cross-references are maintained automatically.

### 17.5 `learn-kb` Using Deep Research Outputs

The `learn-kb` skill manages knowledge bases (Dify, surreal-memory palace, local files, URLs). Deep research outputs should be first-class KB sources:

```
/learn-kb add --type palace --name deep-research-output \
  --palace-id kb-research-output \
  --content-dir ./my-topic.research
```

**Structured ingestion layers:**
- **Raw layer**: `source_cache/` → ingested as raw documents
- **Retrieval layer**: `report.md` → indexed for semantic search
- **Graph layer**: `knowledge_graph.json` → loaded as entity-relationship graph
- **Epistemic layer**: `contradictions.json` → loaded as contradiction events with confidence scoring

### 17.6 `learn-survey` Using Research Findings

Performance-based diagnostics (mini-research tasks) are richer than quizzes. `learn-survey` can use research tasks to assess prior knowledge:
- Ask the learner to research a sub-topic in 5 minutes
- Evaluate their question formulation, source selection, and synthesis
- Use the result to set the recursion floor and mastery priors

### 17.7 `learn-plan` Using Research Scope

`learn-plan` should consume annotated concept DAGs from research outputs. DAG depth, betweenness centrality, and IRT difficulty estimates predict curriculum complexity and time estimates. Concepts with high betweenness (bridging separate sub-graphs) are curriculum bottlenecks and should be scheduled with extra time.

### 17.8 Recursion and Escalation in Research

The Feynman loop's patterns map directly to research strategies:

| Feynman Pattern | Research Strategy | Implementation |
|---|---|---|
| **Vertical recursion** (gap → child loop) | **Depth-first research** (drill into sub-topic) | Sub-question spawns child research thread |
| **Horizontal escalation** (novice → peer → skeptic) | **Breadth-first research** (same topic, multiple angles) | Same topic researched from multiple source types |
| **Recursion floor** | **Depth limit** (don't recurse beyond N levels) | Max research depth = 3 |
| **Mastery closure** (score ≥ 0.7) | **Confidence threshold** (claim confidence ≥ 0.85) | Verification gate before export |

### 17.9 Bidirectional Learning-Research Cycle

The optimal cycle for human and agent learning:

```
learn → research → explain → gap → re-research → curriculum → plan → practice → retain → certify → wiki
```

1. **Learn**: Initial exposure via `learn-survey` or `feynman-loop`
2. **Research**: Deep research on the topic to build comprehensive understanding
3. **Explain**: Feynman loop explains findings to novice/peer/skeptic
4. **Gap**: Identify knowledge gaps in the explanation
5. **Re-research**: Targeted research on gap topics
6. **Curriculum**: Auto-generated from research concept DAG
7. **Plan**: `learn-plan` sequences prerequisites
8. **Practice**: `learn-practice` with transfer problems
9. **Retain**: `learn-retain` schedules spaced repetition
10. **Certify**: `learn-certify` validates mastery
11. **Wiki**: Findings published to Karpathy wiki for compounding

---

## 18. Google OKF Alignment & Karpathy Wiki Convergence

### 18.1 The Discovery: Google's Open Knowledge Format (OKF) v0.1

On **June 12, 2026**, Google Cloud published the **Open Knowledge Format (OKF) v0.1** — an open, vendor-neutral specification for representing curated knowledge as a directory of **Markdown files with YAML frontmatter**. It formalizes the Karpathy LLM Wiki pattern into an interoperable standard.

> *"OKF is an open, human- and agent-friendly format for representing knowledge — the metadata, context, and curated insight that surrounds data and systems. It is designed to be authored by people, generated by agents, exchanged across organizations, and consumed by both."*
> — OKF v0.1 Specification, Google Cloud

**Core principles:**
1. **Minimally Opinionated** — Only one required field: `type`. Everything else is optional.
2. **Producer/Consumer Independence** — Human-authored bundles can be consumed by AI agents; agent-generated bundles can be queried by other agents.
3. **Format, Not Platform** — Not tied to any cloud, database, model provider, or agent framework.

### 18.2 OKF Bundle Structure

```
path/to/bundle/
├── index.md              # Directory listing for progressive disclosure
├── log.md                # Chronological history of updates
├── <concept>.md           # A concept at the bundle root
└── <subdirectory>/
    ├── index.md
    ├── <concept>.md
    └── <subdirectory>/
        └── …
```

**Concept document frontmatter (only `type` is required):**
```yaml
---
type: <Type name>                  # REQUIRED
title: <Optional display name>
description: <Optional one-line summary>
resource: <Optional canonical URI for the underlying asset>
tags: [<tag>, <tag>, …]           # Optional
timestamp: <ISO 8601 datetime>    # Optional last-modified time
# … other producer-defined key/value pairs
---
```

### 18.3 Convergence with Karpathy LLM Wiki

OKF explicitly acknowledges its lineage from Karpathy's LLM Wiki pattern. The `index.md` and `log.md` patterns that Prometheus already uses are exactly what OKF has formalized.

| Karpathy Wiki Pattern | OKF Standard | Prometheus Implementation |
|---|---|---|
| Raw sources (immutable) | `resource` URI in frontmatter | `source_cache/` directory |
| Wiki (LLM-generated markdown) | OKF concept `.md` files | `wiki/` directory |
| Schema (`CLAUDE.md` / `AGENTS.md`) | OKF bundle conventions | `.research/SKILL.md` |
| `index.md` (content catalog) | OKF `index.md` | `index.md` in `.research` |
| `log.md` (chronological) | OKF `log.md` | `log.md` in `.research` |
| Cross-references (markdown links) | OKF cross-linking | `knowledge_graph.json` + wiki links |

### 18.4 Aligning `.research` Package with OKF

The Prometheus `.research` package should adopt OKF as its base format while preserving research-specific extensions:

```
my-topic.research/          ← OKF bundle root
├── index.md                ← OKF directory listing (concepts, sources, reports)
├── log.md                  ← OKF chronological log (research sessions, updates)
├── okf_version: "0.1"      ← Bundle version in index.md frontmatter
│
├── concepts/               ← OKF concept pages (auto-generated from knowledge graph)
│   ├── agent-protocols.md
│   ├── mcp-standard.md
│   └── ag-ui-protocol.md
│
├── sources/                ← OKF concept pages for source documents
│   ├── source-001.md
│   └── source-002.md
│
├── report.md              ← Human-readable final report (OKF concept page)
├── report.pdf             ← Formatted export
│
├── manifest.json          ← Prometheus research metadata (schema version, provenance)
├── citations.json         ← Structured citation database
├── knowledge_graph.json   ← Entity-relationship graph (feeds OKF concept pages)
├── embeddings/            ← Vector embeddings
├── entity_graph.json      ← Canonical entity resolution
├── timeline.json          ← Temporal events
├── contradictions.json    ← Detected conflicts
├── confidence_scores.json ← Per-claim confidence
├── follow_up_questions.json ← Suggested next research
├── source_cache/          ← Mirrored/raw source content
├── search_trace.json      ← Search query history
├── reasoning_trace.json   ← Reasoning steps
├── artifacts/             ← Generated exports
│
└── SKILL.md               ← How to use this research package as a skill context
```

**Prometheus-specific OKF extensions:**

| Extension Field | Purpose | Location |
|---|---|---|
| `research_id` | UUID for this research run | `manifest.json` + concept frontmatter |
| `confidence` | 0.0–1.0 confidence score | Concept frontmatter (`confidence: 0.92`) |
| `verification_status` | `verified` / `unverified` / `contradicted` | Concept frontmatter |
| `research_stage` | `planning` / `gathering` / `synthesis` / `complete` | Concept frontmatter |
| `source_hash` | SHA-256 of source content at retrieval | `source_cache/` metadata |
| `retriever` | Which MCP server retrieved this (`tavily-mcp`, etc.) | `citations.json` |
| `claims_supported` | Array of claim IDs this source supports | `citations.json` |
| `thread_provenance` | Which research thread discovered this | `citations.json` |

### 18.5 AI-Native Document Format Landscape

The ecosystem has converged into a layered stack:

| Layer | Standard | Purpose | Format |
|-------|----------|---------|--------|
| Discovery | `llms.txt` | Point AI to important content | Markdown |
| Instructions | `AGENTS.md` / `CLAUDE.md` | Tell coding agents how to behave | Markdown |
| **Knowledge** | **OKF** | **Package curated knowledge for agents** | **Markdown + YAML** |
| Tool Access | MCP | Connect agents to tools and data | JSON-RPC |
| Agent Communication | A2A | Cross-organizational agent messaging | JSON |
| **UI Rendering** | **A2UI** | **Generate dynamic user interfaces** | **JSON** |
| Transport | AG-UI | Real-time agent-frontend sync | SSE / HTTP |

**OKF and A2UI are complementary, not competing:** OKF packages knowledge (what the agent knows); A2UI packages user interfaces (how to show it). In a full Prometheus research app, the agent uses OKF bundles as its knowledge base and generates A2UI payloads for the UI.

### 18.6 OKF Enrichment via Deep Research

Google ships an **Enrichment Agent** with OKF that walks a dataset and drafts concept documents. The Prometheus deep-research pipeline is a natural enrichment agent:

1. **Input**: Research query + optional documents
2. **Process**: Deep Research 10-stage pipeline
3. **Output**: OKF bundle with enriched concept pages, cross-links, citations, and index

The enrichment pattern:
```
Research Query → Deep Research Pipeline → OKF Concept Pages → Wiki Integration
                                      ↓
                              surreal-memory persistence
                              (entity graph + vector search)
```

### 18.7 Research-to-Wiki Compiler

A dedicated `research-to-wiki` compiler converts `.research` packages into OKF-compliant wiki bundles:

```bash
prometheus-research compile-wiki ./my-topic.research \
  --output ./wiki/my-topic/ \
  --format okf
```

**Compiler stages:**
1. Parse `knowledge_graph.json` → generate OKF concept pages for each entity
2. Parse `citations.json` → generate OKF source pages with attribution
3. Parse `report.md` → generate OKF topic summary page
4. Parse `contradictions.json` → generate contradiction pages with evidence
5. Parse `timeline.json` → generate chronological event page
6. Generate `index.md` with progressive disclosure catalog
7. Generate `log.md` with research session history
8. Write cross-links between all pages
9. Compute bundle hash for content-addressability

---

## 19. Threaded/Concurrent Research Architecture

### 19.1 The Multi-Threaded Research Paradigm

Concurrent research is the dominant paradigm for deep investigation in 2026. Leading systems demonstrate massive horizontal scaling:

| System | Scale | Pattern |
|--------|-------|---------|
| **Kimi K2.6** | 300 sub-agents, 4,000 steps | Dynamic task decomposition, shared context, Claw Groups |
| **MiniMax Agent Teams** | Leader-Worker-Verifier | Adversarial quality gates, parallel info retrieval |
| **Claude Code** | Subagents + Agent Teams | Parallel/sequential/background dispatch, context isolation |
| **LangGraph Deep Agents** | Subgraphs-as-nodes | Supervisor pattern, map-reduce, checkpoint persistence |

**Key insight for Prometheus:** The Feynman loop's `recursion` and `horizontal_escalation` patterns already provide the conceptual scaffolding for parallel research. Gaps spawn child loops; audience levels spawn parallel investigations. The deep-research pipeline should formalize this into a **Threaded Research Engine**.

### 19.2 Thread Decomposition Model

Research topics decompose into parallel threads via the **Planner** stage:

```
Research Query (L3: Outer Loop)
    ↓
Planner decomposes into N sub-questions
    ↓
Search Planner maps each sub-question to a thread
    ↓
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ Thread 1    │ │ Thread 2    │ │ Thread 3    │
│ Sub-q A     │ │ Sub-q B     │ │ Sub-q C     │
│ Evidence A  │ │ Evidence B  │ │ Evidence C  │
│ Partial KG  │ │ Partial KG  │ │ Partial KG  │
└─────────────┘ └─────────────┘ └─────────────┘
    ↓                    ↓              ↓
    └────────────────────┴──────────────┘
                      ↓
              Merge Stage (deterministic code)
              → Deduplicate → Entity Resolve → Conflict Detect
                      ↓
              Final Knowledge Graph + Report
```

### 19.3 Per-Thread Context Schema

Each thread maintains isolated context in surreal-memory:

```json
{
  "thread_id": "thread-uuid",
  "research_id": "parent-research-uuid",
  "sub_question": "What is the AG-UI protocol specification?",
  "system_prompt": "You are a research specialist investigating AG-UI...",
  "tool_permissions": ["tavily-mcp:search", "sequential-thinking:think"],
  "evidence": [],
  "partial_graph": {},
  "status": "running",
  "tokens_used": 0,
  "tokens_budget": 50000,
  "created_at": "2026-07-03T16:00:00Z",
  "last_checkpoint": "2026-07-03T16:15:00Z"
}
```

**Context isolation principles:**
- Each thread has its own isolated context window, custom system prompt, and restricted tool permissions
- Bulky artifacts (raw evidence, full pages) are stored externally in surreal-memory; only lightweight references stay in the thread context
- Verbose intermediate outputs are summarized before handoff to the merge stage
- Thread state is checkpointed to surreal-memory every 5–10 minutes

### 19.4 Thread Types

| Thread Type | Purpose | Spawn Trigger | Merge Strategy |
|-------------|---------|---------------|----------------|
| **Source Thread** | Deep investigation of one source type | One per source category (web, academic, API, graph) | Union of evidence sets |
| **Sub-question Thread** | Investigation of one atomic question | One per sub-question from Planner | Union of answers, intersection of citations |
| **Verification Thread** | Independent verification of a claim | One per claim with confidence < 0.8 | Voting/confidence averaging |
| **Synthesis Thread** | Cross-thread synthesis of findings | Spawned after all source threads complete | Knowledge graph merge |
| **Drift Thread** | Re-verification of stale claims | Scheduled by `prometheus-research drift` | Update or flag claims |

### 19.5 Merge Stage: Deterministic Aggregation

**Models do semantic work; code does deterministic work.** The merge stage uses deterministic code (not LLMs) for:

1. **Parsing**: Extract structured data from each thread's output
2. **Deduplication**: Remove repeated sources, claims, and files (canonical source ID at ingestion)
3. **Entity Resolution**: Merge duplicate entities across threads (graph matching algorithms improve accuracy >10%)
4. **Conflict Detection**: Identify contradictory claims across threads (temporal reasoning, confidence weighting)
5. **Ranking**: Score evidence by cross-thread support
6. **Provenance Preservation**: Track which thread discovered each claim

```
Thread Results → Parse → Deduplicate → Entity Resolve → 
Conflict Detect → Rank Evidence → Synthesize → Verify → 
Update Master Graph → Write .research Package
```

### 19.6 Cross-Thread Synchronization

A **shared discovery registry** in surreal-memory prevents redundant source retrieval:
- When a thread starts a search, it checks the registry
- If another thread already retrieved the source, the thread loads the cached version
- New sources are added to the registry with thread provenance

Cross-thread citation graph:
- Each citation includes `thread_provenance` (which thread discovered it)
- Citations discovered by multiple threads get higher confidence
- The verifier thread checks cross-thread support for each claim

### 19.7 KBD Orchestrator Integration

The KBD L1/L2/L3 loop architecture maps naturally to threaded research:

| KBD Layer | Threaded Research Role |
|---|---|
| **L3 (Outer)** | Defines the research goal, termination criteria, and thread budget |
| **L2 (Evolver)** | Adaptive decomposition strategy — adjusts thread count and depth based on preliminary findings |
| **L1 (KBD)** | Thread spawning and merge planning; one L1 tick = one thread merge cycle |
| **L0 (Harness)** | Individual thread execution (subagent or sub-process) |

### 19.8 Feynman Loop as Thread Orchestrator

The Feynman loop's patterns extend to parallel research:

```
/feynman-loop --concept-id "agent-protocols"
    ↓
Explain (produces research plan)
    ↓
Grade (identifies gaps)
    ↓
Gaps detected → Spawn parallel research threads (one per gap)
    ↓
┌─────────────────────────┐
│ Thread A: "What is MCP?"│
│ Thread B: "What is A2A?" │
│ Thread C: "What is AG-UI?"│
└─────────────────────────┘
    ↓
Merge (re-integrate findings into explanation)
    ↓
Re-grade (check if gaps are resolved)
```

**Horizontal escalation** (novice → peer → skeptic) spawns parallel threads at the same depth but different audience angles.

### 19.9 Concurrency Management

**Token budget reality:** Multi-agent systems use ~15× more tokens than single-agent; naive systems have 53–86% token duplication rates. Prometheus should implement:

- **AIMD backpressure** (TCP-inspired): Additive increase of thread count when latency is low; multiplicative decrease when API latency spikes
- **Semantic caching**: Cache similar search queries across threads (reduces overhead by ~70%)
- **Shared context pruning**: Remove redundant shared context between threads after each merge cycle
- **Tiered thread limits**: 
  - Web search threads: max 5 concurrent (API rate limits)
  - Verification threads: max 3 concurrent (model quota)
  - Synthesis threads: max 1 (serial by design)

---

## 20. Long-Running Process Management

### 20.1 The Durable Artifacts Pattern

The consensus architecture across Anthropic, Google, and the open-source community is: **the durable artifacts are the real continuity layer — not the model's context window.** The model is one worker inside a recoverable workflow. The workspace remembers.

```
LONG RUNNING RESEARCH HARNESS
=============================

initializer session
  ↓
creates durable research state:
  - research_spec.json        (topic, scope, deliverables, constraints)
  - research_plan.json        (sub-questions, priorities, dependencies)
  - progress_log.md           (chronological: what was tried, what worked, what failed)
  - evidence_bank/            (retrieved documents, excerpts, source metadata)
  - knowledge_graph.json      (entities, relationships, claims, confidence)
  - verification_log.json     (which claims were checked, by whom, against what)
  - current-waypoint.json     (KBD-style phase/stage tracking)
  ↓
worker session 1 starts fresh
  ↓
reads durable state → does one bounded task → verifies → logs → exits
  ↓
worker session 2 starts fresh
  ↓
reads durable state → continues
  ↓
...
  ↓
judge / evaluator decides whether research goal is met
```

**Key insight:** A fresh context window reading good state from disk is better than a stale context window carrying hours of tool output. Restarting is garbage collection.

### 20.2 Checkpointing and Resumption

**LangGraph checkpointing** (immutable checkpoint chains with `thread_id` isolation) is the industry standard for production:
- Fault tolerance: if a node fails at step 8, only step 8 is retried
- Human-in-the-loop: workflow pauses, checkpoints state, waits hours, then resumes exactly where it left off
- Time-travel debugging: navigate backward and forward through execution history
- Checkpoint forking: rewind to where bad data was introduced, inject correction, branch forward

**For Prometheus:** The deep-research pipeline should checkpoint after each stage:

```json
{
  "checkpoint_id": "chk-2026-07-03T16-00-00Z",
  "thread_id": "research-uuid-1234",
  "phase": "verify",
  "stage": "evidence_collection",
  "state": {
    "research_spec": { ... },
    "completed_sub_questions": ["sq-1", "sq-2"],
    "current_sub_question": "sq-3",
    "evidence_bank": { ... },
    "knowledge_graph": { ... },
    "context_summary": "Progressive summary of findings..."
  },
  "next_steps": ["retrieve_source_X", "verify_claim_Y"],
  "metadata": {
    "tokens_consumed": 152340,
    "sources_retrieved": 23
  }
}
```

**Checkpoints are stored in surreal-memory** under the `research_checkpoint` entity type, enabling:
- Cross-session resumption (different harness, same research)
- Multi-agent collaboration (multiple agents read/write same checkpoint)
- Historical audit (full trace of research evolution)

### 20.3 KBD Waypoint Pattern for Research

The existing KBD orchestrator `current-waypoint.json` pattern maps directly to research tracking:

```json
{
  "research_id": "research-uuid",
  "active_phase": "verify",
  "last_completed_stage": "evidence_collection",
  "next_action": "verify_claim_Y",
  "stages_completed": 5,
  "stages_total": 10,
  "sub_questions_completed": 2,
  "sub_questions_total": 5,
  "threads_active": 3,
  "threads_completed": 7,
  "quality_gates_passed": ["plan_approved", "source_verified"],
  "quality_gates_pending": ["contradiction_resolved", "citation_validated"],
  "budget": {
    "tokens_consumed": 152340,
    "tokens_budget": 500000,
    "api_calls": 47,
    "estimated_remaining_tokens": 100000
  },
  "session_history": [
    {"session_id": "sess-1", "started_at": "...", "ended_at": "...", "stages_completed": ["plan", "search"]},
    {"session_id": "sess-2", "started_at": "...", "ended_at": "...", "stages_completed": ["retrieve", "collect"]}
  ]
}
```

### 20.4 Preventing Context Window Degradation

**Progressive summarization** (5 layers) prevents attention diffusion:

| Layer | Content | Storage |
|-------|---------|---------|
| 1 | Raw highlights/excerpts from sources | `source_cache/` |
| 2 | Bolded main ideas from Layer 1 | `evidence_chunks.jsonl` |
| 3 | Highlighted top insights from Layer 2 | `verified_claims.json` |
| 4 | Synthesized claims with confidence and attribution | `resolved_claims.json` |
| 5 | Knowledge graph entities and relationships | `knowledge_graph.json` (surreal-memory) |

**Hybrid memory layers** (surreal-memory provides all of these):
- **Short-term**: Redis-style session cache (current thread context)
- **Medium-term**: Vector store for semantic search (evidence embeddings)
- **Long-term**: Graph database for entity relationships (knowledge graph)
- **Episodic**: Session logs and reasoning traces (audit trail)

### 20.5 Accuracy Preservation

**Verification at every step** prevents silent error accumulation:

| Verification Type | When | How |
|---|---|---|
| **LLM-as-Judge** | After each synthesis stage | Another model evaluates claim-source alignment |
| **Reflexion** | After each tool call | Agent reflects on whether the tool result was useful |
| **Adversarial Debate** | Before final export | Two agents debate the validity of key claims |
| **Process Verification** | Continuous | Monitor that each stage's outputs meet schema requirements |

**Confidence decay model:** Older claims lose confidence over time. After 30 days, confidence is multiplied by 0.9. After 90 days, by 0.7. Claims with confidence < 0.5 are flagged for re-verification.

### 20.6 Multi-Session/Day Research

**Session stitching** via structured handoff documents:
- Each session ends by writing a `session-handoff.json` to surreal-memory
- The next session reads the handoff and all checkpoints before continuing
- `progress_log.md` is append-only and human-readable
- Incremental report building: each session appends completed sections to `report.md`

**Time-aware source freshness:** Sources are tagged with retrieval timestamp. After 7 days, web sources are flagged as potentially stale. After 30 days, they require re-verification.

### 20.7 Research as Background Process

**Cron + webhook + event-driven triggers:**

```bash
# Schedule recurring research (e.g., weekly market analysis)
prometheus-research schedule \
  --topic "weekly AI market analysis" \
  --cron "0 9 * * 1" \
  --output "./research/weekly/"

# Triggered research (e.g., when a knowledge drift threshold is exceeded)
prometheus-research watch \
  --research-id research-abc123 \
  --drift-threshold 0.8 \
  --on-drift "re-verify and extend"
```

**Background execution via KBD L3 outer loop:**
```
/loop-define weekly-research
   goal: "Produce weekly AI market analysis"
   feedback_sources: [tavily-mcp, prometheus-knowledge]
   termination: { max_ticks: 1, cadence: cron("0 9 * * 1") }
   escalation: { on_drift: "re-verify", on_stale: "re-search" }
```

### 20.8 The Karpathy Loop in Long-Running Research

The Karpathy Loop (bounded context → enqueue → supervised reflect/ingest) operates at two frequencies:

**Micro-loop (per iteration):**
- **Context**: `pk context` reads bounded committed project/shared/global snapshots before each planning step
- **Reflect**: `forge reflect` evaluates the quality of each research stage
- **Enqueue**: completion writes one metadata-only job; the worker performs durable ingestion

**Macro-loop (per session):**
- **Focus**: Load prior research state from surreal-memory before starting a new session
- **Reflect**: `forge reflect` on the entire research session quality
- **Ingest**: Write the complete `.research` package to the knowledge base and wiki

### 20.9 Graceful Degradation for Partial Failures

**Degradation ladder:**
1. **Retry** (exponential backoff, max 3 attempts)
2. **Fallback** (switch to alternative retriever: Tavily → Brave → SerpAPI)
3. **Reduce capability** (lower depth, reduce thread count, skip verification)
4. **Cache** (use stale data with confidence penalty)
5. **Human handoff** (pause and ask for direction)

**Partial `.research` packages are valid and useful:** Even incomplete research produces valuable knowledge assets. A package with `completion_status: "partial"` and `progress_percentage: 65` is still queryable, citable, and extensible.

### 20.10 surreal-memory as the Unified Knowledge Layer

surreal-memory is the ideal substrate for long-running research because it provides all required data models in one system:

| Research Need | surreal-memory Feature | Usage |
|---------------|------------------------|-------|
| **Entity graph** | Native graph queries (RELATE, ->, <-) | Knowledge graph storage and traversal |
| **Vector search** | Embedding + HNSW index | Semantic similarity for evidence retrieval |
| **Document store** | JSON document model | Checkpoints, thread state, handoff documents |
| **Relational** | SQL-like tables | Citation registry, claim database, session log |
| **Time-travel** | `VERSION` clause | Audit history, drift detection |
| **Real-time sync** | ElectricSQL + PGlite | Offline research on Tauri desktop, sync when online |
| **MCP integration** | Native MCP server | Any agent can query/update research state |

**Research-specific surreal-memory schema:**
```sql
DEFINE TABLE research_checkpoint SCHEMAFULL;
DEFINE FIELD research_id ON research_checkpoint TYPE string;
DEFINE FIELD phase ON research_checkpoint TYPE string;
DEFINE FIELD stage ON research_checkpoint TYPE string;
DEFINE FIELD state ON research_checkpoint TYPE object;
DEFINE FIELD metadata ON research_checkpoint TYPE object;
DEFINE FIELD created_at ON research_checkpoint TYPE datetime;

DEFINE TABLE research_thread SCHEMAFULL;
DEFINE FIELD research_id ON research_thread TYPE string;
DEFINE FIELD sub_question ON research_thread TYPE string;
DEFINE FIELD status ON research_thread TYPE string;
DEFINE FIELD evidence ON research_thread TYPE array;
DEFINE FIELD partial_graph ON research_thread TYPE object;
DEFINE FIELD tokens_used ON research_thread TYPE int;

DEFINE TABLE research_claim SCHEMAFULL;
DEFINE FIELD text ON research_claim TYPE string;
DEFINE FIELD confidence ON research_claim TYPE float;
DEFINE FIELD sources ON research_claim TYPE array;
DEFINE FIELD verified ON research_claim TYPE bool;
DEFINE FIELD thread_id ON research_claim TYPE string;

DEFINE TABLE research_source SCHEMAFULL;
DEFINE FIELD url ON research_source TYPE string;
DEFINE FIELD title ON research_source TYPE string;
DEFINE FIELD content_hash ON research_source TYPE string;
DEFINE FIELD retrieved_at ON research_source TYPE datetime;
DEFINE FIELD thread_id ON research_source TYPE string;
```

---

## 21. Updated References

### Deep Research Agents

1–10. *(same as §16)*

### Skill Platforms

11–18. *(same as §16)*

### UI Protocols

19–25. *(same as §16)*

### Architecture & Knowledge Graphs

26–35. *(same as §16)*

### Feynman Learning Integration

36. Huber — Inquiry-Based Learning definition [https://www.uni-bielefeld.de](https://www.uni-bielefeld.de)
37. ACRL Framework — "Research as Inquiry" [https://www.ala.org/acrl/standards/ilframework](https://www.ala.org/acrl/standards/ilframework)
38. UNESCO — Four-step inquiry process [https://unesdoc.unesco.org](https://unesdoc.unesco.org)
39. SP-TeachLLM — Curriculum Decomposition Module [https://arxiv.org/abs/2506.10466](https://arxiv.org/abs/2506.10466)
40. Aldrich — Curriculum Prerequisite Network [https://journals.sagepub.com/doi/10.1177/1052562915590738](https://journals.sagepub.com/doi/10.1177/1052562915590738)
41. Feynman Technique Tutor — Gap identification [https://github.com/jasonjmcgifford/feynman-technique-tutor](https://github.com/jasonjmcgifford/feynman-technique-tutor)
42. VoiceScriber — Self-explanation meta-analysis [https://arxiv.org/abs/2501.03297](https://arxiv.org/abs/2501.03297)
43. Karpathy — LLM Wiki Gist [https://gist.github.com/karpathy/...
](https://gist.github.com/karpathy/...)

### Google OKF & AI Document Standards

44. Google Cloud — OKF v0.1 Announcement [https://cloud.google.com/blog/topics/knowledge-management/introducing-open-knowledge-format](https://cloud.google.com/blog/topics/knowledge-management/introducing-open-knowledge-format)
45. Google Cloud — OKF Specification [https://github.com/GoogleCloudPlatform/knowledge-catalog](https://github.com/GoogleCloudPlatform/knowledge-catalog)
46. Sam McVeety — OKF Design Rationale [https://cloud.google.com/blog/products/data-analytics/open-knowledge-format-okf](https://cloud.google.com/blog/products/data-analytics/open-knowledge-format-okf)
47. Google — A2UI Protocol [https://a2ui.org/](https://a2ui.org/)
48. Google Developers — A2UI v0.9 [https://developers.googleblog.com/a2ui-v0-9-generative-ui/](https://developers.googleblog.com/a2ui-v0-9-generative-ui/)
49. llms.txt — Discovery standard [https://llmstxt.org/](https://llmstxt.org/)
50. AGENTS.md — Cross-tool standard [https://github.com/addyosmani/agent-engineer](https://github.com/addyosmani/agent-engineer)

### Threaded/Concurrent Research

51. Kimi K2.6 — 300 sub-agents [https://www.kimi.com/blog/kimi-k2-6](https://www.kimi.com/blog/kimi-k2-6)
52. MiniMax Agent Teams [https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953](https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953)
53. Claude Code Subagents [https://www.builder.io/blog/claude-code-subagents](https://www.builder.io/blog/claude-code-subagents)
54. LangGraph Deep Agents [https://github.com/langchain-ai/deepagents](https://github.com/langchain-ai/deepagents)
55. Claude Code Parallel Subagents Best Practices [https://claudecodeguides.com/parallel-subagents-claude-code-best-practices-2026/](https://claudecodeguides.com/parallel-subagents-claude-code-best-practices-2026/)
56. Rivista AI — Context Isolation [https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)
57. RapidClaw — LangGraph Production [https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026](https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026)
58. Graph Matching-Based Knowledge Fusion [https://www.researchsquare.com/article/rs-4641408/v1.pdf](https://www.researchsquare.com/article/rs-4641408/v1.pdf)

### Long-Running Process Management

59. Addy Osmani — Long-running Agents [https://addyosmani.com/blog/long-running-agents/](https://addyosmani.com/blog/long-running-agents/)
60. Nicolas Bustamante — Long Running Agent Engineering [https://nicolasbustamante.com/blog/long-running-agent-engineering](https://nicolasbustamante.com/blog/long-running-agent-engineering)
61. Google ADK — Persistent Sessions [https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/](https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/)
62. LangGraph Checkpointing [https://www.autolearningagents.com/langgraph/checkpointing.php](https://www.autolearningagents.com/langgraph/checkpointing.php)
63. DeepAgents on LangGraph [https://pub.towardsai.net/deepagents-on-langgraph-debugging-long-running-ai-agents-with-time-travel-ff897ef50b73](https://pub.towardsai.net/deepagents-on-langgraph-debugging-long-running-ai-agents-with-time-travel-ff897ef50b73)
64. Easton Dev — LangGraph State Management [https://eastondev.com/blog/en/posts/ai/20260424-langgraph-agent-architecture/](https://eastondev.com/blog/en/posts/ai/20260424-langgraph-agent-architecture/)
65. DuraLang — Temporal + LangChain [https://github.com/ombharatiya/ai-system-design-guide/blob/main/07-agentic-systems/11-durable-execution.md](https://github.com/ombharatiya/ai-system-design-guide/blob/main/07-agentic-systems/11-durable-execution.md)
66. Indium — State Persistence Strategies [https://www.indium.tech/blog/7-state-persistence-strategies-ai-agents-2026/](https://www.indium.tech/blog/7-state-persistence-strategies-ai-agents-2026/)
67. Exabase — Memory Drift in AI Agents [https://exabase.io/blog/what-is-memory-drift-in-ai-agents](https://exabase.io/blog/what-is-memory-drift-in-ai-agents)

---

*This specification was synthesized from exhaustive parallel research across 67+ authoritative sources, conducted on 2026-07-03. It is intended as a living document — please open issues or PRs for corrections, additions, or refinements.*

*Sections 17–20 added after second research phase covering Feynman learning integration, Google OKF alignment, threaded/concurrent research architecture, and long-running process management with surreal-memory as the unified knowledge layer.*
