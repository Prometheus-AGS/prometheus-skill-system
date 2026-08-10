---
name: stage-02-search
description: >
  Deep research Stage 02 — Search. Executes web searches for each sub-question
  from the research plan. Deduplicates URLs, filters low-quality domains, and
  produces a ranked source list for Stage 03 retrieval.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-02, search, web-search, source-discovery]
---

# Stage 02 — Search

## Purpose

Execute targeted web searches for each sub-question in the research plan.
Produce a deduplicated, domain-filtered list of source URLs ranked by relevance
and quality signal. This list is the input to Stage 03 (Retrieve).

## Input

| Field | Type | Description |
|-------|------|-------------|
| `plan` | object | Research plan from Stage 01 |
| `sub_questions` | string[] | Questions to search |
| `search_strategy` | object | Keywords and filters per sub-question |
| `depth` | enum | Controls result count: shallow=10, deep=25, exhaustive=50 per question |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `source_urls` | string[] | Deduplicated, ranked URLs |
| `search_metadata` | object[] | Per-URL: search query used, rank, domain, snippet |
| `total_found` | int | Total URLs before deduplication |
| `filtered_count` | int | URLs removed by domain filter |

## Instructions

1. **Execute searches in batches** — for each sub-question, run 2–3 search
   queries using the strategy from Stage 01. Use `firecrawl_search` as primary;
   fall back to `tavily_search` if Firecrawl is unavailable.

2. **Domain filtering** — exclude: social media (twitter.com, reddit.com),
   content farms (ehow.com, answers.com), paywalled results that cannot be
   retrieved, and duplicates of already-indexed domains unless the content is
   demonstrably different.

3. **Deduplicate** — canonical URL normalization (strip tracking params, trailing
   slashes). Keep the highest-ranked copy when duplicates exist.

4. **Rank by quality signal** — prefer: `.edu`, `.gov`, `.org` academic domains;
   official vendor documentation; known-quality publications; content with visible
   authorship and date. Deprioritize undated content and AI-generated aggregators.

5. **Emit source list** — write `<job_id>/sources/url-list.json` with ranked URLs
   and metadata. Return the list as JSON for Stage 03.

## Integration

Primary: `firecrawl_search` (returns richer content with full-page extraction)
Fallback: `tavily_search`
Both require API key environment variables: `FIRECRAWL_API_KEY` / `TAVILY_API_KEY`.
At least one must be set or this stage fails with a clear error.

## Example

**Input query:** "Which vector databases are production-battle-tested as of 2025-2026?"

**Search queries executed:**
```
"vector database production 2025 benchmark"
"Pinecone Qdrant Weaviate Chroma production comparison"
"vector database reliability failure modes production"
```

**Output (excerpt):**
```json
[
  { "url": "https://qdrant.tech/benchmarks/", "domain": "qdrant.tech", "rank": 1 },
  { "url": "https://weaviate.io/blog/ann-benchmarks", "domain": "weaviate.io", "rank": 2 }
]
```
