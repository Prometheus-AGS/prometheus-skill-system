---
name: stage-07-graph
description: >
  Deep research Stage 07 — Graph. Builds a knowledge graph in surreal-memory
  from verified sources and resolved claims. Creates entities for topics, claims,
  and sources, with typed relations between them. Exports graph.json for the
  .research package.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-07, knowledge-graph, surreal-memory, entity-relations]
---

# Stage 07 — Graph

## Purpose

Build a structured knowledge graph from the verified, deduplicated, resolved
claim set. Store entities and relations in surreal-memory for cross-session
querying. Export the graph to `graph.json` in the `.research` package.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `source_registry` | object[] | Verified sources (Stage 04+05) |
| `resolved_claims` | object[] | Resolved claim set from Stage 06 |
| `credibility_scores` | object | `{url: score}` map |
| `job_id` | string | Used to scope entities in surreal-memory |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `graph_json` | object | Full graph: `{nodes[], edges[]}` |
| `node_count` | int | Total entities created |
| `edge_count` | int | Total relations created |
| `entity_ids` | object | `{name: surreal_memory_id}` for citation linking |

## Instructions

1. **Create topic entities** — for each resolved claim topic, create a `Topic`
   entity in surreal-memory: `create_entity(name=<topic>, entityType="Topic")`.

2. **Create claim entities** — for each resolved claim, create a `Claim` entity
   with observations: `[position, confidence, strategy, job_id]`.

3. **Link claims to sources** — `create_relation(from=<claim>, to=<source>, relationType="cites")`
   for each source that supports the claim.

4. **Link contradictions** — `create_relation(from=<claimA>, to=<claimB>, relationType="contradicts")`
   for the pre-resolution contradiction pairs.

5. **Link claims to topics** — `create_relation(from=<claim>, to=<topic>, relationType="addresses")`.

6. **Export graph** — call `read_graph` or reconstruct from created entities.
   Write `<job_id>/graph.json`. If surreal-memory is unavailable, build graph
   in-memory from the source registry and write to disk only.

## Integration

`surreal-memory` MCP: `create_entity`, `create_relation`, `read_graph`
`scripts/build-graph.sh` for offline graph construction fallback
Entity types: `ResearchSource`, `Claim`, `Topic`
Relation types: `cites`, `contradicts`, `addresses`, `supports`

## Example

**Entities created:**
```
Topic: "vector-db-throughput" (id: topic-001)
Claim: "Qdrant 500K QPS" (id: claim-001, confidence: 0.72)
ResearchSource: "qdrant.tech/benchmarks" (id: src-001, score: 77)
```

**Relations created:**
```
claim-001 → cites → src-001
claim-001 → addresses → topic-001
```
