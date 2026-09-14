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
| `graph_json` | object | Full graph: `{topics[], claims[], relations[]}` (`references/schemas/research-graph.schema.json`) |
| `claim_count` | int | Claims after content-addressed merging |
| `relation_count` | int | `cites` and `contradicts` relations |
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

6. **Export graph** — write `<package_id>/graph.json` with the builder:

   ```bash
   bash scripts/build-graph.sh <package_dir> [--critical critical-claims.txt] --out <package_dir>/graph.json
   ```

   It reads `sources/registry.json`, `sources/credibility.json` (labels,
   evidence, scores), and `contradictions.json`, and emits the shape of
   `references/schemas/research-graph.schema.json`: `topics[]`, `claims[]`,
   `relations[]`.

   - **Claim ids are content-addressed:** `claim-` + the first 16 hex characters
     of `sha256("<package_id>:<normalised text>")`, where normalisation
     lower-cases, collapses whitespace, and strips trailing punctuation. The same
     sentence from two chunks or two sources is therefore **one** claim: its
     `sources[]` are the union and its `label` is the higher of the copies
     (`verified` > `inferred` > `unverified` > `blocked`) with that copy's
     evidence. The ids match the ones `detect-contradictions.sh` wrote.
   - Every claim carries `label` (from `credibility.json`; a claim no source
     states is `inferred`), `critical` (`true` for claims listed in
     `--critical`, one text per line, or marked `critical` in the registry;
     Stage 09 may only promote a claim that is already critical here),
     `confidence` (the best source's credibility, capped at 0.5 for
     `unverified`), `sources[]`, `contradicts[]`, and `evidence` for `verified`
     and `blocked` labels.
   - Relations are `cites` (claim → source id) and `contradicts` (claim ↔
     claim, one relation per resolved or unresolved pair in
     `contradictions.json`). Topics group the claims of each contradiction
     topic, then the remaining claims by source domain.

   surreal-memory entities (steps 1–5) mirror this file; if surreal-memory is
   unavailable the file is the graph. The driver still tolerates the legacy
   `{nodes, edges}` shape from older packages, which derives to `unverified`
   because it carries no labels.

## Integration

`surreal-memory` MCP: `create_entity`, `create_relation`, `read_graph`
`scripts/build-graph.sh` writes `graph.json` (content-addressed claims, labels, `cites` and `contradicts`)
Entity types: `ResearchSource`, `Claim`, `Topic`
Relation types: `cites`, `contradicts`, `addresses`, `supports`

## Example

**Entities created:**
```
Topic: "throughput_qps" (id: topic-001)
Claim: "Qdrant sustains 500K QPS" (id: claim-3f2a9c1e0b7d4a55, label: verified, confidence: 0.77)
Claim: "Qdrant sustains 120K QPS" (id: claim-8b1d07e4c2f9a630, label: verified, confidence: 0.61)
ResearchSource: "qdrant.tech/benchmarks" (id: src-001, score: 77)
```

**Relations created:**
```
claim-3f2a9c1e0b7d4a55 → cites → src-001
claim-3f2a9c1e0b7d4a55 → contradicts → claim-8b1d07e4c2f9a630
```
