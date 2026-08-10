# surreal-memory Integration

surreal-memory is an optional but recommended integration. When connected
(MCP tools `create_entity`, `add_memory`, `semantic_search` etc. are available),
the pipeline stores research assets as persistent knowledge graph entities.
When unavailable, all data falls back to disk-only JSON files.

## Entity Types Used

| Entity type | Created in stage | Purpose |
|-------------|-----------------|---------|
| `ResearchSource` | Stage 04 | Indexed web source with credibility metadata |
| `Claim` | Stage 04 | Factual claim extracted from a source |
| `Topic` | Stage 07 | Research topic grouping related claims |

## Relation Types Used

| Relation | From → To | Created in stage | Meaning |
|----------|-----------|-----------------|---------|
| `cites` | Claim → ResearchSource | Stage 07 | This claim is supported by this source |
| `contradicts` | Claim → Claim | Stage 07 | These two claims conflict |
| `addresses` | Claim → Topic | Stage 07 | This claim speaks to this topic |
| `supports` | ResearchSource → Claim | Stage 07 | Alternative direction of cites |

## Stage 04 — Collect

```
create_entity(
  name = <url>,
  entityType = "ResearchSource",
  observations = [
    "job_id:<job_id>",
    "domain:<domain>",
    "chunk_count:<n>",
    "retrieved_at:<ISO8601>"
  ]
)
```

For each extracted claim:
```
add_observations(entityName=<url>, observations=["claim:<claim_text>"])
```

## Stage 07 — Graph

Topic entity:
```
create_entity(name=<topic>, entityType="Topic", observations=["job_id:<job_id>"])
```

Claim entity:
```
create_entity(
  name = <claim_text_truncated_64_chars>,
  entityType = "Claim",
  observations = ["confidence:<float>", "strategy:<strategy>", "job_id:<job_id>"]
)
```

Relations:
```
create_relation(from=<claim_name>, to=<source_url>, relationType="cites")
create_relation(from=<claimA_name>, to=<claimB_name>, relationType="contradicts")
create_relation(from=<claim_name>, to=<topic_name>, relationType="addresses")
```

## Stage 08 — Cite

```
add_memory(
  content = <formatted_citation>,
  user_id = <job_id>,
  metadata = {"url": <url>, "credibility": <score>, "style": <APA|MLA|...>}
)
```

## Cross-Session Querying

After export, to find prior research on a topic in future sessions:

```
semantic_search("vector databases production RAG")
search_memories(query="vector database", user_id=<job_id>)
palace_recall("vector databases") # if palace_ingest was used at export
```

## Availability Check

At Stage 04 start, the pipeline checks for surreal-memory availability:
```bash
# If no surreal-memory MCP tools present → disk-only mode
# manifest.json records: "surreal_memory_used": false
```

Disk fallback files:
- `<job_id>/sources/registry.json` — source registry
- `<job_id>/graph.json` — graph export (still written)
- `<job_id>/citations.json` — citation list
