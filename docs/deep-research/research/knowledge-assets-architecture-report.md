# Knowledge Assets & Architecture Patterns
## Deep Research Skill — Foundational Research Report

**Research Date:** 2026-07-03  
**Researcher:** Orchestrator Research Specialist  
**Scope:** Foundational investigation for designing a universal `deep-research` skill for the Prometheus Skill Pack  
**Target:** Harness-agnostic, portable skill that emits persistent knowledge assets rather than disposable reports

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Knowledge Assets vs. Disposable Reports](#2-knowledge-assets-vs-disposable-reports)
3. [Research Package Formats](#3-research-package-formats)
4. [MCP Server Patterns for Research Capabilities](#4-mcp-server-patterns-for-research-capabilities)
5. [Prometheus MCP Stack Integration](#5-prometheus-mcp-stack-integration)
6. [Universal Deep-Research Skill Architecture](#6-universal-deep-research-skill-architecture)
7. [Persistent Knowledge Objects & Inter-Agent Protocol](#7-persistent-knowledge-objects--inter-agent-protocol)
8. [Native CLI Tooling Recommendations](#8-native-cli-tooling-recommendations)
9. [Comparative Analysis Matrix](#9-comparative-analysis-matrix)
10. [Recommendations for Prometheus Skill Pack](#10-recommendations-for-prometheus-skill-pack)
11. [References & Citations](#11-references--citations)

---

## 1. Executive Summary

The current generation of "deep research" agents (GPT Researcher, LangGraph Open Deep Research, OpenResearcher, Together AI, MiroThinker) predominantly emit **disposable reports**—static Markdown or PDF artifacts that are consumed once and discarded. The next evolutionary step is to treat research outputs as **knowledge assets**: structured, versioned, queryable, and extensible objects that other agents can cite, extend, and build upon.

This report defines the architecture for a **universal deep-research skill** that:
- Produces **persistent knowledge packages** (not just reports)
- Integrates natively with the **Prometheus MCP substrate** (tavily-mcp, sequential-thinking, liter-llm, surreal-memory, forge-rs, prometheus-knowledge)
- Exposes a **Rust-native CLI** (`prometheus-research`) consistent with the existing pack binary surface
- Works across **all harnesses** (Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, Roo, Amp, Gemini CLI)
- Supports **AG-UI / A2UI** real-time progress streaming for long-running research workflows

**Key insight:** The value of deep research is not the report—it's the **structured evidence graph** that enables future reasoning, verification, and synthesis.

---

## 2. Knowledge Assets vs. Disposable Reports

### 2.1 What is a Knowledge Asset?

A **knowledge asset** is a structured, persistent, and machine-actionable representation of research findings that outlives the session that produced it. Unlike a disposable report, a knowledge asset:

| Dimension | Disposable Report | Knowledge Asset |
|-----------|-------------------|-----------------|
| **Format** | Markdown, PDF, DOCX | JSON package with graph, embeddings, traces |
| **Lifespan** | Single-session consumption | Persistent, versioned, extensible |
| **Queryability** | Full-text search only | Graph traversal, semantic search, structured queries |
| **Citations** | Inline hyperlinks or footnotes | Machine-readable `citations.json` with DOI, URL, confidence, access timestamp |
| **Extensibility** | Manual editing | Agent-mergable: other agents can append nodes, resolve conflicts, add evidence |
| **Verifiability** | Human-read only | Machine-checkable: source hashes, retrieval traces, confidence scores |
| **Reusability** | Copy-paste | Importable as MCP tool context or RAG corpus |

**Source:** Neo4j Blog — "Context Engineering in AI Agents" (2026-06-04) [https://neo4j.com/blog/agentic-ai/what-is-context-engineering/]

### 2.2 Existing Format Landscape

| Format / Standard | Purpose | Machine-Readable? | Citation-Native? |
|-------------------|---------|-------------------|------------------|
| **Markdown** | Human-readable report | Partial (frontmatter) | No (manual links) |
| **PDF/DOCX** | Document distribution | No | No |
| **JSON-LD (schema.org)** | Linked data, KG triples | Yes | Via `@id` URIs |
| **CITATION.cff** | Software citation metadata | Yes | Yes (software-focused) |
| **BibTeX / CSL-JSON** | Academic bibliography | Yes | Yes (bibliographic only) |
| **RDF / N-Triples** | Semantic web data | Yes | Via `dcterms:references` |
| **IPFS / IPLD** | Content-addressed storage | Yes | Content-hash native |
| **OpenAlex JSON** | Academic work metadata | Yes | Yes (works, authors, institutions) |
| **Obsidian / Logseq** | Personal knowledge graphs | Partial (Markdown + links) | Backlink-based |
| **JSON-RAG / KG v5** | Structured data + vector hybrid | Yes | Via `source` fields |

**Key observation:** No single existing format captures the full research lifecycle (plan → search → retrieve → verify → synthesize → cite → extend). A **research package format** must be invented or assembled from existing primitives.

**Source:** GraphAware — "LLMs for Knowledge Graph: GPT Prompt Engineering" (2023-10-24) [https://graphaware.com/blog/episode-2-gpt-prompt-engineering/]; GitHub — shihentsou/json-rag [https://github.com/shihentsou/json-rag]

### 2.3 The Research Package as a Knowledge Asset

We propose a **Research Package** (`.research/` directory or `.research.tar.zst`) as the canonical knowledge asset format. It contains:

```
my-topic.research/
├── manifest.json              # Package metadata, schema version, provenance
├── report.md                  # Human-readable final report (optional but expected)
├── citations.json             # Structured citation database
├── knowledge_graph.json       # Entity-relationship graph (JSON-LD or custom schema)
├── embeddings/                # Vector embeddings of chunks, claims, entities
│   ├── chunks-embeddings.npy
│   └── entity-embeddings.npy
├── entity_graph.json          # Typed entities and their resolved canonical IDs
├── timeline.json              # Temporal events with confidence intervals
├── contradictions.json        # Detected conflicts with evidence for each side
├── confidence_scores.json     # Per-claim, per-source, per-entity confidence
├── follow_up_questions.json   # Open questions generated during synthesis
├── source_cache/              # Mirrored/raw source content (with content hashes)
│   ├── source-001/
│   │   ├── content.html       # Raw fetched content
│   │   ├── content.hash       # SHA-256 of content at retrieval time
│   │   └── metadata.json      # URL, fetch timestamp, retriever used, headers
├── search_trace.json          # Every query issued, results retrieved, ranking
├── reasoning_trace.json       # LLM reasoning steps, tool calls, plan revisions
├── artifacts/                 # Generated exports: PDF, DOCX, PPTX, CSV
└── SKILL.md                   # How to use this research package as a skill context
```

This package is **content-addressable** (top-level hash), **diffable** (JSON components can be merged), and **self-describing** (manifest + SKILL.md).

---

## 3. Research Package Formats

### 3.1 `citations.json` — Machine-Readable Citation Database

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

### 3.2 `knowledge_graph.json` — Structured Entity-Relationship Graph

Inspired by JSON-LD and GraphAware's KG prompt engineering approach, the graph uses typed nodes and edges:

```json
{
  "entities": [
    {"id": "ent-001", "type": "Organization", "name": "OpenAI", "canonical_id": "wikidata:Q217219}",
    {"id": "ent-002", "type": "Product", "name": "GPT-4", "canonical_id": null}
  ],
  "relations": [
    {"source": "ent-002", "relation": "DEVELOPED_BY", "target": "ent-001", "confidence": 0.98, "sources": ["cite-001"]}
  ],
  "provenance": "prometheus-research-v1"
}
```

**Source:** GraphAware LLM-KG blog [https://graphaware.com/blog/episode-2-gpt-prompt-engineering/]; GitHub — shimo4228/claude-skill-jsonld-knowledge-graph [https://github.com/shimo4228/claude-skill-jsonld-knowledge-graph]

### 3.3 `embeddings/` — Vector Index for Semantic Retrieval

Stores embeddings of:
- **Source chunks** (for RAG-style retrieval within the research package)
- **Claims** (for semantic similarity and contradiction detection)
- **Entities** (for entity resolution and disambiguation)

Uses **HNSW** indexing (via Qdrant, pgvector, or local hnswlib). The Prometheus stack already uses surreal-memory (Neo4j + vector) for this purpose.

### 3.4 `entity_graph.json` — Canonical Entity Resolution

Maps extracted entities to canonical identifiers (Wikidata, ORCID, company registries). Prevents entity duplication across research sessions and enables cross-research linking.

### 3.5 `timeline.json` — Temporal Event Sequencing

```json
[
  {"event": "GPT-4 launched", "date": "2023-03-14", "confidence": 1.0, "entities": ["ent-002"], "source": "cite-001"}
]
```

### 3.6 `contradictions.json` — Conflict Detection & Evidence Balancing

```json
[
  {
    "id": "contradiction-001",
    "claim_a": {"text": "X is true", "sources": ["cite-001"], "confidence": 0.8},
    "claim_b": {"text": "X is false", "sources": ["cite-002"], "confidence": 0.7},
    "resolution_strategy": "HUMAN_REVIEW_REQUIRED",
    "auto_resolution": null
  }
]
```

### 3.7 `search_trace.json` & `reasoning_trace.json` — Auditability

Every search query, every tool call, every LLM reasoning step is logged with timestamps and token costs. This enables:
- **Reproducibility**: Re-run the same research with identical parameters
- **Verification**: Human auditors can inspect every decision
- **Cost tracking**: Per-research and per-session cost attribution
- **Debugging**: Identify where hallucinations or mis-retrievals occurred

**Source:** LangSmith / Open Deep Research evaluation tracing; AgentForge architecture [https://github.com/omkarbhad/agentforge]

---

## 4. MCP Server Patterns for Research Capabilities

### 4.1 Why MCP for Research?

The **Model Context Protocol (MCP)** standardizes how AI agents discover and invoke tools. For deep research, MCP servers abstract the complexity of:
- Web search (Tavily, Brave, SerpAPI, native Google/Bing)
- Document retrieval (RAG over local files, databases, APIs)
- Memory and knowledge graphs (Neo4j, SurrealDB, SQLite + vector)
- Reasoning traces (sequential thinking, reflection)
- Citation management (format generation, DOI resolution, link validation)

**Source:** Tetrate — "MCP + RAG: When to Use Both Together" (2026-01-16) [https://tetrate.io/learn/ai/mcp/mcp-rag-when-to-use-both-together]

### 4.2 Best Patterns for MCP-Wrapped Research

#### Pattern A: Retriever-as-Tool (Tavily-MCP model)
The MCP server exposes a single `search` tool that returns ranked, summarized results with source metadata. The LLM decides when to search and how to combine results.

**Example:** Tavily-MCP server exposes `tavily_search(query, max_results, include_raw)` → returns JSON with URLs, titles, snippets, raw content.

**Strength:** Simple, universal, works with any MCP client.  
**Weakness:** LLM must orchestrate multi-step retrieval itself; no built-in planning or verification.

#### Pattern B: Planner-Retriever-Verifier Pipeline (LangGraph Open Deep Research model)
The MCP server exposes a **high-level** `conduct_research` tool that internally runs a full pipeline: plan → decompose → retrieve → verify → synthesize → return structured report.

**Example:** Open Deep Research's LangGraph server exposes a research assistant that can be configured with different MCP tools and search APIs.

**Strength:** Encapsulates complexity; client just sets parameters.  
**Weakness:** Less flexible for clients that want fine-grained control over each step.

#### Pattern C: Hybrid RAG-MCP Loop (MCP-Enhanced RAG)
MCP tools enhance the RAG pipeline itself. The retrieval process uses both static vector stores and dynamic MCP tools (web search, database queries, API calls) as heterogeneous retrieval sources.

**Source:** Tetrate — "Pattern 4: MCP-Enhanced RAG" [https://tetrate.io/learn/ai/mcp/mcp-rag-when-to-use-both-together]

#### Pattern D: Knowledge Graph + MCP (Surreal-Memory / Prometheus-Knowledge model)
The MCP server wraps a graph database (Neo4j, SurrealDB) with vector search. Research findings are persisted as graph nodes/edges, and subsequent research can query the graph via MCP tools.

**Example:** CASCADE architecture uses a Memory Server with Neo4j graph + Supabase vector store, exposing `save_to_memory` and `search_memory` tools.

**Source:** CASCADE paper (2025) — arXiv:2512.23880 [https://arxiv.org/html/2512.23880v1]

#### Pattern E: Multi-Server Orchestration (Prometheus Stack model)
Multiple specialized MCP servers work together:
- `tavily-mcp` → real-time web search
- `sequential-thinking-mcp` → structured reasoning traces
- `liter-llm-mcp` → LLM proxy + model routing
- `surreal-memory-mcp` → persistent memory + graph
- `forge-rs-mcp` → enrichment, reflection, template processing
- `prometheus-knowledge-mcp` → Karpathy-style knowledge base retrieval

The orchestrator (Prometheus skill harness) routes calls between these servers based on the research phase.

**Source:** Prometheus Skill Pack — CLI & Scripts Reference [https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/guide/16-cli-and-scripts.md]

### 4.3 Recommended MCP Interface for Deep Research

We recommend a **tiered MCP interface** that exposes both high-level and low-level tools:

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

---

## 5. Prometheus MCP Stack Integration

### 5.1 Existing Stack Components

The Prometheus Skill Pack already includes a rich MCP substrate. The deep-research skill must integrate natively with:

| Component | Role | Port (SSE) | Integration Point for Deep Research |
|-----------|------|------------|-------------------------------------|
| **tavily-mcp** | Web search | stdio | Primary real-time information retrieval; cross-validation with secondary sources |
| **sequential-thinking** | Reasoning trace | stdio | Capture the planner's reasoning, question decomposition, and hypothesis evolution |
| **liter-llm** | LLM proxy + MCP tool server | stdio | Route research sub-tasks to appropriate models (cheap for summarization, powerful for synthesis) |
| **surreal-memory-server** | Graph memory + MCP + REST | 23001 | Store and retrieve knowledge graphs, entity relationships, and research session memory |
| **prometheus-knowledge** | Karpathy KB CLI / MCP bridge | 8942 | Retrieve existing knowledge base entries; prevent redundant research; link new findings to prior knowledge |
| **forge-rs** | Enrichment, reflection, drift | 8943 | Post-research enrichment: extract entities, detect drift from prior knowledge, generate reflection summaries |
| **sycophancy-correction** | Bias correction | stdio | Verify that research findings aren't over-aligned with the user's prior beliefs; flag confirmation bias |

**Source:** Prometheus Skill Pack — CLI & Scripts Reference (docs/guide/16-cli-and-scripts.md) [https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/guide/16-cli-and-scripts.md]

### 5.2 Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              prometheus-research CLI / MCP Server            │
│  (Rust, Axum, SSE + stdio transports, AG-UI endpoint)        │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┬────────────┬────────────┐
        ▼            ▼            ▼            ▼            ▼
  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
  │  tavily  │ │  liter   │ │  surreal │ │ prometheus│ │  forge   │
  │   -mcp   │ │   -llm   │ │  -memory │ │  -knowledge│ │   -rs    │
  │ (search) │ │ (models) │ │ (graph+  │ │ (KB      │ │(enrich,  │
  │          │ │          │ │ vector)  │ │ retrieve)│ │ reflect) │
  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
        │            │            │            │            │
        └────────────┴────────────┴────────────┴────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
           ┌─────────────┐       ┌─────────────┐
           │ sequential  │       │ sycophancy  │
           │ -thinking   │       │ -correction │
           │ (reasoning) │       │ (bias guard)│
           └─────────────┘       └─────────────┘
```

### 5.3 Specific Integration Patterns

**Tavily-MCP Integration:**
- Use `tavily_search` for primary web retrieval
- Configure `RETRIEVER=tavily,mcp` in GPT Researcher style to enable hybrid web + MCP research
- Cache raw results in `source_cache/` with content hashes

**Sequential-Thinking Integration:**
- Before each planning phase, initialize a sequential-thinking trace
- Log the planner's reasoning (why these sub-questions, why this search strategy)
- At verification, use the trace to audit whether the retriever followed the plan

**Surreal-Memory Integration:**
- After each research session, save the `knowledge_graph.json` and `entity_graph.json` to surreal-memory
- Future research queries `search_memory` to find prior related research
- Use graph traversal to answer "what do we already know about X?" before searching again

**Prometheus-Knowledge Integration:**
- Check the Karpathy KB (via `pk` CLI / MCP bridge) for existing knowledge on the topic
- If prior research exists, load it as a starting point and flag "what has changed since last research?"
- Append new findings to the KB rather than replacing

**Forge-RS Integration:**
- After report generation, run `forge reflect` to detect gaps, contradictions, or outdated claims
- Use `forge enrich` to extract additional entities and relationships from the raw source cache
- Use `forge drift` to compare new findings against the existing knowledge base and flag discrepancies

**Sycophancy-Correction Integration:**
- Before finalizing, run the user's original query and the research findings through the sycophancy checker
- Flag if the report over-validates the user's implicit assumptions
- Generate a "devil's advocate" section with counter-evidence

---

## 6. Universal Deep-Research Skill Architecture

### 6.1 Proposed Architecture: The Prometheus Research Pipeline

Drawing from GPT Researcher's planner-execution model, LangGraph Open Deep Research's configurable pipeline, and AgentForge's multi-agent verification loop, we propose a **10-stage architecture**:

```
┌────────────────────────────────────────────────────────────────────────────┐
│                     PROMETHEUS RESEARCH PIPELINE                          │
├────────────────────────────────────────────────────────────────────────────┤
│  Stage 1: PLANNER           → Decompose query into sub-questions            │
│  Stage 2: SEARCH PLANNER    → Generate search queries per sub-question      │
│  Stage 3: RETRIEVER         → Hybrid: web (Tavily) + RAG + graph + API    │
│  Stage 4: EVIDENCE COLLECTOR→ Normalize, deduplicate, chunk, score        │
│  Stage 5: EVIDENCE VERIFIER → Faithfulness check, source validation       │
│  Stage 6: CONFLICT RESOLVER → Detect contradictions, score confidence       │
│  Stage 7: KNOWLEDGE GRAPH   → Build entity-relationship graph               │
│  Stage 8: CITATION MANAGER  → Generate structured citations, resolve DOIs     │
│  Stage 9: REPORT GENERATOR  → Synthesize findings into report.md            │
│  Stage 10: ARTIFACT EXPORTER→ Emit .research package, PDF, DOCX, etc.     │
└────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Stage Details

#### Stage 1: Planner (Question Decomposer)
- **Input:** User query + optional constraints (depth, breadth, format, date range)
- **Process:** LLM generates an objective research plan with N sub-questions
- **Output:** `plan.json` — ordered sub-questions with priority, estimated sources, and success criteria
- **MCP Tools:** `sequential-thinking` (trace reasoning), `liter-llm` (route to appropriate model)

**Source:** GPT Researcher architecture — "Planner generates research questions" [https://docs.gptr.dev/blog]; LangGraph Open Deep Research — `plan_and_execute` workflow [https://github.com/langchain-ai/open_deep_research]

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
- **Pattern:** MCP-Enhanced RAG (Tetrate Pattern 4) — treat tool invocation as part of retrieval

**Source:** Tetrate — MCP-Enhanced RAG [https://tetrate.io/learn/ai/mcp/mcp-rag-when-to-use-both-together]; AgentForge — hybrid retriever (Qdrant + web) [https://github.com/omkarbhad/agentforge]

#### Stage 4: Evidence Collector
- **Input:** `raw_evidence/`
- **Process:** Normalize formats (HTML → Markdown, JSON → flat); deduplicate by content hash; chunk into context windows; score relevance with embeddings + cross-encoder
- **Output:** `evidence_chunks.jsonl` — scored, chunked, deduplicated evidence
- **MCP Tools:** `liter-llm` (embedding model routing), `forge-rs` (enrichment)

#### Stage 5: Evidence Verifier
- **Input:** `evidence_chunks.jsonl` + `plan.json`
- **Process:** For each claim in the draft, check if evidence supports it (claim extraction → evidence matching → faithfulness scoring); if faithfulness < threshold, route back to retriever with refined query
- **Output:** `verified_claims.json` — claims with confidence scores and supporting evidence IDs
- **MCP Tools:** `sequential-thinking` (structured verification), `sycophancy-correction` (bias check)
- **Max Retries:** 2 loops before returning best attempt with confidence flag

**Source:** AgentForge verifier agent — "If faithfulness < threshold → routes back to Retriever" [https://github.com/omkarbhad/agentforge]; LongTracer — citation verification framework [https://github.com/ENDEVSOLS/LongTracer]

#### Stage 6: Conflict Resolver
- **Input:** `verified_claims.json`
- **Process:** Detect contradictions (semantic similarity + logical negation detection); score evidence balance for each side; flag for human review if auto-resolution confidence < 0.8
- **Output:** `contradictions.json` + `resolved_claims.json`
- **MCP Tools:** `surreal-memory` (check prior knowledge for established facts), `forge-rs` (drift detection)

#### Stage 7: Knowledge Graph Builder
- **Input:** `resolved_claims.json` + `citations.json`
- **Process:** Extract entities (NER); resolve to canonical IDs (Wikidata, ORCID, company registries); infer relationships; build JSON-LD compatible graph
- **Output:** `knowledge_graph.json` + `entity_graph.json`
- **MCP Tools:** `surreal-memory` (persist graph), `prometheus-knowledge` (link to KB)

**Source:** GraphAware — prompt engineering for KG construction [https://graphaware.com/blog/episode-2-gpt-prompt-engineering/]; ZBrain — GraphRAG for multi-hop reasoning [https://zbrain.ai/knowledge-graphs-for-agentic-ai/]

#### Stage 8: Citation Manager
- **Input:** `knowledge_graph.json` + `source_cache/`
- **Process:** Resolve DOIs; validate URLs (HEAD request, check for 404); format citations in requested style (APA, MLA, Chicago, IEEE, BibTeX); generate `citations.json`
- **Output:** `citations.json` + inline citation markers in `report.md`

#### Stage 9: Report Generator
- **Input:** All prior outputs
- **Process:** Synthesize into structured report: executive summary, methodology, findings per sub-question, contradictions/discussions, conclusion, references
- **Output:** `report.md` (primary) + `report.pdf`, `report.docx` (optional)
- **MCP Tools:** `liter-llm` (synthesis model, e.g., GPT-4o, Claude Sonnet)
- **Pattern:** Human-in-the-loop optional: pause before final write for user approval of plan/structure

#### Stage 10: Artifact Exporter
- **Input:** All research artifacts
- **Process:** Bundle into `.research` package; compress; compute content hash; optionally publish to IPFS or knowledge base
- **Output:** `my-topic.research.tar.zst` (content-addressable)
- **MCP Tools:** `prometheus-knowledge` (publish to KB), `surreal-memory` (index graph)

### 6.3 State Machine & Orchestration

The pipeline is implemented as a **LangGraph-compatible state machine** with checkpoints after each stage. This enables:
- **Resumability:** If a stage fails, resume from the last checkpoint
- **Human-in-the-loop:** Pause at configurable gates (plan approval, conflict review)
- **Parallelism:** Stages 3–5 can run in parallel per sub-question
- **Observability:** Full trace via LangSmith or OpenTelemetry

**Source:** GPT Researcher LangGraph multi-agent — "Browser → Planner → Researcher → Writer → Publisher" [https://docs.gptr.dev/blog]; LangGraph Open Deep Research — stateful graph with reflection loops [https://github.com/langchain-ai/open_deep_research]

---

## 7. Persistent Knowledge Objects & Inter-Agent Protocol

### 7.1 Emitting Knowledge Objects

The deep-research skill must emit objects that are **queryable by other agents** without re-running the research. We define three tiers of knowledge objects:

| Tier | Object | Format | Query Interface | Lifetime |
|------|--------|--------|-----------------|----------|
| **L1** | Research Package | `.research.tar.zst` | File system / IPFS | Permanent (user-managed) |
| **L2** | Knowledge Graph Fragment | `knowledge_graph.json` + embeddings | MCP tool `research_query_graph` | Persistent (surreal-memory) |
| **L3** | Citation & Claim Database | `citations.json` + `verified_claims.json` | MCP tool `research_query_claims` | Persistent (surreal-memory) |

### 7.2 How Other Agents Query, Extend, and Cite

**Query:**
- `research_query_graph(research_id, entity_type, relation)` → returns subgraph
- `research_query_claims(research_id, topic, min_confidence)` → returns verified claims
- `research_query_sources(research_id, claim_id)` → returns raw evidence

**Extend:**
- `research_extend(research_id, new_evidence)` → appends to package, updates graph, re-runs verification on affected claims
- `research_fork(research_id, new_query)` → creates a new research package seeded from the existing graph

**Cite:**
- `research_cite(research_id, format="apa")` → returns formatted citations for the package or a specific claim
- Agents include `research_id` and `claim_id` in their own output metadata for provenance tracking

### 7.3 Inter-Agent Protocol

```json
{
  "protocol": "prometheus-research-v1",
  "message_type": "KNOWLEDGE_OBJECT",
  "research_id": "research-abc123",
  "object_type": "CLAIM", // or GRAPH_FRAGMENT, CITATION, CONTRADICTION
  "payload": { ... },
  "provenance": {
    "created_by": "prometheus-research-v1.2.0",
    "created_at": "2026-07-03T15:00:00Z",
    "content_hash": "sha256:..."
  }
}
```

### 7.4 Preventing Knowledge Entropy

As knowledge objects accumulate, quality degrades without active maintenance. The deep-research skill should integrate with **forge-rs** for:
- **Periodic re-verification:** Re-check claims against live sources; flag stale citations
- **Drift detection:** Compare new web search results against stored claims; detect when "truth" has evolved
- **Memory consolidation:** Compress old research packages into summary nodes; archive raw evidence

**Source:** ai-memory MCP server — "L1/L2 reflection, contradiction detection, auto-tagging" [https://mcpservers.org/servers/alphaonedev/ai-memory-mcp]; CASCADE — memory consolidation and retrieval [https://arxiv.org/html/2512.23880v1]

---

## 8. Native CLI Tooling Recommendations

### 8.1 Existing Prometheus Binary Surface

The Prometheus Skill Pack already exposes six tool binaries:

| Binary | Role |
|--------|------|
| `prometheus` | Skill management, self-learning, GitOps validation, Cedar policy, sycophancy |
| `forge` | Enrichment, reflection, drift, templates, MCP server |
| `pk` / `pk-cherry` | Karpathy KB CLI / MCP bridge |
| `liter-llm` | LLM proxy + MCP tool server |
| `surreal-memory-server` | Graph memory + MCP + REST |
| `prometheus-rust-auditor` | Staged Rust quality pipeline |

**Source:** Prometheus Skill Pack — CLI & Scripts Reference [https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/guide/16-cli-and-scripts.md]

### 8.2 Proposed: `prometheus-research` Binary

A new Rust binary (`prometheus-research`) that provides the CLI surface for the deep-research skill:

```bash
# Conduct research
prometheus-research conduct "quantum computing error correction 2025"   --depth 3 --breadth 4 --output ./quantum.research

# Query existing research
prometheus-research query ./quantum.research   --question "What are the latest surface codes?"

# Extend existing research with new evidence
prometheus-research extend ./quantum.research   --query "logical qubit experiments 2026"

# Export to different formats
prometheus-research export ./quantum.research   --format pdf --output ./quantum-report.pdf

# Verify claims against live sources
prometheus-research verify ./quantum.research   --claims claim-001,claim-002

# Check for knowledge drift (re-run vs. stored)
prometheus-research drift ./quantum.research   --threshold 0.8

# Run as MCP server (SSE + stdio)
prometheus-research mcp --port 8944 --transport sse
prometheus-research mcp --transport stdio

# Show real-time research progress (AG-UI stream)
prometheus-research watch --research-id research-abc123

# List research packages in knowledge base
prometheus-research list --kb ~/.prometheus/knowledge/

# Fork research into new investigation
prometheus-research fork ./quantum.research   --query "superconducting qubits specifically"
```

### 8.3 CLI Design Principles

Following the Prometheus pack's existing patterns:
- **Subcommand-based:** `prometheus-research <verb> <args>`
- **MCP dual-mode:** Supports both `--transport stdio` (for Claude/Codex/etc.) and `--transport sse` (for web/harness integration)
- **Config file:** `~/.prometheus/research.yaml` for default depth, breadth, model preferences, retriever selection
- **Environment variables:** `PROMETHEUS_RESEARCH_DEPTH`, `PROMETHEUS_RESEARCH_MODEL`, etc.
- **Progress signals:** JSON progress events to stderr for AG-UI consumption
- **Exit codes:** 0 = success, 1 = general error, 2 = verification failed, 3 = contradiction detected, 4 = network/search failure

### 8.4 NPM Script Integration

As with other skills, the deep-research skill should expose npm scripts for validation and installation:

```json
{
  "scripts": {
    "validate:research": "node scripts/validate-research-skill.js",
    "install:research": "scripts/install-binaries.sh prometheus-research",
    "research:smoke": "prometheus-research --version && prometheus-research conduct --dry-run "test""
  }
}
```

---

## 9. Comparative Analysis Matrix

### 9.1 Deep Research Projects Comparison

| Project | Output Format | Knowledge Asset? | MCP Support | Model Independence | Verifier Loop | Graph Output | Open Source |
|---------|-------------|------------------|-------------|-------------------|---------------|--------------|-------------|
| **GPT Researcher** | Markdown, PDF, DOCX | ❌ (disposable) | ✅ (client + server) | ✅ (any LLM) | ✅ (reviewer/reviser) | ❌ | ✅ Apache-2 |
| **LangGraph Open Deep Research** | Markdown | ❌ (disposable) | ✅ (native) | ✅ (init_chat_model) | ✅ (reflection) | ❌ | ✅ MIT |
| **OpenResearcher** | Academic paper | ❌ (static) | ❌ | ❌ | ✅ (CoT) | ❌ | ✅ |
| **Together AI Open Deep Research** | Markdown | ❌ | ❌ | ❌ (Together only) | ✅ | ❌ | ✅ |
| **MiroThinker** | Prediction + plan | ❌ | ❌ | ❌ | ✅ (long-horizon) | ❌ | Partial |
| **AgentForge** | Verified response | ⚠️ (trace only) | ❌ | ✅ (LiteLLM) | ✅ (Ragas) | ❌ | ✅ |
| **CASCADE** | Code + docs | ⚠️ (memory graph) | ✅ (4 servers) | ✅ | ✅ | ✅ (Neo4j) | ✅ |
| **Proposed Prometheus Research** | `.research` package | ✅ (persistent) | ✅ (native stack) | ✅ | ✅ (multi-layer) | ✅ (JSON-LD) | ✅ |

### 9.2 Skill Platform Portability Analysis

See parallel research report on **Skill Platform Specifications** for full details. Key finding: the common denominator across Claude Code, Codex, OpenCode, Cursor, Kimi, and others is:
- **Markdown-based skill definition** (`SKILL.md`)
- **MCP tool server** (stdio or SSE)
- **Configuration via frontmatter / JSON / YAML**

The Prometheus deep-research skill should therefore ship as:
1. A `SKILL.md` file with skill definition, progress signals, and usage instructions
2. A `prometheus-research` MCP server (stdio + SSE)
3. A `research/` directory with prompt templates, schemas, and example configurations

---

## 10. Recommendations for Prometheus Skill Pack

### 10.1 Immediate Actions (MVP)

1. **Define the `.research` package schema** as a JSON Schema (`.research/schema.json`) with versioning
2. **Implement `prometheus-research` CLI** in Rust with Axum, supporting SSE + stdio MCP transports
3. **Integrate with existing MCP stack:** Use tavily-mcp, sequential-thinking, liter-llm, surreal-memory, forge-rs, prometheus-knowledge, sycophancy-correction as native plugins
4. **Ship a `SKILL.md`** that exposes the skill across all 8 harnesses (Claude, Codex, OpenCode, Cursor, Windsurf, Kimi, Roo, Amp)
5. **Support AG-UI / A2UI progress streaming** for real-time research visibility

### 10.2 Medium-Term (V2)

1. **Add entity resolution service** (Wikidata, ORCID, Crunchbase APIs) to canonicalize entities across research sessions
2. **Implement knowledge drift detection** using forge-rs + periodic re-verification cron jobs
3. **Build a visual research explorer** (React + assistant-ui) for browsing `.research` packages as interactive knowledge graphs
4. **Support collaborative research:** Multiple agents can append to the same `.research` package with merge conflict resolution

### 10.3 Long-Term (V3)

1. **Federated research network:** Research packages can cite other packages (inter-package knowledge graph)
2. **Research DAO:** Community-verified research packages with reputation staking
3. **Autonomous research agent:** Self-directed research that schedules its own follow-up investigations based on drift detection

---

## 11. References & Citations

### Primary Sources

1. **GPT Researcher** — GitHub: `assafelovic/gpt-researcher` (28.1k stars, 3.8k forks). Architecture: planner + execution agents, reviewer/reviser loop, LangGraph multi-agent support. MCP client + server. Deep Research mode: recursive tree exploration, configurable depth/breadth, ~$0.40 per research, ~5 min completion. [https://github.com/assafelovic/gpt-researcher]

2. **GPT Researcher Documentation** — "Building the Ultimate Autonomous Research Agent" (2026-03-03). Details: Browser → Editor → Researcher (parallel) → Reviewer → Reviser → Writer → Publisher graph. LangGraph StateGraph with ResearchState. [https://docs.gptr.dev/blog]

3. **GPT Researcher Deep Research** — "Introducing Deep Research: The Open Source Alternative" (2025-02-26). Recursive research tree: breadth exploration + depth diving + concurrent processing + context management. Configurable `deep_research_breadth`, `deep_research_depth`, `deep_research_concurrency`. [https://docs.gptr.dev/blog]

4. **LangGraph Open Deep Research** — GitHub: `langchain-ai/open_deep_research`. Configurable, model-independent deep research agent. Supports `init_chat_model()` for any provider. Search APIs: Tavily, MCP, native web search. Evaluation: Deep Research Bench (#6 ranking, RACE 0.4344). Deployments: LangGraph Studio, LangGraph Platform, Open Agent Platform. [https://github.com/langchain-ai/open_deep_research]

5. **LangGraph Open Deep Research — Legacy Implementations** — Plan-and-Execute workflow (`legacy/graph.py`) and Multi-Agent Supervisor-Researcher (`legacy/multi_agent.py`) with MCP support. [https://github.com/langchain-ai/open_deep_research]

6. **Tetrate — MCP + RAG: When to Use Both Together** (2026-01-16). Four hybrid patterns: (1) RAG with MCP fallback, (2) MCP with RAG grounding, (3) Iterative RAG-MCP loop, (4) MCP-Enhanced RAG. "MCP and RAG operate at different layers of the AI stack and address orthogonal concerns." [https://tetrate.io/learn/ai/mcp/mcp-rag-when-to-use-both-together]

7. **AgentForge** — GitHub: `omkarbhad/agentforge` (2026-03-05). Multi-agent research pipeline: Planner → Retriever (hybrid Qdrant + web) → Generator (LiteLLM) → Verifier (faithfulness + retry loop). Memory: mem0. Evaluation: Ragas. [https://github.com/omkarbhad/agentforge]

8. **CASCADE** — arXiv:2512.23880v1 (2025). Uses four MCP servers: Tavily (search), Memory (Neo4j + Supabase vector), Research (code intelligence + RAG), Code Execution. Memory server: dual-store (vector + graph), `save_to_memory` / `search_memory` tools. [https://arxiv.org/html/2512.23880v1]

9. **GraphAware — LLMs for Knowledge Graph: GPT Prompt Engineering** (2023-10-24). Two-phase prompt design: entity/relationship extraction → elaboration. JSON format for rich properties. Array-like format for human interpretation. [https://graphaware.com/blog/episode-2-gpt-prompt-engineering/]

10. **Neo4j — Context Engineering in AI Agents** (2026-06-04). Memory management: short-term scratchpads, long-term structured facts with citations, compression into decision summaries. Tool calls as context authoring: "Return small, typed records instead of raw text." [https://neo4j.com/blog/agentic-ai/what-is-context-engineering/]

11. **ZBrain — Knowledge Graphs for Agentic AI** (2026-05-05). GraphRAG benefits: factual accuracy, efficiency (fewer tokens), transparency (path-based evidence), multi-hop reasoning. "KG acts as a reasoning scaffold." [https://zbrain.ai/knowledge-graphs-for-agentic-ai/]

12. **LongTracer** — GitHub: `ENDEVSOLS/LongTracer` (2026-05-19). Detect hallucinations in LLM responses. Verify every claim against source documents using hybrid STS + NLI. Framework adapters: LangChain, LlamaIndex, Haystack, LangGraph. [https://github.com/ENDEVSOLS/LongTracer]

13. **Prometheus Skill Pack — CLI & Scripts Reference** (docs/guide/16-cli-and-scripts.md). Binary CLIs: `prometheus`, `forge`, `pk`/`pk-cherry`, `liter-llm`, `surreal-memory-server`, `prometheus-rust-auditor`. MCP port table: surreal-memory (23001), prometheus-knowledge (8942), forge-rs (8943), stdio servers (sycophancy-correction, liter-llm, sequential-thinking, tavily, firecrawl). [https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/guide/16-cli-and-scripts.md]

14. **ai-memory MCP Server** — mcpservers.org (2026-05-09). Rust binary (tokio + axum). Four tiers: Keyword, Semantic, Smart, Autonomous. Features: recursive learning (L1/L2 reflection), contradiction detection, auto-tagging, 6-factor scoring. 91 HTTP routes, 83 CLI subcommands. [https://mcpservers.org/servers/alphaonedev/ai-memory-mcp]

15. **Prospectus-AI** — GitHub: `GuangzhiSu/Prospectus-AI` (2026-06-25). Agent2 LangGraph pipeline: Retriever → Section Planner → Section Writer → Verifier → Revision → Assembler. Hybrid mode: semantic + fact filtering. Pluggable retriever architecture. [https://github.com/GuangzhiSu/Prospectus-AI]

16. **Omni-Agent** — GitHub: `Idk507/omni-agent`. Full architecture: 15 agents, 20 middleware, 10 MCP servers, skill system, plugin system, task engine, hook registry. Research-specific: `research_agent`, `web_search_agent`, `reflection_agent`, `consensus_agent`, `citation_store_server`. [https://github.com/Idk507/omni-agent]

17. **JSON-RAG / KG v5.2.1** — GitHub: `shihentsou/json-rag`. Knowledge graph engine for structured data. FTS5 (exact, <1ms) + Graph traversal (<10ms) + Vector (optional, 50-200ms). Bitemporal support. JSON/JSON-LD output. [https://github.com/shihentsou/json-rag]

18. **Claude Code skill: JSON-LD Knowledge Graph** — GitHub: `shimo4228/claude-skill-jsonld-knowledge-graph` (2026-05-16). Ships `graph.jsonld` next to `llms.txt`. Schema.org-compatible triples. Machine-readable structure for LLMs. [https://github.com/shimo4228/claude-skill-jsonld-knowledge-graph]

19. **Open Deep Research Bench** — Evaluation benchmark for deep research agents. 100 PhD-level tasks (50 English, 50 Chinese), 22 fields. RACE score via LLM-as-judge (Gemini). [Referenced in LangGraph Open Deep Research README]

20. **MCP-CLI** — GitHub: `P.Schmid/mcp-cli` (2026). Lightweight CLI to interact with MCP servers. Cited in "Efficient Reinforcement Finetuning for Large Toolspaces" (2026-01-26). [https://github.com/P.Schmid/mcp-cli]

### Secondary / Background Sources

21. **Chain of Ideas** — arXiv:2410.13185 (2024-10-24). Baseline comparison: GPT-Researcher enhanced with plan-and-solve and RAG. [https://arxiv.org/html/2410.13185v3]

22. **Chatbots to Knowledge Agents** — TechRxiv (2025). Architectural validation: retrieval layer, memory layer (episodic + evidence + semantic), reasoning layer, evidence evaluation, control loop. [https://www.techrxiv.org/users/1009916/articles/1375984/master/file/data/Chatbots to Knowledge Agents/Chatbots to Knowledge Agents.pdf]

23. **ArXiv:2605.15184** — "Is Grep All You Need? How Agent Harnesses Reshape Agentic Search" (2026-05-14). Comparison of custom harnesses vs. provider-native CLI agents (Claude Code, Codex, Gemini CLI). [https://arxiv.org/html/2605.15184v1]

24. **ArXiv:2602.14690** — "Configuring Agentic AI Coding Tools: An Exploratory Study" (2026-03-28). Eight configuration mechanisms: Context Files, Skills, MCP, Subagents, Rules, Prompts, Environment, Workflows. [https://arxiv.org/html/2602.14690v1]

25. **CLI Tools Boom 2025-2026** — CNBlogs (2026-04-02). Key data: Claude Code (103k stars), Gemini CLI (100k), Codex (72k), Aider (43k). "MCP servers are essentially CLI tool capabilities packaged as AI-callable functions." [https://www.cnblogs.com/qiniushanghai/p/19812984]

---

*End of Report*

*Report compiled: 2026-07-03 CDT*
*Research scope: Knowledge Assets & Architecture Patterns for Prometheus Deep Research Skill*
