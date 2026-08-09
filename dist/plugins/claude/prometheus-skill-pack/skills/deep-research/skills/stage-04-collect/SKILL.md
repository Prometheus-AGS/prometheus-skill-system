---
name: stage-04-collect
description: >
  Deep research Stage 04 — Collect. Indexes content chunks from Stage 03 into
  surreal-memory as ResearchSource entities. Extracts key claims per chunk for
  downstream verification and graph building.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-04, collect, indexing, surreal-memory, entity-extraction]
---

# Stage 04 — Collect

## Purpose

Index all retrieved content chunks into surreal-memory as structured entities.
Extract key claims from each chunk. Produce the indexed source registry that
Stage 05 uses for credibility verification and Stage 07 uses for graph building.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `chunks` | object[] | Content chunks from Stage 03 |
| `job_id` | string | Research job identifier for scoping |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `source_registry` | object[] | `{url, entity_id, claims[], chunk_ids[]}` per source |
| `indexed_count` | int | Sources successfully indexed |
| `claim_count` | int | Total claims extracted |

## Instructions

1. **Group chunks by source URL** — aggregate all chunks from the same domain
   into a single source entity.

2. **Create surreal-memory entities** — for each unique source, call:
   ```
   create_entity(
     name = <url>,
     entityType = "ResearchSource",
     observations = [
       "job_id:<job_id>",
       "domain:<domain>",
       "chunk_count:<n>",
       "retrieved_at:<timestamp>"
     ]
   )
   ```

3. **Extract claims** — for each chunk, identify 1–5 factual claims (statements
   that can be true or false, not opinions). Store claims as additional
   observations on the entity.

4. **Handle surreal-memory unavailability** — if surreal-memory MCP is not
   connected, store the source registry in `<job_id>/sources/registry.json`
   on disk. Pipeline continues; cross-session querying will not be available.

5. **Write registry** — emit `<job_id>/sources/registry.json` for Stages 05–07.

## Integration

Primary: `surreal-memory` MCP (`create_entity`, `add_observations`)
Fallback: disk-based JSON registry at `<job_id>/sources/registry.json`
Entity type: `ResearchSource`
Claim relation type: `supports` (created in Stage 07)

## Example

**Input chunk:** `{ "url": "https://qdrant.tech/benchmarks/", "text": "Qdrant achieves 98.5% recall..." }`

**surreal-memory call:**
```
create_entity(
  name="https://qdrant.tech/benchmarks/",
  entityType="ResearchSource",
  observations=["job_id:vdb-rag-001", "domain:qdrant.tech", "claim:98.5% recall at 1ms p99"]
)
```
