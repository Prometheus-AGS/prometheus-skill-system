---
name: stage-01-planner
description: >
  Deep research Stage 01 — Planner. Decomposes a research query into structured
  sub-questions, a search strategy, and a token budget. Produces the research
  plan that drives all subsequent stages.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-01, planner, query-decomposition, research-plan]
---

# Stage 01 — Planner

## Purpose

Transform a raw research query into a structured research plan. The plan defines
the sub-questions to answer, the search strategy, stage sequence, token budget,
and confidence threshold. All subsequent stages consume this plan.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `query` | string | Raw research query from the user |
| `depth` | enum | `shallow` / `deep` / `exhaustive` |
| `kb_ids` | string[] | Optional knowledge base identifiers for grounding |
| `context` | string | Optional prior context from the session |

## Output

Writes `templates/research-plan.md` populated instance to `<job_id>/plan.md`.

| Field | Type | Description |
|-------|------|-------------|
| `sub_questions` | string[] | 3–10 specific questions that decompose the query |
| `search_strategy` | object | Keywords, site filters, date range |
| `stage_sequence` | string[] | Which stages to run (depth determines which) |
| `token_budget` | object | Per-stage token allocation |
| `confidence_threshold` | float | Minimum confidence for report delivery (default: 0.7) |

## Instructions

1. **Parse the query** — identify the core question, time horizon, geographic
   scope, and any implicit constraints (e.g. "production-ready only").

2. **Decompose into sub-questions** — generate 3–10 specific questions that,
   when answered, cover the full query. Think adversarially: what evidence
   would disprove the most likely answer?

3. **Build search strategy** — for each sub-question, identify 2–3 high-signal
   search terms. Prioritize primary sources (official docs, peer-reviewed papers,
   vendor announcements) over secondary aggregators.

4. **Assign token budget** — distribute the total token budget by stage weight:
   frontier stages (01, 05, 06, 07, 09) get 60%, medium stages (02, 03, 04) get
   30%, small stages (08, 10) get 10%.

5. **Write plan** — populate `templates/research-plan.md` and write to
   `<job_id>/plan.md`. Emit the plan as JSON to stdout for pipeline consumption.

## Integration

Uses `sequential_thinking` tool for adversarial decomposition when available.
Optionally grounds in KB via `palace_recall` (surreal-memory) or `dify_search`
(Dify KB) when `kb_ids` is set.

## Example

**Input:**
```json
{ "query": "Current state of vector databases for production RAG", "depth": "deep" }
```

**Output sub-questions:**
```
1. Which vector databases are production-battle-tested as of 2025-2026?
2. What are the latency and throughput benchmarks for Pinecone vs Qdrant vs Weaviate?
3. What are the operational failure modes most commonly reported?
4. How do pricing models compare at 10M vector scale?
5. What do major RAG framework maintainers recommend?
6. What are the migration paths between vector database providers?
```
