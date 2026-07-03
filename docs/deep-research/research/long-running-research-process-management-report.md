# Long-Running Research Process Management

## Structured Research Report for the Prometheus Deep-Research Skill Specification

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Scope:** Foundational research for enhancing the "deep-research" skill specification for the Prometheus Skill Pack  
**Sources:** 40+ web searches, arXiv papers, GitHub repositories, official documentation, and industry analysis (2024–2026).

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [1. Managing Long-Running Research Without Losing Context](#1-managing-long-running-research-without-losing-context)
3. [2. Checkpointing and Resumption Patterns](#2-checkpointing-and-resumption-patterns)
4. [3. Preventing Context Window Degradation](#3-preventing-context-window-degradation)
5. [4. Accuracy Preservation Techniques for Multi-Step Research](#4-accuracy-preservation-techniques-for-multi-step-research)
6. [5. Durable Execution Frameworks: Temporal, LangGraph, and Beyond](#5-durable-execution-frameworks-temporal-langgraph-and-beyond)
7. [6. KBD Orchestrator Waypoint/Progress Pattern Applied to Research](#6-kbd-orchestrator-waypointprogress-pattern-applied-to-research)
8. [7. Multi-Session and Multi-Day Research Handling](#7-multi-session-and-multi-day-research-handling)
9. [8. Research as a Background Process](#8-research-as-a-background-process)
10. [9. Maintaining Research Quality Over Long Durations](#9-maintaining-research-quality-over-long-durations)
11. [10. The Karpathy Loop Applied to Long-Running Research](#10-the-karpathy-loop-applied-to-long-running-research)
12. [11. Handling Partial Failures in Long Research Pipelines](#11-handling-partial-failures-in-long-research-pipelines)
13. [Cross-Cutting Insights & Strategic Recommendations](#cross-cutting-insights--strategic-recommendations)
14. [References](#references)

---

## Executive Summary

Long-running research processes—spanning hours to days—are the frontier of agentic AI in 2025–2026. The dominant paradigm has shifted from "one long conversation" to **recoverable, durable workflows** where the workspace, not the model's context window, is the continuity layer. Key architectural patterns emerging across Google, Anthropic, OpenAI, and the open-source community include:

- **Checkpointing & time-travel**: LangGraph's immutable checkpoint chains, Temporal's durable execution, and Google's ADK persistent session storage enable crash recovery and human-in-the-loop pauses.
- **Fresh-session loops**: The "Ralph Loop" and Anthropic's initializer-plus-worker pattern demonstrate that restarting with clean context windows and rehydrating from disk artifacts is superior to trying to maintain one giant session.
- **Hierarchical memory**: Progressive summarization, hybrid memory layers (Redis/vector/SQL), and external knowledge graphs prevent context degradation while preserving semantic access.
- **Verification at every step**: Multi-agent verification (LLM-as-Judge, Reflexion, adversarial debate, process verification) prevents silent error accumulation that compounds across multi-step research pipelines.
- **Graceful degradation**: Circuit breakers, fallback chains, and tiered degradation ensure research pipelines deliver partial value rather than total failure when APIs, sources, or tools become unavailable.
- **Drift detection & re-verification**: Stale source flagging, confidence decay models, and periodic knowledge audits keep long-duration research from becoming obsolete before completion.

For Prometheus, these findings map directly onto the existing ecosystem: the **KBD orchestrator's `current-waypoint.json`** pattern provides a natural research-progress tracking substrate; the **Karpathy Loop** (focus→reflect→ingest) provides a quality-maintenance ritual for long-running research; and the **Feynman Learning Loop** can validate and deepen research findings through recursive explanation and gap identification.

---

## 1. Managing Long-Running Research Without Losing Context

### The Core Problem: Handoff Across Amnesia

The fundamental challenge of long-running research is not the length of the conversation but the **handoff across amnesia**—the model's inability to natively persist across context windows, sandboxes, process crashes, or days of work.[^1] [^2] A fresh session is born with no knowledge of what the previous session tried, which sources failed, which claims were verified, or which sub-questions remain open.

### The Architecture That Won: Durable Artifacts as Continuity Layer

The consensus architecture across Anthropic, Google, Cursor, and the open-source community is remarkably consistent: **the durable artifacts are the real continuity layer**—not the model's context window.[^2] The model is one worker inside a recoverable workflow. The workspace remembers.

```
LONG RUNNING RESEARCH HARNESS
=============================

initializer session
  v
creates durable research state:
  - research_spec.json        (topic, scope, deliverables, constraints)
  - research_plan.json        (sub-questions, assigned priorities, dependencies)
  - progress_log.md           (chronological: what was tried, what worked, what failed)
  - evidence_bank/            (retrieved documents, excerpts, source metadata)
  - knowledge_graph.json      (entities, relationships, claims, confidence scores)
  - verification_log.json     (which claims were checked, by whom, against what)
  - current-waypoint.json     (KBD-style phase/stage tracking)
  v
worker session 1 starts fresh
  v
reads durable state → does one bounded research task → verifies → logs → exits
  v
worker session 2 starts fresh
  v
reads durable state → continues
  v
...
  v
judge / evaluator decides whether research goal is actually met
```

This pattern is variously called:
- **The Ralph Loop** (Geoffrey Huntley, Ryan Carson): a bash loop feeding a prompt file to the agent, with a plan on disk as shared state.[^1]
- **Anthropic's initializer + repeated workers**: an initializer agent creates the environment (spec, plan, progress file, git baseline), and subsequent workers read state, pick one task, implement, verify, update, and exit.[^2]
- **Google's Agent Development Kit (ADK) sessions**: `DatabaseSessionService` persists agent state across container restarts, scale-to-zero, and unexpected failures.[^3]

### Key Insight: Fresh Sessions Beat Giant Sessions

Research consistently shows that **a fresh context window reading good state from disk is better than a stale context window carrying hours of tool output**.[^2] Restarting is not giving up—it is garbage collection. The loop can be "dumb" if the workspace is smart.[^1]

### Research-Specific Context Requirements

For deep research specifically, the durable state must include:

| State Type | Contents | Storage Format |
|---|---|---|
| **Intent State** | Research question, scope boundaries, deliverable format, depth target | `research_spec.json` |
| **Progress State** | Sub-questions completed, evidence gathered, gaps identified, next priorities | `progress_log.md` + `research_plan.json` |
| **Environment State** | Search APIs configured, source credibility tiers, tool access credentials | `research_config.json` |
| **Evidence State** | Retrieved documents, excerpts, source metadata, retrieval timestamps | `evidence_bank/` (directory of JSON + raw) |
| **Synthesis State** | Claims extracted, confidence scores, contradictions flagged, resolutions | `knowledge_graph.json` |
| **Recovery State** | Last known good checkpoint, failed paths, alternative sources tried | `recovery_log.json` |
| **Verification State** | Which claims were verified, against which sources, verification method | `verification_log.json` |

[^1]: https://addyosmani.com/blog/long-running-agents/ (Addy Osmani, "Long-running Agents", 2026)
[^2]: https://nicolasbustamante.com/blog/long-running-agent-engineering (Nicolas Bustamante, "Long Running Agent Engineering", 2026)
[^3]: https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/ (Google, "Build Long-running AI agents with ADK", 2026)

---

## 2. Checkpointing and Resumption Patterns

### 2.1 Complete State Snapshots vs. Incremental Persistence

Two primary checkpointing approaches exist for research agents:

| Approach | Description | Best For | Trade-off |
|---|---|---|---|
| **Complete State Snapshots** | Save full agent state, context, intermediate data, and system state at each meaningful step | Audit trails, regulatory contexts, human-in-the-loop | Storage cost, serialization overhead |
| **Clean Breakpoints** | Predefined checkpoints only; prevent mid-operation interruptions | Deterministic, phase-gated research workflows | Less flexible, potential re-work between checkpoints |

For long-running research (>4 hours), systems without state persistence have a **90% higher risk of total task failure** due to API timeouts or infrastructure disruptions.[^4]

### 2.2 LangGraph Checkpointing: The Industry Standard

LangGraph's persistence layer has become the reference implementation for agent checkpointing. It automatically saves a snapshot of the **full graph state** after every execution step, creating an immutable chain of checkpoints organized by `thread_id`.[^5] [^6]

**Key capabilities:**
- **Fault tolerance**: If a node fails at step 8 of a 10-step workflow, only step 8 is retried—not steps 1–7.[^5]
- **Human-in-the-loop**: The workflow pauses, checkpoints state, and waits. Hours later, a new process loads the checkpoint and resumes exactly where it left off.[^5]
- **Time-travel debugging**: Navigate backward and forward through execution history, inspect state at any point, and **fork** from a past checkpoint with a correction injected.[^6]
- **Checkpoint forking**: When a stakeholder discovers an error (e.g., outdated 2024 benchmarks presented as 2026 insights), developers can rewind to the checkpoint where the bad data was introduced, inject a correction, and branch forward without discarding prior work.[^6]

**Production recommendation:** Use `MemorySaver` for development, but immediately plan migration to `PostgresSaver` (or `SqliteSaver` for simpler deployments) for production.[^7]

### 2.3 Temporal Workflows: Durable Execution for Stochastic Programs

Temporal is the gold standard for deterministic durable execution, but traditional workflow engines assume fixed control flow. The key innovation in 2026 is the **mapping of agent loops onto Temporal workflows**: the agent loop becomes a workflow, and each model call and tool call becomes a durable activity.[^8]

On a crash, completed model calls and tool invocations replay from the log rather than re-executing, so you do not re-pay tokens or re-fire side effects.[^8] Notable integrations:
- **Temporal + OpenAI Agents SDK**: General availability in early 2026, wrapping each agent invocation and tool call as a durable activity.[^8]
- **Temporal + Google ADK**: Experimental, rerouting LLM calls through activities with minimal code change.[^8]
- **DuraLang**: A missing durability layer for LangChain that adds a `@dura` decorator to make every LLM call, tool call, and agent-to-agent call individually recoverable, automatically retried, and fully observable through Temporal—without rewriting code into graphs.[^8]

### 2.4 The Honest Tension: Checkpointing vs. Full Durable Execution

Framework-native checkpointing (LangGraph) recovers **state**. A full durable-execution engine (Temporal) additionally provides exactly-once side effects, durable timers, signals, and replay semantics across deploys.[^8] The gap matters most when tool calls have irreversible external effects (e.g., sending emails, posting to systems). For research agents that are mostly LLM reasoning with recoverable, idempotent tools, **framework checkpointing plus idempotency keys on non-idempotent tools is often sufficient**.

### 2.5 Checkpointing Pattern for Prometheus Research

```json
{
  "checkpoint_id": "chk-2026-07-03T16-00-00Z",
  "thread_id": "research-uuid-1234",
  "timestamp": "2026-07-03T16:00:00Z",
  "phase": "retrieve",
  "stage": "evidence_collection",
  "state": {
    "research_spec": { ... },
    "completed_sub_questions": ["sq-1", "sq-2"],
    "current_sub_question": "sq-3",
    "evidence_bank": { ... },
    "knowledge_graph": { ... },
    "context_summary": "Progressive summary of findings so far..."
  },
  "next_steps": ["retrieve_source_X", "verify_claim_Y"],
  "metadata": {
    "api_calls_made": 47,
    "tokens_consumed": 152340,
    "sources_retrieved": 23,
    "last_verified": "2026-07-03T15:55:00Z"
  }
}
```

[^4]: https://www.indium.tech/blog/7-state-persistence-strategies-ai-agents-2026/ (Indium, "7 State Persistence Strategies for Long-Running AI Agents", 2026)
[^5]: https://www.autolearningagents.com/langgraph/checkpointing.php (AutoLearningAgents, "LangGraph Checkpointing and Time-Travel Debugging", 2026)
[^6]: https://pub.towardsai.net/deepagents-on-langgraph-debugging-long-running-ai-agents-with-time-travel-ff897ef50b73 (DeepAgents on LangGraph, 2026)
[^7]: https://eastondev.com/blog/en/posts/ai/20260424-langgraph-agent-architecture/ (Easton Dev, "LangGraph State Management", 2026)
[^8]: https://github.com/ombharatiya/ai-system-design-guide/blob/main/07-agentic-systems/11-durable-execution.md (AI System Design Guide, "Durable Execution", 2025)

---

## 3. Preventing Context Window Degradation

### 3.1 The Problem: Context Rot and Attention Diffusion

Long context windows (even 200K tokens) do not solve the problem—information still degrades. Research shows that LLMs struggle with **multi-turn dependencies, entity tracking, and coherence** in long narratives, and that attention naturally diffuses as irrelevant details accumulate.[^9] The problem is not insufficient context window size; it is **competing information that dilutes agent attention over time**.[^4]

### 3.2 Progressive Summarization: The Core Technique

Progressive summarization is the dominant technique for managing context in long-running research. Inspired by Tiago Forte's "Second Brain" methodology, it operates through iterative compression:

1. **Layer 1**: Raw highlights/excerpts from sources (full fidelity, high volume)
2. **Layer 2**: Bolded main ideas from Layer 1 (filtered relevance)
3. **Layer 3**: Highlighted top insights from Layer 2 (maximum compression)
4. **Layer 4**: Synthesized claims with confidence scores and source attribution
5. **Layer 5**: Knowledge graph entities and relationships (semantic abstraction)

For LLM agents, this maps to **summarization-based context management**: periodically compress the tool-using history by LLM-generated summaries that retain task-relevant information, keeping a compact context while enabling the agent to scale beyond the fixed context window.[^10] Each action generation is based on (i) the most recent summarization, and (ii) context accumulated after that summary.

### 3.3 Hierarchical Memory Architecture

Production systems use a **three-layer hybrid memory** architecture:[^4]

| Layer | Technology | Contents | Lifespan | Retrieval Speed |
|---|---|---|---|---|
| **Short-term / Working** | Redis / in-memory | Current conversation context, active tasks, recent tool outputs | Minutes to hours | <1ms |
| **Long-term / Semantic** | Vector DB (pgvector, Pinecone, Qdrant) | Summarized past interactions, learned patterns, entity embeddings | Indefinite | 5–50ms |
| **Durable / Audit** | SQL / Object Storage | Complete run history, compliance records, immutable logs | Permanent (cold) | 100ms+ |

For research specifically, the **vector database** stores semantic embeddings of claims, evidence, and sources, enabling the agent to retrieve relevant prior work by meaning rather than keywords.[^4]

### 3.4 View Compilation: Context Reconstruction on Resume

When a research agent resumes after a long pause, it must reconstruct its context from the durable state. **View compilation** converts raw execution logs into structured, summarized insights that are injected into the agent's working context.[^4] This is analogous to the KBD orchestrator's `position-reminder.txt`—a concise orientation document that tells a fresh worker where the project stands.

### 3.5 Research-Specific Context Management

For the Prometheus deep-research pipeline, context management should follow this pattern:

```
RESEARCH CONTEXT LAYERS
=======================

L0: Raw Session Context (ephemeral, current turn only)
  - Active search results
  - Current reasoning chain
  - Live tool outputs

L1: Working Context (survives one session, rehydrated each time)
  - Progressive summary of findings
  - Open sub-questions and their priorities
  - Current knowledge graph subgraph being investigated
  - Recent verification results

L2: Persistent Knowledge (survives across sessions, in SurrealDB/Postgres)
  - Full knowledge graph
  - Complete evidence bank with embeddings
  - Citation registry
  - Verification history and confidence scores
  - Research plan and progress tracking

L3: Archive / Audit (cold storage, compliance)
  - Full checkpoint history
  - Raw tool outputs and API traces
  - Human review decisions
```

[^9]: https://arxiv.org/html/2505.24575v1 (NexusSum: Hierarchical LLM Agents for Long-Form Narrative Summarization, 2025)
[^10]: https://openreview.net/attachment?id=azy6Tjy3QH&name=pdf (SUPO: Summarization-based Policy Optimization, 2025)

---

## 4. Accuracy Preservation Techniques for Multi-Step Research

### 4.1 The Silent Failure Mode: Error Accumulation

The defining failure mode of production agentic systems in 2026 is **error accumulation**: a multi-agent pipeline passes every demo but silently accumulates three bad decisions by step four, producing a confidently, fluently wrong final output.[^11] There is no signal to catch it—just a clean-looking result at the end.

### 4.2 Four Verification Architectures

Research has identified four distinct verification architectures, each with specific failure modes:[^11]

| Architecture | How It Works | Failure Mode | Best For |
|---|---|---|---|
| **Output Scoring (LLM-as-Judge)** | A second model scores the final output against rubrics | Misses compounding errors that originated mid-pipeline | Cheap, fast sanity checks |
| **Reflexion Loops** | The solver reflects on its own output and retries | Does not converge on hard problems; burns retries without improvement | Well-defined tasks with clear success criteria |
| **Adversarial Debate** | Two models argue opposing positions; a judge decides | Expensive; requires sophisticated judge infrastructure | High-stakes, ambiguous reasoning |
| **Process Verification** | Verify each step, not just the terminal state | Higher latency and cost; more infrastructure | Extended research workflows where root causes matter |

**Key insight:** The solver and verifier should be **different models** (or at least different model families). Shared training data, RLHF preferences, and systematic gaps undermine independence. For highest-stakes research, use models from different providers for solver and judge.[^11]

### 4.3 Chain-of-Thought Logging and Faithfulness

Chain-of-thought (CoT) reasoning traces are **not authoritative transcripts** of the model's computation. Research consistently shows that models can change their answers based on factors (biased prompts, sycophantic pressure) that never appear in the stated reasoning trace.[^12] Extended-thinking models do not solve this—their traces are better described as scratchpads that influence output rather than faithful records.[^12]

**Practical implication for research:** Use CoT to improve model performance, but rely on **behavioral testing and external verification**—not trace inspection—for quality control. Build explicit verification steps, multi-model checks, constrained output formats that make errors detectable, and human review gates at the right points.[^12]

### 4.4 Confidence Decay Models

As research progresses, claims accumulate and their confidence can drift. The **confidence decay** approach reduces the confidence of unverified memories over time:[^13]

- Memories not verified within a configurable threshold (default 90 days) have their confidence reduced by a decay rate per sweep.
- The verification status transitions: `unverified → stale → contradicted`.
- Batch verification checks whether source files still exist and whether their content still matches the memory's source hash.

This prevents the research agent from treating old, unverified claims as equally trustworthy as recently verified ones.

### 4.5 Prometheus Accuracy Preservation Stack

```
VERIFICATION AT EACH RESEARCH STEP
===================================

Search Step:
  - Source credibility scoring (domain authority, recency, citation count)
  - Retrieval relevance verification (was the right document found?)
  - Duplicate detection (have we seen this source before?)

Extraction Step:
  - Claim-source alignment (does the excerpt actually support the claim?)
  - OCR/parse quality check (was the document correctly extracted?)

Synthesis Step:
  - Multi-model cross-check (different model verifies the synthesis)
  - Contradiction detection (does this conflict with prior findings?)
  - Confidence scoring (how strong is the evidence?)

Report Step:
  - Citation traceability (can every claim be traced to a source?)
  - Factual drift check (have sources changed since retrieval?)
  - Human review gate for high-stakes claims
```

[^11]: https://pub.towardsai.net/how-multi-agent-self-verification-actually-works-and-why-it-changes-everything-for-production-ai-71923df63d01 ("How Multi-Agent Self-Verification Actually Works", 2026)
[^12]: https://www.mindstudio.ai/blog/what-is-chain-of-thought-faithfulness-ai-reasoning ("What Is Chain-of-Thought Faithfulness?", 2026)
[^13]: https://github.com/Haustorium12/memory-v3 (Memory-v3: HaluMem Source Verification & Confidence Decay, 2026)

---

## 5. Durable Execution Frameworks: Temporal, LangGraph, and Beyond

### 5.1 Comparative Framework Analysis

| Framework | Checkpoint Model | Durability Level | Best For | Key Limitation |
|---|---|---|---|---|
| **LangGraph** | Graph-state snapshots at each super-step | State recovery, time-travel, human-in-the-loop | Research workflows with branching, cycles, and review gates | Does not guarantee exactly-once side effects |
| **Temporal** | Activity replay log; workflow state machine | Exactly-once execution, durable timers, signals, replay across deploys | Complex, multi-day workflows with external commitments and payments | High ceremony; requires workflow/activity decomposition |
| **Google ADK** | DatabaseSessionService (SQLite/Cloud SQL) | Session persistence across container restarts and scale-to-zero | Google Cloud-native agent deployments; human-in-the-loop | Tied to Google ecosystem |
| **Restate / DBOS** | Library-level durable workflows | Low-code durability for agent frameworks | Teams wanting durability without workflow restructuring | Newer, less ecosystem maturity |
| **DuraLang** | `@dura` decorator on LangChain code | Temporal-backed durability without graph rewrite | Existing LangChain codebases needing durability | Adds dependency on Temporal infrastructure |

### 5.2 The Durable Execution → Agent Loop Mapping

The canonical mapping for research agents is:[^8]

```
AGENT LOOP          →    TEMPORAL / DURABLE FRAMEWORK
=======================================================

research goal       →    workflow definition (deterministic)
planning step       →    activity (orchestrator node)
search tool call    →    durable activity (idempotent, retryable)
source retrieval    →    durable activity
claim extraction    →    durable activity
synthesis step      →    durable activity
verification step   →    durable activity (with human signal gate)
report generation   →    durable activity

CRASH → replay from last completed activity, not from beginning
HUMAN REVIEW → signal into workflow, pause with no compute cost
RESUME → load checkpoint, continue from next activity
```

### 5.3 LangGraph's Production Architecture

For self-hosted deployment, the recommended stack is:[^7]
- **LangGraph Server** (managed) or self-hosted Node.js server with compiled graph
- **PostgresSaver** for persistence backend
- **LangSmith** for observability (execution traces, state inspection, runtime monitoring)
- **Thread IDs** for isolating research runs

The honest trade-off: LangGraph is not free. The graph mental model requires team buy-in, and the LangChain ecosystem adds bundle weight. Reserve LangGraph for workflows that genuinely need **durability, branching, and auditability**—which deep research does.[^7]

[^7]: https://letsbuildsolutions.com/blog/ai-ml/langgraph-in-production-stateful-agent-graphs-conditional-edges-and-checkpointing-for-reliable-multi-step-ai-workflows/ ("LangGraph in Production", 2026)
[^8]: https://github.com/ombharatiya/ai-system-design-guide/blob/main/07-agentic-systems/11-durable-execution.md (AI System Design Guide, "Durable Execution", 2025)

---

## 6. KBD Orchestrator Waypoint/Progress Pattern Applied to Research

### 6.1 The Existing KBD Pattern

The Prometheus KBD orchestrator already uses a structured waypoint/progress system:

```json
{
  "phase": "phase-ci-cross-model-qa-and-hardening",
  "previousPhase": "phase-ci-all-green",
  "status": "reflect_complete",
  "stage": "reflect",
  "changes_completed": 3,
  "changes_total": 3,
  "goals_met": 3,
  "goals_total": 3,
  "sycophancy_gate": 0.0,
  "next_command": "phase CLOSED",
  "currentTask": "phase CLOSED — 3/3 goals MET...",
  "updatedAt": "2026-07-03T20:10:00Z"
}
```

This pattern maps directly onto research process management with minimal adaptation.

### 6.2 Research-Adapted Waypoint Schema

```json
{
  "phase": "research-evidence-collection",
  "previousPhase": "research-planning",
  "status": "in_progress",
  "stage": "verify",
  "changes_completed": 7,
  "changes_total": 10,
  "goals_met": 4,
  "goals_total": 6,
  "quality_gate": 0.85,
  "next_command": "continue evidence_collection → verify sub-question sq-8",
  "exactNextCommand": "invoke verifier on sq-8 claims",
  "recommendedNextPhase": "research-synthesis",
  "currentTask": "Verifying claims from sq-8 against primary sources...",
  "research_metadata": {
    "sub_questions": {
      "completed": ["sq-1", "sq-2", "sq-3", "sq-4", "sq-5", "sq-6", "sq-7"],
      "in_progress": "sq-8",
      "pending": ["sq-9", "sq-10"]
    },
    "evidence_stats": {
      "sources_retrieved": 47,
      "claims_extracted": 156,
      "claims_verified": 89,
      "contradictions_flagged": 12,
      "contradictions_resolved": 8
    },
    "session_metrics": {
      "api_calls": 124,
      "tokens_consumed": 487000,
      "wall_time_hours": 3.2,
      "estimated_remaining_hours": 1.8
    }
  },
  "updatedAt": "2026-07-03T20:10:00Z"
}
```

### 6.3 Phase-Based Research Pipeline

The KBD orchestrator's four nested layers (L0 harness micro-loop, L1 tactical KBD loop, L2 strategic evolver loop, L3 outer standing loop) map to research as follows:

| KBD Layer | Research Analog | Responsibility |
|---|---|---|
| **L0: Harness Micro-Loop** | Single search-retrieve-extract iteration | Execute one bounded research task, verify, log, exit |
| **L1: Tactical KBD Loop** | One sub-question investigation (plan→search→collect→verify) | Orchestrate the 10-stage deep-research pipeline for one sub-question |
| **L2: Strategic Evolver Loop** | Full research topic synthesis (merge sub-questions, resolve contradictions) | Cross-sub-question integration, contradiction resolution, knowledge graph merging |
| **L3: Outer Standing Loop** | Meta-research evaluation (is the research complete? Is it accurate? What gaps remain?) | Quality gate, human review, research scope adjustment, recursion decision |

Each layer's `current-waypoint.json` acts as a checkpoint, enabling resumption at any granularity.

---

## 7. Multi-Session and Multi-Day Research Handling

### 7.1 Session Stitching

When research spans multiple sessions or days, the core challenge is **session stitching**: ensuring that session N+1 understands the state left by session N without losing information or duplicating work.[^1]

The pattern is straightforward but disciplined:
1. **Session close ritual**: Before exiting, the agent writes a structured handoff document (`session_close.md`) containing: what was accomplished, what remains open, what failed and why, what the next priority is, and any warnings or context for the next session.
2. **State hydration**: The next session reads the handoff document, the current waypoint, the progress log, and the knowledge graph to reconstruct context.
3. **Overlap verification**: The new session verifies that its understanding matches the durable state by checking a few key claims or counts before proceeding.

### 7.2 Incremental Report Building

Rather than generating the final report only at the end, research should build **incrementally**:

- Each completed sub-question appends its synthesis to a growing `report_draft.md`.
- The draft includes marker comments indicating which sections are finalized and which are placeholders.
- The knowledge graph is updated in place, not rebuilt from scratch.
- Citations are accumulated in a `citations.json` registry, not recollected at report time.

This approach ensures that if the research is interrupted at 80% completion, the 80% is already usable—not locked inside an intermediate state that requires final synthesis to extract.

### 7.3 Time-Aware Source Handling

For multi-day research, **source freshness** becomes a concern. Sources retrieved on day 1 may be stale by day 3 if the topic is rapidly evolving (e.g., breaking news, stock prices, regulatory announcements). The research pipeline should:

- Tag each source with `retrieved_at` timestamp.
- Flag sources older than a configurable threshold for re-verification.
- Re-query high-priority sources before final synthesis if the research span exceeds the topic's velocity half-life.

### 7.4 Overnight and Long-Duration Patterns

Google's ADK documentation explicitly addresses this pattern: **sleep through the idle time, and wake up exactly where you left off**.[^3] In a containerized environment, containers cold-start and scale to zero during idle periods. With persistent session storage (SQLite locally, Cloud SQL in production), every in-flight research run survives server restarts.

For Prometheus, this maps to:
- SurrealDB or Postgres as the persistent state layer.
- Cron-triggered or event-driven wake-ups for scheduled research continuation.
- ElectricSQL sync to ensure the local Tauri app has the latest research state even after offline periods.

[^1]: https://addyosmani.com/blog/long-running-agents/ (Addy Osmani, "Long-running Agents", 2026)
[^3]: https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/ (Google, "Build Long-running AI agents with ADK", 2026)

---

## 8. Research as a Background Process

### 8.1 Trigger Patterns

Long-running research can be initiated and continued through three primary trigger patterns:[^14]

| Pattern | Trigger | Best For | Implementation |
|---|---|---|---|
| **Cron-based** | Time-based schedule (e.g., `0 2 * * *` for daily at 2 AM) | Periodic intelligence briefs, recurring competitive monitoring, daily report generation | Standard cron expressions; agent evaluates what to research based on schedule |
| **Event-driven** | External system event (webhook, file upload, database change) | Reactive research triggered by news, PR merges, customer tickets, market events | Webhook endpoint → event queue → agent session spawn |
| **Hybrid** | Cron + events + human initiation | Comprehensive research operations where any input source can trigger continuation | Unified trigger registry; cron tick is just another event |

### 8.2 Background Execution Architecture

A production background research agent needs:

- **Persistent server**: Runs on cloud infrastructure, not the user's laptop.
- **Durable queue**: Events (cron ticks, webhooks, human commands) flow into a queue (Celery + Redis, Kafka, or SurrealDB's built-in messaging).
- **Session spawning/resumption**: Each event spawns or resumes an agent session with the event payload as context.
- **Crash resilience**: Failed events can be retried with backoff; checkpointed sessions resume from the last good state.
- **Concurrency limits**: A burst of events should not spawn unbounded concurrent research sessions. Implement queuing and max-concurrency controls.

### 8.3 Event-Driven Research Continuation

The unified trigger system proposed by Hermes Agent/Gobii is instructive:[^14]

```
UNIFIED TRIGGER SYSTEM
======================

Cron tick    → scheduler.tick()     → spawn/resume research session
Webhook POST → /hooks/inbound/<id>  → spawn/resume research session
Email arrival → IMAP idle listener   → spawn/resume research session
Human command → CLI / UI action      → spawn/resume research session
Agent-to-agent → peer message        → spawn/resume research session

All paths converge on the same session-spawning machinery.
```

For Prometheus, this means the deep-research skill should expose:
- A `schedule_research` tool that creates cron jobs.
- A `webhook_trigger` endpoint that receives external events.
- A `continue_research` command that resumes from the last checkpoint.

### 8.4 Resource Management for Background Research

Background research must be budget-conscious:
- **Token budgets**: Cap total tokens per research run; track consumption in the waypoint.
- **Wall-clock budgets**: Maximum duration; if exceeded, checkpoint and schedule continuation.
- **Source freshness budgets**: Maximum age of sources before re-retrieval is required.
- **Model tier routing**: Use cheaper models for search planning and evidence retrieval; reserve expensive models for synthesis and verification.

[^14]: https://github.com/NousResearch/hermes-agent/issues/491 (Hermes Agent, "Webhook-Triggered Agent Sessions", 2026)

---

## 9. Maintaining Research Quality Over Long Durations

### 9.1 Periodic Re-Verification

Long-running research risks **factual drift**: sources change, new evidence emerges, and earlier conclusions may no longer hold. Best practices include:[^13]

- **Stewardship runs**: Periodic maintenance cycles that scan for duplicate candidates, detect conflicting entries, flag stale entries, and suggest canonical promotion.
- **Drift scans**: Compare memory entries against live source files to find stale, missing, or changed references.
- **Verification candidates ranking**: Prioritize never-verified canonical entries, entries with failed verification, and entries exceeding the staleness threshold.
- **Age-aware recall**: Apply configurable exponential age decay so stale, non-evergreen memories sink in retrieval results while canonical knowledge stays prominent.

### 9.2 Stale Source Flagging

Sources retrieved early in a long research process should be explicitly flagged for re-verification:[^13]

```
SOURCE LIFECYCLE
================

retrieved → active → verified → canonical
   ↓           ↓          ↓
stale    → outdated → superseded → archived

Transitions triggered by:
- Age threshold exceeded
- Source content changed (SHA-256 mismatch)
- Source no longer reachable (404, domain expired)
- Newer, contradictory source found
- Human reviewer flag
```

### 9.3 Knowledge Graph Health Monitoring

A periodic **lint** pass (as prescribed by the Karpathy LLM Wiki pattern) should health-check the research knowledge base:[^15]

- Contradictions between claims
- Stale claims superseded by newer sources
- Orphan claims with no inbound citations
- Important concepts mentioned but lacking their own entity page
- Missing cross-references between related topics
- Data gaps that could be filled with additional web search

### 9.4 Quality Metrics Dashboard

For long-running research, track these metrics continuously:

| Metric | Definition | Target |
|---|---|---|
| **Verification Coverage** | % of claims with at least one verification | >90% |
| **Source Freshness** | % of sources retrieved within last N hours | >80% for fast-moving topics |
| **Contradiction Resolution Rate** | % of flagged contradictions with resolved status | >75% |
| **Confidence Decay** | Average confidence score of claims over time | Stable or increasing |
| **Citation Traceability** | % of claims traceable to a specific source excerpt | >95% |
| **Checkpoint Recovery Success** | % of interrupted sessions successfully resumed | >99% |

[^13]: https://github.com/ipiton/agent-memory-mcp (Agent Memory MCP: drift detection, stewardship, verification, 2026)
[^15]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f (Karpathy, "LLM Wiki", 2026)

---

## 10. The Karpathy Loop Applied to Long-Running Research

### 10.1 The Karpathy LLM Wiki Pattern

Andrej Karpathy's LLM Wiki pattern consists of three layers and five operations:[^15] [^16]

**Three Layers:**
- **Raw Sources** (immutable): Curated collection of source documents. The LLM reads but never modifies.
- **The Wiki** (LLM-generated): Directory of markdown files—summaries, entity pages, concept pages. The LLM owns this layer entirely.
- **The Schema** (conventions): Configuration document telling the LLM how the wiki is structured and what workflows to run.

**Five Operations:**
- **Ingest**: Process a new source, update multiple wiki pages, append to log.
- **Query**: Answer questions by synthesizing existing pages; good answers are filed back into the wiki.
- **Lint**: Periodic health checks for contradictions, gaps, stale claims, orphan pages.
- **Compile**: Transform raw sources into structured wiki pages.
- **Audit**: Review and validate the quality and consistency of the wiki.

### 10.2 Mapping the Karpathy Loop to Deep Research

The Karpathy Loop's three core hooks map directly to the deep-research pipeline stages:

| Karpathy Hook | Deep Research Stage | Action |
|---|---|---|
| **`focus` (UserPromptSubmit)** | **Planner + Search** | Inject compiled knowledge graph context into the research prompt; orient the agent with `index.md` and relevant wiki pages |
| **`reflect` (Stop)** | **Verify + Resolve + Cite** | Review what was discovered; identify gaps; flag contradictions; note what needs deeper investigation |
| **`ingest` (Post-Reflect)** | **Collect + Graph** | Write session findings back to the wiki: update entity pages, add new evidence, cross-reference sources, append to `log.md` |

For long-running research, this loop operates at two frequencies:
- **Micro-loop**: Every search-retrieve-extract iteration updates the evidence bank and knowledge graph.
- **Macro-loop**: Every session boundary (or every N hours) runs a full lint/audit pass, resolves contradictions, and generates the next session's prioritized sub-questions.

### 10.3 The Wiki as a Compounding Research Artifact

Unlike traditional RAG (which rediscovers knowledge from scratch per query), the wiki **compiles knowledge once and keeps it current**. Cross-references already exist. Contradictions have already been flagged. Synthesis reflects everything ingested.[^16]

For long-running research, this is transformative: the research agent does not start each session with a blank slate. It starts with a structured, cross-referenced, contradiction-audited knowledge base that grows richer with every iteration. The `.research` package output should be structured as a **wiki-like knowledge object**—not just a flat report.

### 10.4 Integration with Prometheus Ecosystem

The Prometheus ecosystem already has the Karpathy Loop integrated via Claude Code hooks. For long-running research, this integration should be extended:

- **`focus` injects not just general knowledge but the current research state**: progress, open questions, verified claims, and flagged contradictions.
- **`reflect` captures not just the session's lessons but the research-specific verification results**: which claims were confirmed, which were challenged, which sources proved unreliable.
- **`ingest` writes not just to the general KB but to the research-specific evidence bank and knowledge graph**.

[^15]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f (Karpathy, "LLM Wiki", 2026)
[^16]: https://www.aibuilderclub.com/blog/karpathy-llm-wiki ("Karpathy's LLM Wiki: A Knowledge Base That Compounds", 2026)

---

## 11. Handling Partial Failures in Long Research Pipelines

### 11.1 The Graceful Degradation Ladder

A production research pipeline must handle failures at every layer. The trusted degradation ladder is:[^17]

1. **Retry briefly** if the failure is genuinely transient (network timeout, 503).
2. **Switch to a compatible fallback** if the contract can stay the same (e.g., switch search API from Tavily to Perplexity).
3. **Reduce capability on purpose** if the backup is weaker (e.g., a research agent can return gathered evidence and queue synthesis if the synthesis model is unavailable).
4. **Serve cached or queued work** if real-time AI is unavailable (return previously retrieved evidence with a staleness warning).
5. **Hand off cleanly** when correctness matters more than continuity (escalate to human researcher with a structured partial-results package).

### 11.2 Circuit Breaker Pattern for Research APIs

Circuit breakers prevent retry storms that amplify failures:[^18]

```
CLOSED → (failures exceed threshold) → OPEN → (cooldown expires) → HALF-OPEN
  ↑                                    ↓
  ←←←←←← (probe succeeds) ←←←←←←←←←←←←
  ←←←←←← (probe fails) → OPEN_EXTENDED (longer cooldown)
```

| Parameter | Typical Value | Purpose |
|---|---|---|
| Failure threshold | 3–5 failures | Trips the breaker |
| Detection window | 5 minutes | Time window for counting failures |
| Initial backoff | 5 minutes | Cooldown before first probe |
| Extended backoff | 15 minutes | Cooldown after repeated failures |

**Critical**: Only infrastructure failures (timeouts, 502/503/504) should trip the breaker. Business logic errors (400, 401, validation failures) indicate request problems, not service problems.[^18]

### 11.3 Fallback Chains for Model and Tool Failures

Production research systems implement multi-tier fallback chains:[^19]

```yaml
fallbackChains:
  search:
    - tavily      # Primary
    - perplexity  # Secondary
    - serper      # Tertiary
    - bing        # Last resort
  
  synthesis_model:
    - claude-sonnet-4   # Primary
    - gpt-4o            # Secondary
    - gemini-1.5-pro    # Tertiary
  
  verification_model:
    - gpt-4o-mini       # Primary (cheap, good at judging)
    - claude-haiku      # Secondary
```

### 11.4 Structured Error Recovery

Every failure in the research pipeline should produce a structured error record that enables downstream recovery:[^20]

```json
{
  "code": "SOURCE_RETRIEVAL_FAILED",
  "message": "Primary source returned 404; fallback source retrieved successfully",
  "context": {
    "task": "verify_claim_X",
    "source_url": "https://example.com/paper",
    "fallback_url": "https://archive.org/...",
    "claim_id": "claim-123"
  },
  "suggestions": [
    "Use archive.org fallback for dead academic links",
    "Flag original source as potentially unstable",
    "Consider re-retrieving from publisher's DOI"
  ],
  "recoverable": true,
  "degradation_level": 1
}
```

### 11.5 Partial Results Are Valuable

The most important principle for failure handling in research: **partial results are still results**. If a research pipeline processing 200 documents hits an error on document 201, the first 200 documents' worth of evidence, claims, and synthesis should be preserved and delivered—with clear metadata about what was not completed.[^1]

For Prometheus, this means the `.research` package should be **valid and useful even when incomplete**. A `completion_status` field can indicate `complete`, `partial`, or `failed`, and consumers should handle partial packages gracefully.

[^17]: https://www.buildmvpfast.com/blog/graceful-degradation-ai-agents-fallback-model-unavailable-2026 ("Graceful Degradation for AI Agents", 2026)
[^18]: https://zylos.ai/research/2026-02-20-graceful-degradation-ai-agent-systems/ (Zylos, "Graceful Degradation Patterns in AI Agent Systems", 2026)
[^19]: https://github.com/williamzujkowski/nexus-agents/blob/main/docs/architecture/CONTEXT_LOAD_BALANCING.md (Nexus Agents, "Context Load Balancing", 2026)
[^20]: https://github.com/ray-manaloto/ai-agent-study-guide/blob/main/docs/tools/BEST-PRACTICES.md (AI Agent Study Guide, "Error Recovery Patterns", 2025)

---

## Cross-Cutting Insights & Strategic Recommendations

### For the Prometheus Deep-Research Skill Specification

1. **Adopt the "workspace is the memory bus" philosophy**: The `.research` package (directory of JSON, markdown, and embeddings) is the continuity layer—not the model's context window. Every session starts fresh, rehydrates from disk, and leaves the workspace cleaner than it found it.

2. **Use LangGraph checkpointing as the primary durability mechanism**: For the 10-stage deep-research pipeline (Planner→Search→Retrieve→Collect→Verify→Resolve→Graph→Cite→Report→Export), compile each stage as a LangGraph node with `PostgresSaver` checkpointing. This provides time-travel debugging, human-in-the-loop gates, and crash recovery out of the box.

3. **Extend the KBD `current-waypoint.json` pattern**: Each research run gets its own waypoint file tracking phase, stage, sub-question progress, evidence stats, and quality metrics. This is the human-readable and machine-readable progress tracker.

4. **Implement the Karpathy Loop at two frequencies**: Micro-loop (every iteration updates the evidence bank and knowledge graph) and macro-loop (every session boundary runs lint/audit, resolves contradictions, and generates the next session's priorities).

5. **Design for partial completion**: The `.research` package must be valid and useful when incomplete. Include `completion_status`, `progress_percentage`, and `next_steps` fields so consumers can act on partial research.

6. **Invest in verification infrastructure**: Multi-agent verification (LLM-as-Judge, process verification, adversarial debate) is not optional for production research. Error accumulation is silent and structural. Budget 20–30% of token spend on verification.

7. **Use SurrealDB for the persistent knowledge layer**: SurrealDB's multi-model capabilities (graph + vector + document + relational in one ACID transaction) make it ideal for storing the knowledge graph, evidence embeddings, and checkpoint state without cross-store consistency problems.[^21]

8. **Implement cron + webhook triggers**: Research should not only be interactive. Support scheduled research (daily intelligence briefs), event-driven research (triggered by news or data changes), and human-initiated deep dives.

9. **Track confidence decay and source freshness**: Every claim in the knowledge graph should have a `confidence` score, `verified_at` timestamp, and `source_hash`. Background stewardship runs should periodically downgrade unverified claims and flag stale sources.

10. **Fresh sessions > giant sessions**: The Ralph Loop principle applies directly. A research agent that runs for 6 hours in one conversation will degrade. A research agent that restarts 12 times with 30-minute sessions, each rehydrating from a rich workspace, will produce higher-quality output.

### Architecture Sketch for the Enhanced Deep-Research Skill

```
PROMETHEUS LONG-RUNNING DEEP RESEARCH
======================================

Trigger Layer:
  - Cron scheduler (periodic research)
  - Webhook endpoint (event-driven research)
  - Human command (ad-hoc research)
  - All converge on: spawn/resume research session

Orchestration Layer (LangGraph + KBD Waypoints):
  - Phase: planning → evidence_collection → synthesis → verification → export
  - Stage: within each phase, granular steps tracked in current-waypoint.json
  - Checkpoint: after every node, persisted to Postgres/SurrealDB

Agent Layer (Fresh Sessions per Unit of Work):
  - Planner Agent: decomposes topic into sub-questions
  - Search Agent: retrieves sources for one sub-question
  - Extractor Agent: pulls claims and evidence
  - Verifier Agent: checks claims against sources
  - Synthesizer Agent: merges findings into knowledge graph
  - Each agent: fresh context, reads state, does bounded work, writes state, exits

Knowledge Layer (Karpathy Wiki + SurrealDB):
  - evidence_bank/: raw sources with metadata
  - wiki/: compiled pages, entity pages, concept pages
  - knowledge_graph.json: entities, relationships, claims, confidence
  - index.md: catalog of all pages
  - log.md: append-only chronological record

Quality Layer:
  - Multi-agent verification at each stage
  - Confidence decay for unverified claims
  - Drift detection for stale sources
  - Periodic lint/audit passes
  - Human review gates at phase boundaries

Export Layer:
  - .research package: structured knowledge object (not just a report)
  - Includes: report.md, knowledge_graph.json, citations.json, evidence_bank/, verification_log.json, reasoning_traces/
```

[^21]: https://surrealdb.com/platform/spectron (SurrealDB Spectron: Agent memory platform, 2026)

---

## References

[^1]: Osmani, A. (2026). *Long-running Agents*. https://addyosmani.com/blog/long-running-agents/

[^2]: Bustamante, N. (2026). *Long Running Agent Engineering*. https://nicolasbustamante.com/blog/long-running-agent-engineering

[^3]: Google Developers Blog. (2026). *Build Long-running AI agents that pause, resume, and never lose context with ADK*. https://developers.googleblog.com/build-long-running-ai-agents-that-pause-resume-and-never-lose-context-with-adk/

[^4]: Indium. (2026). *7 State Persistence Strategies for Long-Running AI Agents in 2026*. https://www.indium.tech/blog/7-state-persistence-strategies-ai-agents-2026/

[^5]: AutoLearningAgents. (2026). *LangGraph Checkpointing and Time-Travel Debugging*. https://www.autolearningagents.com/langgraph/checkpointing.php

[^6]: Sheik Nomaan, Ph.D. (2026). *Deepagents on LangGraph: Debugging Long-Running AI Agents with Time-Travel*. https://pub.towardsai.net/deepagents-on-langgraph-debugging-long-running-ai-agents-with-time-travel-ff897ef50b73

[^7]: Easton Dev. (2026). *LangGraph State Management: Checkpoints, Thread State, and Failure Recovery*. https://eastondev.com/blog/en/posts/ai/20260424-langgraph-agent-architecture/

[^8]: AI System Design Guide. (2025). *Durable Execution*. https://github.com/ombharatiya/ai-system-design-guide/blob/main/07-agentic-systems/11-durable-execution.md

[^9]: Saxena et al. (2025). *NexusSum: Hierarchical LLM Agents for Long-Form Narrative Summarization*. arXiv:2505.24575. https://arxiv.org/html/2505.24575v1

[^10]: Anonymous. (2025). *Scaling LLM Multi-Turn RL with End-to-End Summarization-Based Context Management*. https://openreview.net/attachment?id=azy6Tjy3QH&name=pdf

[^11]: *How Multi-Agent Self-Verification Actually Works*. (2026). https://pub.towardsai.net/how-multi-agent-self-verification-actually-works-and-why-it-changes-everything-for-production-ai-71923df63d01

[^12]: MindStudio. (2026). *What Is Chain-of-Thought Faithfulness? Why AI Reasoning Traces Are Unreliable*. https://www.mindstudio.ai/blog/what-is-chain-of-thought-faithfulness-ai-reasoning

[^13]: Haustorium12. (2026). *Memory-v3: Brain-inspired persistent memory for AI coding assistants*. https://github.com/Haustorium12/memory-v3

[^14]: NousResearch / Hermes Agent. (2026). *Feature: Webhook-Triggered Agent Sessions*. https://github.com/NousResearch/hermes-agent/issues/491

[^15]: Karpathy, A. (2026). *LLM Wiki*. GitHub Gist. https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f

[^16]: AI Builder Club. (2026). *Karpathy's LLM Wiki: A Knowledge Base That Compounds*. https://www.aibuilderclub.com/blog/karpathy-llm-wiki

[^17]: Build MVP Fast. (2026). *Graceful Degradation for AI Agents*. https://www.buildmvpfast.com/blog/graceful-degradation-ai-agents-fallback-model-unavailable-2026

[^18]: Zylos. (2026). *Graceful Degradation Patterns in AI Agent Systems*. https://zylos.ai/research/2026-02-20-graceful-degradation-ai-agent-systems/

[^19]: Nexus Agents. (2026). *Context Load Balancing*. https://github.com/williamzujkowski/nexus-agents/blob/main/docs/architecture/CONTEXT_LOAD_BALANCING.md

[^20]: AI Agent Study Guide. (2025). *Error Recovery Patterns*. https://github.com/ray-manaloto/ai-agent-study-guide/blob/main/docs/tools/BEST-PRACTICES.md

[^21]: SurrealDB. (2026). *Spectron: Agent memory you can trust*. https://surrealdb.com/platform/spectron

[^22]: Let's Build Solutions. (2026). *LangGraph in Production: Stateful Agent Graphs, Conditional Edges, and Checkpointing*. https://letsbuildsolutions.com/blog/ai-ml/langgraph-in-production-stateful-agent-graphs-conditional-edges-and-checkpointing-for-reliable-multi-step-ai-workflows/

[^23]: Rajat Pandit. (2026). *State Management in LangGraph: Checkpointing and Time Travel*. https://rajatpandit.com/agentic-ai/langgraph-state-management-checkpoints

[^24]: C-Sharp Corner. (2026). *Handle Agent Memory, Checkpointing, and Recovery in Multi-Agent Workflows Using LangGraph*. https://www.c-sharpcorner.com/article/handle-agent-memory-checkpointing-and-recovery-in-multi-agent-workflows-using/

[^25]: DuraLang. (2026). *Make stochastic AI systems durable with one decorator*. https://github.com/deepansh-saxena/DuraLang

[^26]: Google Cloud Events. (2026). *Long-Running Agents: The AI that never sleeps*. https://www.googlecloudevents.com/next-vegas/session/3931993/session-library

[^27]: OpenReview. (2025). *Context-Aware Hierarchical Merging for Long Document Summarization*. https://arxiv.org/html/2502.00977v1

[^28]: Forte, T. (2018). *Second Brain Case Study: Progressive Summarization in the Intelligence Community*. https://fortelabs.com/blog/second-brain-case-study-progressive-summarization-in-the-intelligence-community/

[^29]: FlowSearch. (2025). *Advancing deep research with dynamic structured knowledge flow*. arXiv:2510.08521. https://arxiv.org/html/2510.08521v1

[^30]: Intelligent Living. (2026). *Karpathy's LLM Wiki: The Markdown Knowledge Base Pattern*. https://www.intelligentliving.co/karpathy-llm-wiki-markdown-knowledge-base/

[^31]: EchoFold AI. (2026). *Karpathy's LLM Wiki: How We Built a 2,188-Document Personal Knowledge Base*. https://echofold.ai/news/karpathy-llm-wiki-how-to-build-personal-knowledge-base

[^32]: Harness Engineering Academy. (2026). *Building Resilient AI Agents: Implementing Retry Logic, Fallback Patterns, and Graceful Degradation*. https://harnessengineering.academy/blog/building-resilient-ai-agents-implementing-retry-logic-fallback-patterns-and-graceful-degradation-for-unreliable-tools/

[^33]: Machine Learning Plus. (2026). *LangGraph Error Handling: Retries & Fallback Strategies*. https://machinelearningplus.com/gen-ai/langgraph-error-handling-retries-fallback-strategies/

[^34]: IsoFinancial MCP. (2025). *Reliability Infrastructure: Multi-Source Fallback and Graceful Degradation*. https://github.com/Niels-8/isofinancial-mcp/blob/main/docs/RELIABILITY.md

[^35]: Agent Memory MCP. (2026). *MCP server that gives AI agents persistent memory with semantic search*. https://github.com/ipiton/agent-memory-mcp

[^36]: Wiki Forge. (2026). *Second-brain knowledge repository for humans and LLMs*. https://github.com/FasalZein/wiki-forge

[^37]: Orogat, A. (2026). *Is Agent Memory a Database? Rethinking Data Foundations for Long-Term AI Agent Memory*. arXiv:2605.26252. https://arxiv.org/abs/2605.26252

[^38]: Ert Yurk. (2026). *SurrealDB for AI Agents: Relational, Graph, and Vector in One Database*. https://ertyurk.com/posts/surrealdb-for-ai-agents/

[^39]: Mastra Agent Surreal Starter. (2025). *Mastra Agent starter using SurrealDB*. https://github.com/jonathanprozzi/mastra-agent-surreal-starter

[^40]: Autodidact Skill. (2026). *An autonomous research agent built on Karpathy's LLM Wiki*. https://github.com/DamonChen-anan/autodidact-skill
