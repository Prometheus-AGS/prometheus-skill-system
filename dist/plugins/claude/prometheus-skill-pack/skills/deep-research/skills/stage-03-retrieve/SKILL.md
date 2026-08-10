---
name: stage-03-retrieve
description: >
  Deep research Stage 03 — Retrieve. Fetches full page content for each source
  URL from Stage 02 and chunks it into retrievable segments. Handles PDF, HTML,
  and structured data formats. Outputs content chunks for Stage 04 indexing.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-03, retrieve, content-extraction, chunking]
---

# Stage 03 — Retrieve

## Purpose

Fetch the full content of each source URL and split it into chunks of ≤2K tokens.
Handle multiple content types (HTML, PDF, JSON). Skip unreachable or paywalled
sources gracefully. Output is a structured list of content chunks for Stage 04.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `source_urls` | string[] | Ranked URL list from Stage 02 |
| `search_metadata` | object[] | Per-URL metadata from Stage 02 |
| `token_budget` | object | Token allocation for this stage |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `chunks` | object[] | Content chunks: `{url, chunk_id, text, token_count, content_type}` |
| `retrieved_count` | int | Sources successfully retrieved |
| `failed_count` | int | Sources that could not be retrieved |
| `failed_urls` | string[] | List of failed URLs with reason |

## Instructions

1. **Fetch in batches of 5** — retrieve content for 5 URLs concurrently.
   Respect a 10-second timeout per URL. Mark timeouts as failed.

2. **Content type handling:**
   - HTML → `firecrawl_scrape` with `formats: ["markdown"]` for clean extraction
   - PDF → attempt `firecrawl_scrape`; if unavailable, use kreuzberg extraction
   - JSON/structured → parse directly, extract relevant fields

3. **Chunk content** — split each retrieved document into chunks of ≤2000 tokens
   with 100-token overlap between adjacent chunks. Preserve paragraph boundaries
   where possible. Each chunk gets a `chunk_id` of `<domain>-<n>`.

4. **Skip gracefully** — if a URL returns 4xx/5xx, is paywalled (detected by
   lack of body content or login-wall patterns), or times out, log it to
   `failed_urls` and continue. Do not block the pipeline.

5. **Write output** — write all chunks to `<job_id>/sources/` as individual
   JSON files (`chunk-<n>.json`). Return chunk list for Stage 04.

## Integration

Primary: `firecrawl_scrape` for HTML/PDF extraction
Secondary: kreuzberg (`skills/document-extraction/kreuzberg`) for complex PDFs
Timeout: 10 seconds per URL
Max content per source: 50K tokens (truncate with warning if exceeded)

## Example

**Input URL:** `https://qdrant.tech/benchmarks/`

**Output chunks:**
```json
[
  {
    "url": "https://qdrant.tech/benchmarks/",
    "chunk_id": "qdrant.tech-0",
    "text": "Qdrant v1.8 achieves 98.5% recall at 1ms p99 latency...",
    "token_count": 1847,
    "content_type": "html"
  }
]
```
