---
title: Prometheus Deep Research Skill — Master Specification
date: 2026-07-03
version: 1.0.0-draft
status: specification
authors: [Prometheus AGS Research Team]
---

# Prometheus Deep Research Skill — Master Specification

> **Status:** Draft specification for community review  
> **Version:** 1.0.0-draft  
> **Date:** 2026-07-03  
> **Scope:** Architecture, feature specification, platform strategy, and implementation roadmap for a world-class, harness-agnostic deep-research skill within the Prometheus Skill Pack ecosystem.

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

---

## 1. Executive Summary

After exhaustive research across 50+ sources spanning open-source projects, academic papers, protocol specifications, and platform documentation, we conclude that **the deep-research agent landscape has matured beyond disposable reports to a new paradigm: persistent, queryable knowledge assets**. Every leading project — from GPT Researcher to LangGraph Open Deep Research to MiroThinker — still primarily emits static documents. The opportunity for Prometheus is to build the **first deep-research system that natively produces structured, extensible knowledge packages** while remaining installable and operable across any skill-aware harness.

This specification defines:
- A **10-stage research pipeline** (Planner → Search → Retrieve → Collect → Verify → Resolve → Graph → Cite → Report → Export) implemented as a portable skill.
- A **dual-nature delivery**: both a `SKILL.md`-based portable skill (for Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, Roo, Amp, Gemini) and a native Rust MCP server with AG-UI/A2UI streaming.
- A **`.research` package format** containing knowledge graphs, citations, embeddings, contradictions, confidence scores, and audit trails — not just a report.
- A **unified React + AG-UI research app** that can render as an MCP App, a standalone web UI, or a Tauri desktop app, using `prometheus-entity-management` for reactive state and `flint-platform-agent` for runtime coordination.

**Key differentiator:** Other agents produce reports. Prometheus Research produces **knowledge infrastructure** that other agents can query, extend, and cite.

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

*This specification was synthesized from exhaustive parallel research across 50+ authoritative sources, conducted on 2026-07-03. It is intended as a living document — please open issues or PRs for corrections, additions, or refinements.*
