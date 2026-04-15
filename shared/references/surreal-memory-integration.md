# Surreal-Memory Integration Guide

All skills in the Prometheus Skill Pack can leverage the **surreal-memory** MCP
server for persistent, cross-session, distributed state management. This
reference defines the integration contract used by every skill.

**Server**: [Prometheus-AGS/surreal-memory-server](https://github.com/Prometheus-AGS/surreal-memory-server)
**Transport**: HTTP SSE (`/mcp/sse`) or stdio
**Default URL**: `http://localhost:23001/mcp/sse`

---

## Available MCP Tools

### Knowledge Graph

| Tool               | Purpose                                          | Used By                       |
| ------------------ | ------------------------------------------------ | ----------------------------- |
| `create_entity`    | Create a named entity with type and observations | All skills (state objects)    |
| `create_entities`  | Batch entity creation                            | KBD (change inventory)        |
| `add_observations` | Append observations to existing entity           | Evolver (assessment findings) |
| `create_relation`  | Link two entities                                | KBD (change→phase relations)  |
| `create_relations` | Batch relation creation                          | Entity skills (schema graphs) |
| `read_graph`       | Retrieve full or filtered graph                  | Evolver (state reload)        |
| `search_entities`  | Text search across entities                      | All skills (discovery)        |
| `semantic_search`  | Vector-based similarity search                   | Evolver (landscape analysis)  |
| `delete_entity`    | Remove entity and its relations                  | All skills (cleanup)          |
| `delete_relation`  | Remove a specific relation                       | All skills (cleanup)          |

### Graph-RAG Traversal

| Tool               | Purpose                                   | Used By                          |
| ------------------ | ----------------------------------------- | -------------------------------- |
| `find_path`        | Shortest path between entities            | KBD (dependency tracing)         |
| `expand_neighbors` | Get entity's immediate graph neighborhood | Evolver (impact analysis)        |
| `get_related`      | Find entities related by type/relation    | GitOps (cluster→service mapping) |

### Scoped Memory (mem0-compatible)

| Tool                             | Purpose                                  | Used By                          |
| -------------------------------- | ---------------------------------------- | -------------------------------- |
| `add_memory`                     | Store scoped memory (user/session/agent) | All skills (cross-session state) |
| `get_memory`                     | Retrieve by ID                           | All skills                       |
| `search_memories`                | Search within scope                      | All skills (prior context)       |
| `hybrid_search_memories`         | BM25 + vector weighted search            | Evolver (landscape recall)       |
| `get_memory_history`             | Temporal history of a memory             | Evolver (iteration tracking)     |
| `compress_memories`              | Consolidate old memories                 | Long-running evolutions          |
| `add_memories_from_conversation` | Extract memories from chat               | Session persistence              |

### TaskStreams

| Tool                         | Purpose                              | Used By                          |
| ---------------------------- | ------------------------------------ | -------------------------------- |
| `create_task_stream`         | Named task context with token budget | KBD (per-phase stream)           |
| `add_to_task_stream`         | Append context/decisions/artifacts   | KBD, Evolver (progress tracking) |
| `get_context_for_task`       | Token-budgeted context retrieval     | All skills (context loading)     |
| `auto_summarize_task_stream` | Rolling summarization                | Long-running phases              |
| `archive_task_stream`        | Archive completed task               | KBD (phase completion)           |

### Mindmaps

| Tool                        | Purpose                         | Used By                          |
| --------------------------- | ------------------------------- | -------------------------------- |
| `create_mindmap`            | Create structured mindmap       | Evolver (analysis visualization) |
| `generate_ideation_mindmap` | Auto-generate from topic        | Planning phases                  |
| `export_mindmap`            | Export to Mermaid/Markdown/JSON | Reports                          |

---

## Detection Protocol

Every skill that supports surreal-memory MUST detect availability before use:

```
1. Check environment: $SURREAL_MEMORY_URL (explicit override)
2. Check MCP tools: is 'create_entity' available in the tool list?
3. Check .mcp.json: does it reference surreal-memory?
4. Fallback: use filesystem state (graceful degradation)
```

Skills MUST NOT fail if surreal-memory is unavailable. All integrations
are additive — filesystem state is always the fallback.

---

## State Mapping Patterns

### Iterative Evolver → Surreal-Memory

```
Evolution         → Entity (type: "evolution", name: evolution_name)
Assessment        → Entity (type: "assessment") + relation "assessed_by" → Evolution
Plan Item         → Entity (type: "plan_item") + relation "planned_for" → Evolution
Execution Result  → Observation on Plan Item entity
Reflection        → Entity (type: "reflection") + relation "reflects_on" → Evolution
```

### KBD Orchestrator → Surreal-Memory

```
Project           → Entity (type: "project", name: project_name)
Phase             → Entity (type: "phase") + relation "phase_of" → Project
Change            → Entity (type: "change") + relation "change_in" → Phase
Progress Update   → Observation on Change entity
Waypoint          → Memory (scope: session, type: procedural)
Constraint        → Entity (type: "constraint") + relation "constrains" → Project
```

### GitOps Skills → Surreal-Memory

```
Cluster           → Entity (type: "cluster", name: cluster_context)
Service           → Entity (type: "service") + relation "deployed_to" → Cluster
Deployment        → Entity (type: "deployment") + relation "deploys" → Service
Pipeline Run      → Entity (type: "pipeline_run") + relation "triggered_by" → Deployment
```

### Artifact Refiner → Surreal-Memory

```
Artifact          → Entity (type: "artifact", name: artifact_name)
Refinement Cycle  → Entity (type: "refinement") + relation "refines" → Artifact
Constraint Result → Observation on Refinement entity
```

---

## TaskStream Usage

For long-running operations, use TaskStreams instead of entities:

```yaml
# KBD phase execution
stream_name: 'kbd-{project}-{phase}'
model: 'claude-sonnet-4-6' # for token budgeting
entries:
  - type: 'context' # assessment, plan, constraints
  - type: 'decision' # tool selection, change ordering
  - type: 'artifact' # produced files, test results
  - type: 'progress' # task completion updates
```

TaskStreams auto-summarize when approaching token limits, preserving
the most relevant context for the active model.

---

## Cross-Skill Graph Queries

The knowledge graph enables powerful cross-skill queries:

```
# Find all services deployed by a specific evolution cycle
find_path(from: "evolution:api-improvement", to_type: "service")

# Get all changes that failed artifact-refiner QA
search_entities(query: "refinement FAIL", type: "refinement")

# Expand what a KBD phase produced
expand_neighbors(entity: "phase:phase-2-sales", relation_type: "change_in")
```

---

## Environment Variables

| Variable                   | Default                          | Purpose             |
| -------------------------- | -------------------------------- | ------------------- |
| `SURREAL_MEMORY_URL`       | `http://localhost:23001/mcp/sse` | Server URL          |
| `SURREAL_MEMORY_NAMESPACE` | `prometheus`                     | SurrealDB namespace |
| `SURREAL_MEMORY_DATABASE`  | `skillpack`                      | SurrealDB database  |
