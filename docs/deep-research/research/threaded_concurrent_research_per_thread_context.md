# Threaded/Concurrent Research with Per-Thread Context

## Research Report for the Prometheus Skill Pack: Deep-Research Enhancement

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Specialist (Sub-agent)  
**Topic:** Threaded/Concurrent Research with Per-Thread Context  
**Status:** Evidence-based, structured findings with citations

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Multi-Threaded Research Agents: State of the Art](#2-multi-threaded-research-agents-state-of-the-art)
3. [Per-Thread Context Management](#3-per-thread-context-management)
4. [Merging Results from Concurrent Research Threads](#4-merging-results-from-concurrent-research-threads)
5. [Avoiding Context Loss in Long-Running Threads](#5-avoiding-context-loss-in-long-running-threads)
6. [Parallel Research Decomposition Patterns](#6-parallel-research-decomposition-patterns)
7. [Feynman Loop Recursion Extended to Parallel Research](#7-feynman-loop-recursion-extended-to-parallel-research)
8. [KBD Orchestrator L1/L2/L3 Managing Parallel Research Streams](#8-kbd-orchestrator-l1l2l3-managing-parallel-research-streams)
9. [Concurrency Limits and Resource Management](#9-concurrency-limits-and-resource-management)
10. [LangGraph Parallel Agent Execution for Research](#10-langgraph-parallel-agent-execution-for-research)
11. [Thread Synchronization and Cross-Thread Citation Discovery](#11-thread-synchronization-and-cross-thread-citation-discovery)
12. [Recommendations for Prometheus Deep-Research Integration](#12-recommendations-for-prometheus-deep-research-integration)
13. [Sources & Citations](#13-sources--citations)

---

## 1. Executive Summary

Concurrent, multi-threaded research is emerging as the dominant paradigm for deep, evidence-based investigation. Leading systems—**Kimi K2.6** (300 sub-agents), **MiniMax Agent Teams**, **Claude Code subagents**, and **LangGraph**—demonstrate that parallel research threads dramatically improve coverage, speed, and factual accuracy when properly orchestrated. The key challenges are not spawning threads, but **managing per-thread context**, **merging partial knowledge graphs**, **deduplicating citations**, **resolving contradictions across threads**, and **maintaining state across long-running sessions**.

For Prometheus, the integration pattern is clear: the existing **Feynman Learning Loop** (PMPO-based: explain → grade → gap → recurse) and the **KBD orchestrator's four nested loop layers** (L0–L3) already provide the conceptual scaffolding for parallel research. The deep-research pipeline's 10 stages can be decomposed into parallel sub-pipelines, each with its own context, evidence set, and partial knowledge graph, then merged via deterministic aggregation.

---

## 2. Multi-Threaded Research Agents: State of the Art

### 2.1 Kimi K2.6 Agent Swarm (Moonshot AI)

Kimi K2.6 represents the most aggressive horizontal scaling of multi-agent research to date. Released April 2026, it is a 1-trillion-parameter MoE model with a **"horizontal scaling" architecture** that coordinates up to **300 sub-agents executing across 4,000 coordinated steps simultaneously**—a 3× expansion from K2.5's 100 sub-agents / 1,500 steps. [Kimi K2.6 Tech Blog](https://www.kimi.com/blog/kimi-k2-6)

Key characteristics for research:
- **Dynamic task decomposition**: The orchestrator decomposes complex prompts into heterogeneous subtasks, assigns domain-specialized agents, and synthesizes outputs into unified deliverables (documents, websites, slides, spreadsheets). [Kimi K2.6 Complete Guide](https://www.aimadetools.com/blog/kimi-k2-6-complete-guide/)
- **Shared context**: Sub-agents communicate through a shared context, building on each other's work without duplicating effort. [Kimi K2.6 Explained](https://miraflow.ai/blog/kimi-k2-6-explained-moonshot-ai-open-source-model-ties-gpt-5-5-coding)
- **Benchmark impact**: On BrowseComp Swarm (a multi-source research benchmark), K2.6 scores **86.3%** vs GPT-5.4's 78.4%, demonstrating that parallel research distribution significantly improves factual recall. [Kimi K2.6 Complete Guide](https://www.aimadetools.com/blog/kimi-k2-6-complete-guide/)
- **Claw Groups (research preview)**: Allows humans and agents from heterogeneous sources (any model, any device) to collaborate in the same swarm, with K2.6 as an adaptive coordinator. [MarkTechPost](https://www.marktechpost.com/2026/04/20/moonshot-ai-releases-kimi-k2-6-with-long-horizon-coding-agent-swarm-scaling-to-300-sub-agents-and-4000-coordinated-steps/)

### 2.2 MiniMax Agent Teams (Mavis)

MiniMax's Agent Team system, introduced May 2026, is explicitly designed for **parallel information retrieval and research**. It uses a three-role collaboration flow: **Leader** (translates user goal to task structure), **Worker** (executes sub-tasks with specific tools and specializations), and **Verifier** (ensures deliverables meet quality, checks sources and risks). [MiniMax Agent Team Blog](https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953)

For research specifically, MiniMax identifies four core scenarios:
1. **Parallel information retrieval and research**: Single agents suffer from slow research, polluted context, and biased thinking. Agent Teams split research into parallel channels and merge findings via verifier and synthesizer. [MiniMax Agent Team Blog](https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953)
2. **Independent verifier reduces citation errors**: Verifier checks source verifiability (stable URLs preferred), staleness, and counter-evidence. [MiniMax Agent Team Blog](https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953)
3. **Adversarial quality gates**: Worker and Verifier are in an adversarial relationship—one finishing triggers the other to start, producing high-quality results through multi-round iteration without human micro-management. [MiniMax Release Notes](https://releasebot.io/updates/minimax)
4. **Costs recognized**: MiniMax explicitly documents three costs introduced by multi-agent research: **handoff cost** (reorganizing info between agents), **sharing cost** (token usage for shared context), and **aggregation cost** (merging outputs consistently). [MiniMax Release Notes](https://releasebot.io/updates/minimax)

### 2.3 Claude Code Subagents & Agent Teams (Anthropic)

Claude Code provides **subagents** (specialized workers with isolated context windows, custom prompts, and restricted tool permissions) and **Agent Teams** (coordinated parallel workers). [Builder.io Blog](https://www.builder.io/blog/claude-code-subagents)

For parallel research, key patterns include:
- **Independent reasoning threads**: Each subagent runs in its own context window, allowing parallel research on multiple aspects of a problem simultaneously. [Claude Code Parallel Subagents Best Practices](https://claudecodeguides.com/parallel-subagents-claude-code-best-practices-2026/)
- **Context isolation**: Subagents operate independently without contaminating the primary conversation context, preventing cross-task confusion. [Rivista AI Paper](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)
- **Three execution modes**: Parallel dispatch (independent domains, no shared state), Sequential dispatch (dependencies exist), and Background dispatch (research while user continues working). [Claude Fast Blog](https://claudefa.st/blog/guide/agents/sub-agent-best-practices)
- **Practical example for research**: "Research these 5 companies in parallel using separate sub-agents... Each sub-agent should analyze market position, recent news, and competitive advantages." [Tim Dietrich Blog](https://timdietrich.me/blog/claude-code-parallel-subagents/)
- **Limitations**: Sub-agents cannot spawn other sub-agents (no nesting); parallel writes can cause conflicts; many simultaneous agents may hit API rate limits. [Tim Dietrich Blog](https://timdietrich.me/blog/claude-code-parallel-subagents/)

### 2.4 OpenAI Swarm / Codex

OpenAI's Codex app is described as a "command center managing parallel software-engineering agents." The OpenAI Agents SDK (April 2026 update) includes externalized agent state, snapshotting, and rehydration into fresh containers. [Reinforcement Learning for LLM-based Multi-Agent Systems](https://arxiv.org/html/2605.02801v1)

The OpenAI Agents SDK session system persists conversation items across turns using backends (SQLite, Redis, SQLAlchemy, Dapr, OpenAI Conversations), but session memory alone is not full durable execution. The SDK's human-in-the-loop support serializes interrupted run state (approvals, usage, tool input, nested resumptions) for later resumption. [Zylos Research](https://zylos.ai/research/2026-04-24-durable-execution-agent-runtimes/)

### 2.5 LangGraph Deep Agents (LangChain)

LangChain's **Deep Agents** (built on LangGraph) is a "batteries-included agent harness" explicitly designed for long-horizon, multi-step work with:
- Sub-agents with isolated context windows
- Context management (summarize long threads, offload tool outputs to disk)
- Persistent memory (pluggable state and store backends for cross-session recall)
- Built on LangGraph's streaming, persistence, and checkpointing primitives [LangChain DeepAgents GitHub](https://github.com/langchain-ai/deepagents)

---

## 3. Per-Thread Context Management

### 3.1 The Core Problem

Each concurrent research thread must maintain its own **sub-question**, **evidence set**, **partial knowledge graph**, and **working memory** without contaminating other threads. The main orchestrator's context must remain clean—carrying only high-level directives, not the raw noise of every search result. [Zylos Research](https://zylos.ai/research/2026-03-31-context-window-management-session-lifecycle-long-running-agents/)

### 3.2 Functional Context Isolation

The Claude Code subagent system illustrates the principle: each subagent is a specialized AI assistant with its own isolated context window, custom system prompt, and restricted tool permissions. When a task matches a subagent's expertise, the main system delegates it; the subagent operates independently without contaminating the primary context. [Rivista AI Paper](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)

This principle can be applied along:
- **Functional dimensions**: analysis, execution, validation each get isolated units
- **Hierarchical layers**: planning, implementation, review each get isolated units
- **Minimum permissions**: each unit receives only the permissions required for its specific responsibilities, improving reliability and interpretability [Rivista AI Paper](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)

### 3.3 Lightweight References vs. Full Context

Context isolation often relies on storing large information externally and exposing only lightweight references in the model's window. In the **sandbox approach** (HuggingFace's CodeAgent), bulky output is stored in a separate sandbox and retrieved only when needed. The model interacts with concise references; the sandbox holds full data and provides it on demand. [Rivista AI Paper](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)

Similarly, **schema-based state objects** keep heavy elements (files, logs) in external storage and surface only selected fields. Both approaches reduce token overhead while retaining access to complete context when required. [Rivista AI Paper](https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf)

### 3.4 Per-Thread State Schema (LangGraph Pattern)

In LangGraph, each thread's graph state is a `TypedDict` schema. Every field is explicit, serializable, and validated. The rule of thumb: **if it's in state, it's in Postgres** (or the chosen persistence backend). For research threads, this means:
- `messages`: conversation history (with reducers that append)
- `evidence`: list of collected sources (bounded by policy)
- `sub_question`: the specific research query for this thread
- `partial_graph`: serialized partial knowledge graph
- `status`: running / awaiting_tool / done / error
- `tokens_used` / `tokens_budget`: budget enforcement [RapidClaw Blog](https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026)

### 3.5 Best Practices for Per-Thread Context

| Practice | Rationale | Source |
|----------|-----------|--------|
| Define clear sub-question per thread | Prevents scope drift and overlap | [Claude Fast](https://claudefa.st/blog/guide/agents/sub-agent-best-practices) |
| Store bulky artifacts externally (S3/filesystem) | Keep state schema tiny; 10MB payload × 50 steps × 1000 runs = 500GB checkpoint traffic | [RapidClaw Blog](https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026) |
| Summarize verbose intermediate outputs before handoff | Prevents token cost explosion from shared message history | [Abstract Algorithms](https://abstractalgorithms.dev/langgraph-multi-agent-supervisor-pattern) |
| Use custom system prompts per thread | Specialization improves output quality and reusability | [Builder.io Blog](https://www.builder.io/blog/claude-code-subagents) |
| Restrict tool permissions per thread | Security, precision, and reduced context pollution | [Builder.io Blog](https://www.builder.io/blog/claude-code-subagents) |

---

## 4. Merging Results from Concurrent Research Threads

### 4.1 The Aggregation Challenge

Aggregation turns local child outputs into a global result. This is where deterministic code—not models—should do the heavy lifting: parsing, deduplication, counting, merging, ranking, and provenance preservation. [Aman's AI Journal](https://aman.ai/primers/ai/recursive-transformers/)

The important split:
- **Models do semantic work**: classify, extract, compare, explain, synthesize
- **Code does deterministic work**: count, sort, validate, deduplicate, merge, format

### 4.2 Knowledge Graph Merging

Knowledge graph merging involves:
1. **Schema alignment**: Mapping schemas/ontologies so similar concepts are represented consistently (e.g., "CustomerID" vs "ClientID")
2. **Entity resolution**: Identifying and merging duplicate entities across graphs (e.g., "John Doe" vs "J. Doe")
3. **Relationship integration**: Harmonizing relationships to avoid duplication or inconsistency
4. **Conflict resolution**: Resolving contradictory information using predefined rules or external validation
5. **Data enrichment**: Combining complementary information from different graphs [Meegle](https://www.meegle.com/en_us/topics/knowledge-graphs/knowledge-graph-merging)

### 4.3 Citation Deduplication

From the recursive language model perspective, deduplication should remove:
- Repeated spans (same text cited multiple times)
- Repeated rows (same data point from different sources)
- Repeated files (same PDF discovered by different threads)
- Repeated claims (same factual assertion with different wording) [Aman's AI Journal](https://aman.ai/primers/ai/recursive-transformers/)

Best practice: assign a **canonical source ID** at ingestion. When two threads discover the same URL or DOI, merge under the canonical ID and preserve both thread provenance paths.

### 4.4 Contradiction Resolution

Most competing memory systems store facts as stated and retrieve by similarity. When a new fact contradicts an old one, both are stored. Retrieval surfaces whichever is more similar to the query—which may be the stale version. [Exabase](https://exabase.io/blog/what-is-memory-drift-in-ai-agents)

A robust architecture **actively resolves contradictions** rather than letting old and new memories coexist silently:
1. Identify the relationship between the two statements
2. Determine which is more recent (temporal reasoning)
3. Update the memory graph accordingly
4. Weight the stale version correctly so retrieval surfaces what is true now [Exabase](https://exabase.io/blog/what-is-memory-drift-in-ai-agents)

For research threads, this means a **verification layer** after aggregation that checks:
- **Support**: Every final claim is backed by selected evidence
- **Contradiction**: No selected evidence contradicts the answer
- **Completeness**: The answer covers all parts of the user query
- **Minimality**: Evidence spans are concise and directly relevant [Aman's AI Journal](https://aman.ai/primers/ai/recursive-transformers/)

### 4.5 Graph Matching-Based Fusion

A graph matching-based algorithm for knowledge fusion achieves:
- **Node matching**: Identifies identical or similar nodes by calculating attribute similarity
- **Edge matching**: Matches relationships between nodes based on node matching
- **Conflict resolution**: Handles conflicting information using rules or statistical methods
- **Parallel computing**: Accelerates fusion of large-scale knowledge graphs [Research Square](https://www.researchsquare.com/article/rs-4641408/v1.pdf?c=1721742069000)

Experimental results show average node matching accuracy improvements of over 10% compared to traditional fusion methods, with significant edge matching accuracy and overall fusion quality gains. [Research Square](https://www.researchsquare.com/article/rs-4641408/v1.pdf?c=1721742069000)

### 4.6 Merging Pipeline for Prometheus

Proposed merge stages for `.research` package output:

```
Thread Results → Parse → Deduplicate → Entity Resolve → 
Conflict Detect → Rank Evidence → Synthesize → Verify → 
Update Master Graph → Write .research Package
```

---

## 5. Avoiding Context Loss in Long-Running Threads

### 5.1 The Context Window Problem

AI agents with continuous operation face a fundamental limitation: the context window is working memory. When it fills, something gets dropped. Early in operation, agents try to keep everything—full file contents, complete command outputs, entire web pages. The result is predictable: halfway through a task, the thread is lost. [Dev.to Bobrenze](https://dev.to/bobrenze/ai-agent-context-window-management-how-i-handle-tasks-that-take-longer-than-my-memory-4b47)

### 5.2 Explicit Checkpointing

Every multi-step task gets a state file. For research threads, this means writing intermediate findings to disk: `explored_sources.md`, `hypothesis.md`, `next_steps.md`. If context compacts, the thread re-reads these files. The filesystem is the L2 cache. [Dev.to Bobrenze](https://dev.to/bobrenze/ai-agent-context-window-management-how-i-handle-tasks-that-take-longer-than-my-memory-4b47)

For Prometheus, this maps directly to the `current-waypoint.json` + `progress.json` pattern already present in the KBD orchestrator.

### 5.3 LangGraph Checkpointing: Time-Travel and Fault Tolerance

LangGraph automatically saves a snapshot of the full graph state after every execution step, creating a chain of checkpoints that enables:
- **Fault-tolerant recovery**: If a node fails at step 8 of 10, resume from step 7 without re-running 1–7
- **Pause-and-resume**: Workflows can shut down entirely and resume hours or days later
- **Time-travel debugging**: Navigate backward and forward through execution history, inspect state, and fork from any checkpoint [AutoLearningAgents](https://www.autolearningagents.com/langgraph/checkpointing.php)

Checkpoint backends:
| Backend | Use Case | Durability |
|---------|----------|------------|
| `MemorySaver` | Development/testing only | Process-only; lost on restart |
| `PostgresSaver` | Production (recommended) | Full durability, crash recovery, horizontal scaling |
| `DynamoDBSaver` | AWS-native deployments | Serverless, pay-per-use |

[AutoLearningAgents](https://www.autolearningagents.com/langgraph/checkpointing.php)

### 5.4 Temporal.io: Durable Execution for Long-Running Agents

Temporal Workflows provide fundamentally different guarantees from checkpointing:
- **Automatic state persistence**: Every await point is automatically a checkpoint
- **Automatic failure recovery**: Durable reminders reactivate workflows after crashes without human intervention
- **Replay-based resumption**: On recovery, the workflow replays from the beginning, but completed activities return stored results from the event log
- **Distributed execution**: Workflows fan out across cluster nodes; node failures trigger automatic rebalancing [Diagrid Blog](https://www.diagrid.io/blog/checkpoints-are-not-durable-execution-why-langgraph-crewai-google-adk-and-others-fall-short-for-production-agent-workflows)

Temporal is already used in production by OpenAI for Codex. Temporal Cloud has processed over **9.1 trillion action executions**. [Fast.io](https://fast.io/resources/ai-agent-temporal-integration/)

### 5.5 Production Persistence Checklist

From Harbor Research's experience (41% of runs >90 minutes ended without terminal status before persistence fixes; reduced to 2.8% after): [Solana Garden](https://solana.garden/guides/llm-agent-durable-state-checkpointing-run-persistence-explained/?ref=seo-webhook-related)

1. Define persistence boundary (transcript, ledger, budgets, handles)
2. Append WAL events at step boundaries with monotonic sequence per run
3. Snapshot every K events or T minutes to object store or JSONB
4. Persist `ToolInvoked` before network; `ToolResult` before step advance
5. Issue resume tokens with epoch; enforce single-writer lease
6. Reconcile unknown tool outcomes before continuing write tools
7. Integrate with cancellation FSM and budget ledger atomically
8. Expose resumable status and last checkpoint time to clients
9. Graceful deploy drain with checkpoint before pod termination
10. Load-test `kill -9` mid-run; verify resume within SLA without duplicate side effects

### 5.6 Surreal-Memory Integration

For Prometheus's surreal-memory layer, the integration pattern is:
- Each thread writes its **partial knowledge graph** to surreal-memory at completion
- The **master graph** is stored in a separate surreal-memory namespace
- **Thread IDs** are used as graph namespaces during execution, then merged into the master namespace
- **Temporal/versioning fields** on every node/edge enable contradiction resolution (see Exabase M-1 pattern: 97.4% accuracy on knowledge update tasks) [Exabase](https://exabase.io/blog/what-is-memory-drift-in-ai-agents)

---

## 6. Parallel Research Decomposition Patterns

### 6.1 Map-Reduce over Sources

The canonical pattern for parallel research:
- **Map**: Decompose topic into sub-topics, assign each to a thread
- **Reduce**: Collect partial results, merge knowledge graphs, deduplicate citations, resolve contradictions

In LangGraph, this is implemented via the **Send API**: a supervisor node returns `[Send("researcher", sub_state) for sub_state in sub_states]`, triggering concurrent execution of all branches. Results are collected and merged before the next node runs. [Abstract Algorithms](https://abstractalgorithms.dev/langgraph-multi-agent-supervisor-pattern)

Wall-clock time reduction: from `Σ T_i` to `max(T_i)` — a 3× speedup when tasks take equal time. [Abstract Algorithms](https://abstractalgorithms.dev/langgraph-multi-agent-supervisor-pattern)

### 6.2 Decomposition Techniques

From parallel computing theory, applicable to research decomposition:
- **Recursive decomposition**: Break the research question into sub-questions; break each sub-question further until atomic
- **Data decomposition**: Partition the source corpus (e.g., by domain, by time period, by geographic region)
- **Exploratory decomposition**: Decompose the search space for solutions and search in each subspace; choose among solutions
- **Speculative decomposition**: Launch alternative hypothesis branches in parallel while waiting for disambiguating evidence [Rice University Lecture Notes](https://www.clear.rice.edu/comp422/lecture-notes/comp422-534-2020-Lecture2-ConcurrencyDecomposition.pdf)

### 6.3 Topic → Sub-Topics → Threads → Merge Pipeline

For deep research, the recommended decomposition is:

```
User Query
    ↓
Planner (L1 Tactical Loop) → Decomposes into Research Brief
    ↓
Supervisor (L2 Strategic Loop) → Spawns N research threads
    ↓
Thread 1: Sub-question A + Evidence Set A + Partial Graph A
Thread 2: Sub-question B + Evidence Set B + Partial Graph B
Thread 3: Sub-question C + Evidence Set C + Partial Graph C
    ↓
Aggregator → Merge graphs, deduplicate, resolve contradictions
    ↓
Verifier → Quality gate (source check, coverage, risk)
    ↓
Writer → Synthesize into .research package
    ↓
Karpathy Loop (L3 Outer Loop) → Ingest into KB
```

This maps directly to the existing Prometheus 10-stage pipeline, with stages 2–4 (Search→Retrieve→Collect) running in parallel across threads, and stages 5–10 (Verify→Resolve→Graph→Cite→Report→Export) running as merge and synthesis stages.

### 6.4 Pipeline-Style Document Writing (MiniMax Pattern)

For the final report generation, MiniMax's pipeline pattern is instructive:
- **Planner**: Defines document goal and structure
- **Writer**: Produces the body
- **Formatter**: Handles layout and file objects
- **Evaluator**: Independently checks content, formatting, and file integrity

This turns document generation from one-shot text generation into a CI/CD-like build pipeline: each step produces an intermediate artifact, each step has checks, and each step can be retried locally on failure. [MiniMax Release Notes](https://releasebot.io/updates/minimax)

---

## 7. Feynman Loop Recursion Extended to Parallel Research

### 7.1 The Core Mapping

The Feynman Learning Loop is PMPO-based: **Explain → Grade → Gap → Recurse**. When extended to parallel research:
- **Explain**: A research thread synthesizes its findings into an explanation of its sub-topic
- **Grade**: A verifier grades the explanation for completeness, accuracy, and source quality
- **Gap**: Identified gaps become new sub-questions
- **Recurse**: Each gap can spawn a new research thread (parallel recursive decomposition)

### 7.2 Recursion Floor and Horizontal Escalation

In the Feynman loop, `recursion_floor` controls depth-first drilling (how many levels of "explain this concept" before stopping), and `horizontal_escalation` controls breadth-first expansion (how many related concepts to explore at each level). [Prometheus Context Document]

Applied to research:
- **Depth-first research**: A thread hits a complex concept and recursively decomposes it into sub-concepts (recursion_floor limits the depth)
- **Breadth-first research**: When multiple gap concepts are identified at the same level, they are researched simultaneously across multiple threads (horizontal_escalation limits the breadth)

### 7.3 Parallel Gap Research

When a verifier identifies multiple gaps in a research thread's output, instead of serially researching each gap, the supervisor can spawn **parallel gap threads**:

```
Thread A completes research on "Neural architecture search"
    ↓
Verifier grades: finds gaps in (1) "Differentiable architecture search" 
                 and (2) "Hardware-aware NAS"
    ↓
Supervisor spawns:
    Thread A1: Research "Differentiable architecture search"
    Thread A2: Research "Hardware-aware NAS"
    ↓
Both threads run in parallel
    ↓
Results merge back into Thread A's knowledge graph
    ↓
Thread A re-synthesizes with filled gaps
```

This is analogous to the **recursive decomposition** pattern in parallel computing, where each level of recursion can be parallelized across threads. [Rice University Lecture Notes](https://www.clear.rice.edu/comp422/lecture-notes/comp422-534-2020-Lecture2-ConcurrencyDecomposition.pdf)

### 7.4 Integration with Learner Model

The learner-model Rust crate can track "research skills" as concepts to master. Each research thread's execution generates learning data:
- **Concept**: "Parallel source verification"
- **Mastery**: Increases when the thread successfully identifies and cites conflicting evidence
- **Gap**: If the thread misses counter-evidence, the gap feeds back into the learning plan

---

## 8. KBD Orchestrator L1/L2/L3 Managing Parallel Research Streams

### 8.1 Mapping the Four Nested Layers

The Prometheus Loops Architecture has four nested layers:
- **L0**: Harness micro-loop (tight execution, tool calling, immediate feedback)
- **L1**: Tactical KBD loop (planning, error recovery, sub-task decomposition)
- **L2**: Strategic evolver loop (model refinement, strategy adjustment, pattern learning)
- **L3**: Outer standing loop (continuous improvement, meta-learning, long-term adaptation) [Prometheus Context Document]

For parallel research, each layer maps to a specific orchestration concern:

| Layer | Loop Concern | Parallel Research Application |
|-------|-------------|--------------------------------|
| L0 | Tool execution, immediate feedback | Per-thread tool calling (search, fetch, browse) |
| L1 | Planning, sub-task decomposition | Research brief generation, thread spawning, merge planning |
| L2 | Strategy adjustment, pattern learning | Dynamic decomposition strategies based on topic complexity |
| L3 | Meta-learning, long-term adaptation | Research methodology improvement, research skill mastery tracking |

### 8.2 L1 Tactical Loop: Thread Spawning and Merge Planning

The L1 loop is where the research brief is decomposed into parallel threads. The `current-waypoint.json` + `progress.json` pattern already used by the KBD orchestrator maps directly to:
- `current-waypoint.json`: The current research question, assigned thread ID, and status
- `progress.json`: Aggregate progress across all threads, including completed sub-questions, collected evidence counts, and merge status

### 8.3 L2 Strategic Loop: Adaptive Decomposition

The L2 loop monitors research quality and adjusts the decomposition strategy:
- If threads are consistently returning thin evidence → increase `horizontal_escalation` (more parallel sources per sub-topic)
- If threads are consistently finding highly interrelated concepts → reduce `horizontal_escalation` and increase `recursion_floor` (deeper, narrower threads)
- If merge stage finds many contradictions → spawn additional verification threads

### 8.4 L3 Outer Standing Loop: Research Methodology Improvement

The L3 loop uses the Karpathy Loop (focus → reflect → ingest) to improve the research methodology itself:
- **Focus**: Read the `index.md` of the Karpathy LLM Wiki to inject relevant prior research patterns
- **Reflect**: After each research session, analyze what decomposition strategies worked and which failed
- **Ingest**: Write the learned patterns back to the KB (via `learn-kb`) and update the `deep-research` skill specification

---

## 9. Concurrency Limits and Resource Management

### 9.1 The Resource Contention Problem

Multi-agent systems consume significantly more resources than single-agent: agents use ~4× more tokens than chat interactions; multi-agent systems use ~15× more. That 15× multiplier comes from coordination overhead: context fetching, inter-agent communication, state synchronization, and LLM call latency multiplication. [SoftwareSeni](https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/)

Data retrieval for context assembly dominates execution time, eating 40–50% of execution time. [SoftwareSeni](https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/)

### 9.2 Token Duplication Waste

Peer-reviewed research on major multi-agent frameworks shows staggering token duplication rates:
- **MetaGPT**: 72% token duplication
- **CAMEL**: 86% token duplication
- **AgentVerse**: 53% token duplication

These redundant context-sharing patterns force systems to consume **1.5× to 7× more tokens than necessary**, directly translating to cascading resource contention across API rate limits, GPU infrastructure, and database operations. [Galileo AI](https://galileo.ai/blog/multi-agent-coordination-strategies)

### 9.3 Concurrency Control Strategies

From the MegaFlow distributed orchestration system (used for agent training workloads): [MegaFlow arXiv](https://arxiv.org/html/2601.07526v2)

1. **Three-tier limiting mechanism**:
   - User-specified parameters control rate of Model Service API calls
   - Distributed semaphores ensure task execution never exceeds available compute capacity
   - Administrative quotas provide control over resource usage, preventing system abuse while enabling fair sharing

2. **Ephemeral vs. Persistent tasks**:
   - **Ephemeral**: Provision dedicated compute instance, execute single task, immediately deallocate (perfect isolation, no contention)
   - **Persistent**: Maintain pool of persistent compute instances with pool-based allocation (resource reuse, containerized isolation) [MegaFlow arXiv](https://arxiv.org/html/2601.07526v2)

### 9.4 OS-Inspired Scheduling for LLM Agents

The **HiveMind** system proposes an OS-inspired approach: an Additive Increase / Multiplicative Decrease (AIMD) backpressure controller adapted from TCP congestion control. API latency serves as the congestion signal. When latency increases, concurrency is multiplicatively decreased; when latency is healthy, concurrency is additively increased. [HiveMind arXiv](https://arxiv.org/html/2604.17111v1)

This avoids the "thundering herd" problem where each agent retries independently during rate-limit windows, amplifying load. [HiveMind arXiv](https://arxiv.org/html/2604.17111v1)

### 9.5 Resource Management Best Practices

| Strategy | Implementation | Source |
|----------|---------------|--------|
| Budget allocation per thread | Assign `tokens_budget` per thread in state schema | [RapidClaw Blog](https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026) |
| Rate limiting per agent | Limit API calls per thread; use exponential backoff on 429 | [Galileo AI](https://galileo.ai/blog/multi-agent-coordination-strategies) |
| Queue management | Buffer requests during peak load; process as resources become available | [Tetrate](https://tetrate.io/learn/ai/multi-agent-systems) |
| Backpressure | Downstream agents signal upstream to slow down when overwhelmed | [Tetrate](https://tetrate.io/learn/ai/multi-agent-systems) |
| Semantic caching | 70% cache hit rate reduces overhead from 50% to 30% | [SoftwareSeni](https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/) |
| Context pruning | Share only necessary information between threads | [SoftwareSeni](https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/) |
| Elastic scaling | Adjust resource allocation dynamically based on load | [Tetrate](https://tetrate.io/learn/ai/multi-agent-systems) |

### 9.6 Cost-Performance Trade-off

The critical question: when is the overhead justified? Anthropic's multi-agent research system provides the benchmark: using Claude Opus 4 + Sonnet 4 in multi-agent configuration **outperformed single-agent Claude Opus 4 by 90.2%**—despite 15× token consumption. When capability gains exceed overhead cost, multi-agent makes sense. [SoftwareSeni](https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/)

---

## 10. LangGraph Parallel Agent Execution for Research

### 10.1 The Supervisor Pattern

LangGraph's supervisor pattern puts one node in charge of routing. It receives the user request, inspects current state, picks a worker to delegate to, reads the worker's output, and either delegates again or terminates. [BuildMVPFast](https://www.buildmvpfast.com/blog/langgraph-supervisor-deep-agents-multi-agent-patterns-2026)

For research, the supervisor pattern is typically structured as:
```
Supervisor → Researcher → Fact_Checker → Writer → Supervisor → ... → FINISH
```

Key production features:
- **Checkpointing with time travel**: Every state transition persisted; resume from crash at step 6 without re-running 1–5
- **Human-in-the-loop breakpoints**: Mark specific edges requiring human approval before continuing
- **LangSmith observability**: Every LLM call, tool invocation, and state transition traced [BuildMVPFast](https://www.buildmvpfast.com/blog/langgraph-supervisor-deep-agents-multi-agent-patterns-2026)

### 10.2 The Send API: Dynamic Fan-Out

The `Send` API lets the supervisor fan out to multiple workers simultaneously instead of sequentially. This eliminates serial overhead when workers are independent:

```python
from langgraph.types import Send

def supervisor_fanout_node(state: ResearchState) -> list[Send]:
    tasks = [
        Send("researcher", {**state, "sub_question": "Quantum computing hardware 2025"}),
        Send("researcher", {**state, "sub_question": "Enterprise quantum adoption 2025"}),
        Send("researcher", {**state, "sub_question": "Quantum error correction 2025"}),
    ]
    return tasks
```

Returning a list of `Send` objects triggers LangGraph to execute all branches concurrently. Results are collected and merged before the next node runs. [Abstract Algorithms](https://abstractalgorithms.dev/langgraph-multi-agent-supervisor-pattern)

### 10.3 Subgraph-as-Node Composition

For deep research, LangGraph supports **subgraph composition**: compiled child graphs plug directly into parent graphs. The parent's `AgentState` is a superset of child states, so shared keys flow across boundaries with no transform layer. [GitHub: langgraph-deep-research-agent](https://github.com/prasanna7401/langgraph-deep-research-agent)

This enables hierarchical decomposition:
- **Scope subgraph**: Clarifies user request, writes structured `ResearchBrief`, gates on human-in-the-loop review
- **Supervisor subgraph**: Spawns and coordinates parallel researcher threads
- **Final report generation**: Async node using `ResearchBrief` + joined `notes`

### 10.4 Map-Reduce over Sources (LangGraph Pattern)

LangGraph's built-in parallelism primitives (conditional edges + `Send`) support map-reduce style research:
- **Map**: A router node produces a list of subtasks; returns `Send("coder", sub_state)` for each subtask
- **Reduce**: An aggregator node waits for results and reduces them into a unified output
- **Subgraphs**: Each researcher can be modeled as a subgraph; the parent graph invokes many subgraph nodes at once; the Pregel runtime executes them concurrently per super-step [LangChain Forum](https://forum.langchain.com/t/parallel-execution-with-supervisor-pattern/1665)

### 10.5 Production State Schema for Research

A production `AgentState` for research threads should be:
```python
class ResearchThreadState(TypedDict):
    messages: Annotated[list[AnyMessage], add_messages]
    sub_question: str
    evidence: list[Source]  # bounded by policy
    partial_graph: dict
    status: Literal["running", "awaiting_tool", "done", "error"]
    tokens_used: int
    tokens_budget: int
    thread_id: str
    parent_thread_id: str | None
```

Rule: **Keep state SMALL**—no binary blobs, no giant payloads. Store large artifacts in S3 and keep only the URI in state. [RapidClaw Blog](https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026)

---

## 11. Thread Synchronization and Cross-Thread Citation Discovery

### 11.1 The Synchronization Problem

When research threads run in parallel, they may independently discover the same sources, reach contradictory conclusions, or identify evidence that supports another thread's claims. Without synchronization, the final merge stage is overwhelmed by duplication and conflict resolution. [MiniMax Release Notes](https://releasebot.io/updates/minimax)

### 11.2 Shared Discovery Registry

A **shared discovery registry** (implemented as a Redis cache, surreal-memory table, or simple filesystem lock file) enables cross-thread awareness:
- When Thread A discovers Source X, it writes `(source_url, thread_id, timestamp)` to the registry
- When Thread B searches, it checks the registry before executing its own search
- If Source X is already being processed by Thread A, Thread B can either skip it or request a summary from Thread A's partial results

This reduces the token duplication problem (72–86% in naive multi-agent systems) by preventing redundant source retrieval. [Galileo AI](https://galileo.ai/blog/multi-agent-coordination-strategies)

### 11.3 Cross-Thread Citation Graph

The `.research` package should include a **cross-thread citation graph**:
- Nodes: Claims/facts
- Edges: Supports / Contradicts / Cites / RelatedTo
- Thread provenance: Each edge is tagged with the `thread_id` that discovered it
- Confidence: Aggregated confidence across threads (e.g., 3 threads independently confirming = high confidence; 1 thread claiming with no corroboration = low confidence)

### 11.4 Synchronization Patterns

| Pattern | Use Case | Implementation |
|---------|----------|----------------|
| **Shared memory** | Real-time cross-thread awareness | Redis / surreal-memory with pub/sub |
| **Message passing** | Async notification between threads | Redis Streams / LangGraph Send API |
| **Checkpoint-based** | Periodic sync at merge points | Write to shared state at each checkpoint |
| **Event-driven** | React to new evidence discovery | Event bus triggering verifier re-check |

[Redis Blog](https://redis.io/blog/ai-agent-orchestration/)

### 11.5 Verifier as Synchronization Point

MiniMax's adversarial verifier pattern is an effective synchronization mechanism:
- After all threads complete, the verifier independently checks:
  - Source verifiability (stable URLs)
  - Staleness (is the source current?)
  - Counter-evidence (does any thread's finding contradict another?)
  - Coverage (did all sub-questions get answered?)
- If the verifier finds gaps or contradictions, it signals the supervisor to spawn additional threads
- This creates a **feedback loop** that converges on high-quality answers [MiniMax Release Notes](https://releasebot.io/updates/minimax)

### 11.6 Preventing Race Conditions

In distributed orchestration, sub-millisecond state access is required to prevent race conditions. Redis and distributed caches provide semantic caching with 70% cache hit rates, reducing LLM costs by 70%. Vector search provides 100% recall accuracy for semantic memory retrieval. Pub/sub messaging supports real-time coordination across distributed agents with sub-millisecond latency. [Redis Blog](https://redis.io/blog/ai-agent-orchestration/)

---

## 12. Recommendations for Prometheus Deep-Research Integration

### 12.1 Architecture Recommendation

Implement a **two-layer architecture** for the Prometheus deep-research pipeline:

**Layer 1: Temporal/LangGraph Durable Execution (Orchestration)**
- Temporal.io handles workflow durability, retries, and long-running session management
- LangGraph handles the agent state machine, parallel thread execution, and checkpointing
- This separation keeps "did this complete?" concerns distinct from "what should the agent do next?" concerns [CSDN Blog](https://www.cnblogs.com/lightsong/p/19530436)

**Layer 2: Prometheus-Specific Components**
- **Feynman Loop Integration**: Each research thread's output is fed through `learn-grade` to assess explanation quality; gaps spawn new research threads
- **Karpathy Wiki Integration**: Research findings are written as wiki pages; the `index.md` is updated with new entries; contradictions are flagged in `log.md`
- **Learner Model Tracking**: Research methodology (source verification, synthesis, citation formatting) is tracked as learnable concepts in the learner-model Rust crate
- **Surreal-Memory Backend**: Thread states stored in surreal-memory with temporal versioning; master graph in separate namespace

### 12.2 Skill Specification Updates

The `deep-research` skill should be enhanced with:
1. **Thread spawning primitives**: `spawn_research_thread(sub_question, context, tools)`
2. **Merge stage**: `merge_threads(thread_results)` with built-in deduplication and contradiction resolution
3. **Checkpoint format**: `.research` packages should include per-thread `partial_graph.json` and `evidence.json`
4. **Cross-thread citation graph**: `citations.json` with thread provenance and confidence scoring
5. **Resource limits**: `max_threads`, `tokens_per_thread`, `max_depth` (recursion_floor), `max_breadth` (horizontal_escalation)

### 12.3 Integration with Existing Prometheus Skills

| Existing Skill | Integration Point |
|----------------|-------------------|
| `feynman-loop` | Research thread output → explain step → grade → gap → recurse (spawn new threads) |
| `learn-survey` | Research findings generate diagnostic questions for prior knowledge assessment |
| `learn-plan` | Research scope (sub-question count, evidence depth) feeds curriculum complexity estimates |
| `learn-kb` | `.research` packages become knowledge sources; research output ingested via `pk ingest` |
| `learn-grade` | Verifier acts as grading engine for research thread quality |
| Karpathy Loop | `focus` injects research context from wiki; `reflect` + `ingest` writes findings back |

---

## 13. Sources & Citations

### Primary Sources (AI Agent Systems)

1. **Kimi K2.6 Tech Blog** — Moonshot AI. "Kimi K2.6 Tech Blog: Advancing Open-Source Coding." April 20, 2026. https://www.kimi.com/blog/kimi-k2-6

2. **Kimi K2.6 Complete Guide** — AIMadeTools. "Kimi K2.6 Complete Guide — Open-Source Agentic Model With 300 Sub-Agents." April 21, 2026. https://www.aimadetools.com/blog/kimi-k2-6-complete-guide/

3. **Kimi K2.6 Explained** — Miraflow. "Kimi K2.6 Explained: Moonshot AI's Open-Source Model That Ties GPT-5.5 on Coding." April 29, 2026. https://miraflow.ai/blog/kimi-k2-6-explained-moonshot-ai-open-source-model-ties-gpt-5-5-coding

4. **MiniMax Agent Team Blog** — MiniMax. "MiniMax Agent Team: Built for Long-Running Tasks and Continuous Evolution." May 27, 2026. https://www.minimax.io/blog/minimax-agent-team-long-running-1779893953

5. **MiniMax Release Notes** — ReleaseBot. "MiniMax Release Notes - June 2026 Latest Updates." June 19, 2026. https://releasebot.io/updates/minimax

6. **Claude Code Subagents Guide** — Builder.io. "Claude Code Subagents: How to Create, Use, and Debug Them." April 16, 2026. https://www.builder.io/blog/claude-code-subagents

7. **Claude Code Parallel Subagents** — ClaudeCodeGuides. "Claude Code Parallel Subagents — Best Practices." Updated April 17, 2026. https://claudecodeguides.com/parallel-subagents-claude-code-best-practices-2026/

8. **Claude Code Subagent Best Practices** — ClaudeFast. "Claude Code Sub-Agents: Parallel vs Sequential Patterns." July 3, 2026. https://claudefa.st/blog/guide/agents/sub-agent-best-practices

9. **Tim Dietrich Blog** — "How to Use Claude Code Sub-Agents for Parallel Work." January 17, 2026. https://timdietrich.me/blog/claude-code-parallel-subagents/

10. **Anthropic Subagent PDF** — Resources.anthropic.com. "Claude Code Advanced Patterns: Subagents, MCP, and Scaling to Real Codebases." https://resources.anthropic.com/hubfs/Claude%20Code%20Advanced%20Patterns_%20Subagents,%20MCP,%20and%20Scaling%20to%20Real%20Codebases.pdf

### LangGraph & Technical Frameworks

11. **LangGraph Supervisor Pattern** — BuildMVPFast. "LangGraph Supervisor and Deep Agents: Production Multi-Agent Patterns." May 17, 2026. https://www.buildmvpfast.com/blog/langgraph-supervisor-deep-agents-multi-agent-patterns-2026

12. **Abstract Algorithms** — "Multi-Agent Systems in LangGraph: Supervisor Pattern, Handoffs, and Agent Networks." March 28, 2026. https://abstractalgorithms.dev/langgraph-multi-agent-supervisor-pattern

13. **LangGraph Deep Research Agent** — GitHub: prasanna7401. https://github.com/prasanna7401/langgraph-deep-research-agent

14. **LangGraph Checkpointing** — AutoLearningAgents. "LangGraph Checkpointing and Time-Travel Debugging." May 31, 2026. https://www.autolearningagents.com/langgraph/checkpointing.php

15. **LangGraph Persistence Docs** — LangChain. https://docs.langchain.com/oss/python/langgraph/persistence

16. **RapidClaw Blog** — "LangGraph Checkpointing with Postgres (2026)." May 7, 2026. https://rapidclaw.dev/blog/deploy-langgraph-production-tutorial-2026

17. **LangChain Forum** — "Parallel execution with supervisor pattern." September 27, 2025. https://forum.langchain.com/t/parallel-execution-with-supervisor-pattern/1665

18. **LangChain DeepAgents** — GitHub: langchain-ai/deepagents. https://github.com/langchain-ai/deepagents

### Durable Execution & State Management

19. **Temporal.io Blog** — "Temporal Sandbox Orchestration Harness: The missing layer for running agents." May 7, 2026. https://temporal.io/blog/temporal-sandbox-orchestration-harness-the-missing-layer-for-running-agents

20. **Temporal + LangGraph Architecture** — CSDN/Lightsong. "Temporal + LangGraph: A Two-Layer Architecture for Multi-Agent Coordination." January 25, 2026. https://www.cnblogs.com/lightsong/p/19530436

21. **Temporal + Vercel AI SDK** — Temporal.io Blog. "Building durable agents with Temporal and AI SDK by Vercel." January 20, 2026. https://temporal.io/blog/building-durable-agents-with-temporal-and-ai-sdk-by-vercel

22. **Durable Execution Zylos** — Zylos Research. "Durable Execution for AI Agent Runtimes." April 24, 2026. https://zylos.ai/research/2026-04-24-durable-execution-agent-runtimes/

23. **Diagrid Blog** — "Why Checkpoints Aren't Durable Execution: LangGraph." February 25, 2026. https://www.diagrid.io/blog/checkpoints-are-not-durable-execution-why-langgraph-crewai-google-adk-and-others-fall-short-for-production-agent-workflows

24. **Temporal AI Agent Integration** — Fast.io. "How to Integrate AI Agents with Temporal Workflows." March 23, 2026. https://fast.io/resources/ai-agent-temporal-integration/

25. **LLM Agent Durable State** — Solana Garden. "LLM Agent Durable State, Checkpointing and Run Persistence Explained." June 12, 2026. https://solana.garden/guides/llm-agent-durable-state-checkpointing-run-persistence-explained/

### Context Management & Memory

26. **Context Window Management Zylos** — Zylos Research. "Context Window Management and Session Lifecycle for Long-Running AI Agents." March 31, 2026. https://zylos.ai/research/2026-03-31-context-window-management-session-lifecycle-long-running-agents/

27. **AI Agent Context Management** — Dev.to/bobrenze. "AI Agent Context Window Management: How I Handle Tasks That Take Longer Than My Memory." March 29, 2026. https://dev.to/bobrenze/ai-agent-context-window-management-how-i-handle-tasks-that-take-longer-than-my-memory-4b47

28. **Context Isolation Paper** — Rivista AI. "Context Isolation in Multi-Agent Systems." 2025. https://www.rivista.ai/wp-content/uploads/2025/11/2510.26493v1.pdf

29. **Memory Drift Exabase** — Exabase. "What is memory drift in AI agents?" June 5, 2026. https://exabase.io/blog/what-is-memory-drift-in-ai-agents

30. **Recursive Transformers** — Aman's AI Journal. "Recursive Language Models." 2021. https://aman.ai/primers/ai/recursive-transformers/

31. **jcode Memory Architecture** — GitHub: 1jehuang/jcode. "Memory Architecture: Sidecar Consolidation." January 27, 2026. https://github.com/1jehuang/jcode/blob/master/docs/MEMORY_ARCHITECTURE.md

### Resource Management & Orchestration

32. **Multi-Agent Coordination Galileo** — Galileo AI. "Multi-Agent Coordination Gone Wrong? Fix With 10 Strategies." April 8, 2025. https://galileo.ai/blog/multi-agent-coordination-strategies

33. **OS-Inspired Scheduling** — arXiv. "OS-Inspired Scheduling for Concurrent LLM Agent Workloads." April 15, 2026. https://arxiv.org/html/2604.17111v1

34. **Agent Orchestration Redis** — Redis.io. "AI agent orchestration for production systems." January 14, 2026. https://redis.io/blog/ai-agent-orchestration/

35. **Multi-Agent Systems Tetrate** — Tetrate. "Multi-Agent Systems: Design Patterns and Orchestration." December 7, 2025. https://tetrate.io/learn/ai/multi-agent-systems

36. **Agent Orchestration MindStudio** — MindStudio. "What Is Agent Orchestration? Why It's the Biggest Unsolved Problem in the AI Stack." April 7, 2026. https://www.mindstudio.ai/blog/agent-orchestration-biggest-unsolved-problem-ai-stack

37. **SoftwareSeni** — "Understanding Orchestration Patterns for Multi-Agent Systems." February 16, 2026. https://www.softwareseni.com/understanding-orchestration-patterns-for-multi-agent-systems-and-how-they-affect-performance-coordination-and-reliability/

38. **MegaFlow** — arXiv. "Large-Scale Distributed Orchestration System for the Agentic Era." January 2026. https://arxiv.org/html/2601.07526v2

### Knowledge Graph & Fusion

39. **Knowledge Graph Merging** — Meegle. "Knowledge Graph Merging." February 6, 2026. https://www.meegle.com/en_us/topics/knowledge-graphs/knowledge-graph-merging

40. **Graph Matching Fusion** — Research Square. "Algorithm Design and Optimization for Knowledge Fusion: A Graph Matching-based Approach." https://www.researchsquare.com/article/rs-4641408/v1.pdf?c=1721742069000

### Parallel Computing Theory

41. **Rice University Lecture Notes** — "Principles of Parallel Algorithm Design: Concurrency and Decomposition." 2020. https://www.clear.rice.edu/comp422/lecture-notes/comp422-534-2020-Lecture2-ConcurrencyDecomposition.pdf

42. **Washington University Lecture** — "Introduction to Parallel Algorithms." 2013. https://courses.cs.washington.edu/courses/cse373/13wi/lectures/03-13/26-parallel-algorithms.pdf

43. **Database System Concepts** — "Chapter 22: Parallel and Distributed Query Processing." https://www.db-book.com/slides-dir/PDF-dir/ch22.pdf

### Academic Papers & Research

44. **Reinforcement Learning for Multi-Agent Systems** — arXiv. "Reinforcement Learning for LLM-based Multi-Agent Systems through Orchestration Traces." May 4, 2026. https://arxiv.org/html/2605.02801v1

45. **Recursive Agent Orchestration** — arXiv. "Recursive Agent Orchestration (RAO)." 2026. https://arxiv.org/pdf/2605.06639

46. **ORCH Multi-Agent Orchestrator** — Frontiers in AI. "ORCH: many analyses, one merge—a deterministic multi-agent orchestrator." February 2, 2026. https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2026.1748735/full

47. **Step-DeepResearch** — arXiv. "Step-DeepResearch Technical Report." December 23, 2025. https://arxiv.org/html/2512.20491v2

48. **LangGraph Thesis** — Charles University. "MetaGraph for Agentic Workflows." 2026. https://dspace.cuni.cz/bitstream/handle/20.500.11956/202841/120519071.pdf

---

*End of Report*
