---
name: deep-research
description: >
  10-stage deep research pipeline: Planner → Search → Retrieve → Collect →
  Verify → Resolve → Graph → Cite → Report → Export. Produces persistent
  .research packages (OKF-aligned knowledge assets) with citations, confidence
  scores, knowledge graphs, and contradiction tracking. Integrates with
  surreal-memory, liter-llm, sycophancy-correction, and Feynman learning skills.
  Supersedes disposable report generation with structured knowledge infrastructure.
license: MIT
version: '1.0.0'
allowed-tools: file_system web_search code_interpreter sequential_thinking memory browser tavily firecrawl
model_routing:
  policy_source: liter-llm-bridge
  phases:
    research-plan: frontier
    research-search: medium
    research-retrieve: medium
    research-collect: medium
    research-verify: frontier
    research-resolve: frontier
    research-graph: frontier
    research-cite: small
    research-synthesize: frontier
    research-export: small
  routing_reference: references/model-routing.md
triggers:
  keywords:
    - research
    - deep research
    - investigate
    - analyze
    - deep dive
    - comprehensive report
    - what is the current state of
    - competitive analysis
    - market research
    - technology evaluation
    - literature review
    - knowledge synthesis
    - due diligence
    - study
  semantic: >
    Any request requiring synthesis from multiple web sources with citations,
    verification, and structured knowledge output. Research questions, competitive
    analysis, technology evaluations, due diligence, and any topic requiring
    evidence-backed findings across multiple sources.
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: research
  tags: [research, deep-research, knowledge-graph, okf, pipeline, citations, verification]
---

# deep-research

10-stage research pipeline that turns a query into a persistent, citable knowledge
asset — not a one-off answer.

## When to Use

Invoke this skill when the task requires:

- Synthesis from multiple independent web sources (3+ sources)
- Citations with confidence scores and source credibility ratings
- Detection and resolution of contradictions across sources
- A knowledge graph linking entities, claims, and supporting evidence
- Persistent output you can query later (`.research` package)
- Competitive intelligence, technology landscape, or market research
- Academic-style literature review or due diligence
- Any research question where a hallucinated answer would be harmful
- Long-horizon analysis requiring staged evidence accumulation
- Content that must pass a quality gate before delivery

Do NOT use for:
- Single-source lookups (use web_search directly)
- Quick factual Q&A with no citation requirement
- Tasks where a disposable answer is acceptable

## Quick Start

```
/deep-research <query>
/research <query>
```

**With depth control:**
```
/deep-research --depth shallow "What is Temporal's architecture?"
/deep-research --depth deep "Current state of Rust async runtimes"
/deep-research --depth exhaustive "Regulatory landscape for AI in healthcare EU 2024-2025"
```

**With knowledge base grounding:**
```
/deep-research --kb local:/path/to/docs "How does our system compare to competitors?"
/deep-research --kb dify:my-kb "Summarize our patent landscape"
```

**Depth levels:**
- `shallow` — Stages 1-5, ~20 sources, ~30 min
- `deep` (default) — All 10 stages, ~50 sources, ~60 min
- `exhaustive` — All 10 stages with extended search, ~100+ sources, ~2 hr

## Background Execution (prometheus-research)

`prometheus-research` is a Rust binary (v1.6.0) that runs the deep-research
pipeline as a persistent background server with real-time progress streaming.
It ships with `prometheus-skill-pack` and is installed by
`scripts/install-binaries.sh`.

### Starting the server

```bash
# Start HTTP server on :7891 (launchd auto-starts this on macOS)
prometheus-research --mode server

# Check it is up
curl -s http://127.0.0.1:7891/health | jq .
# {"status":"ok","version":"0.1.0","pid":12345}
```

The launchd service `com.prometheus.research` starts automatically when the
skill pack is installed. Restart manually with:

```bash
launchctl kickstart -k gui/$(id -u)/com.prometheus.research
```

### MCP tools (5)

Use these from within a research session once the server is running:

| Tool | Description |
|------|-------------|
| `research_start` | Start a new research job; returns `job_id` |
| `research_status` | Poll job status and stage progress |
| `research_cancel` | Cancel a running job |
| `research_export` | Export the finished `.research` package |
| `render_component` | Render an A2UI component via surface-bridge |

**Example flow:**

```
research_start(query="State of Rust async runtimes", depth="deep")
  → {"job_id": "job_abc123", "status": "started"}

research_status(job_id="job_abc123")
  → {"job_id": "...", "stage": 3, "stage_name": "Retrieve", "progress": 40, "status": "running"}

research_export(job_id="job_abc123")
  → {"path": "~/.prometheus/research/job_abc123/"}
```

### AG-UI SSE stream

The server emits structured events on a per-job SSE stream. Connect from any
`EventSource`-capable client:

```
GET http://127.0.0.1:7891/api/v1/jobs/{job_id}/events
```

**Event types:**

| `type` field     | When emitted | Key fields |
|------------------|-------------|------------|
| `agent.status`   | Each stage start/progress | `stage`, `stage_name`, `progress`, `status`, `tokens` |
| `agent.message`  | Log messages from stages | `message`, `level` |
| `agent.error`    | Stage failure | `message`, `stage` |
| `a2ui.component` | UI component ready to render | `component`, `props` |

All events include `job_id` and `timestamp`.

**Minimal EventSource listener (browser / HTMX):**

```js
const es = new EventSource('http://127.0.0.1:7891/api/v1/jobs/job_abc123/events');
es.onmessage = (e) => console.log(JSON.parse(e.data));
```

### A2UI component endpoints

Eight pre-built HTMX fragments are served at `/components/{name}`:

| Component | Purpose |
|-----------|---------|
| `progress-bar` | Stage progress ring |
| `source-card` | Individual source with credibility score |
| `citation-list` | Formatted citation list |
| `graph-view` | Knowledge graph minimap |
| `contradiction-panel` | Contradiction log with resolution status |
| `stage-timeline` | 10-stage execution timeline |
| `confidence-meter` | Overall confidence score gauge |
| `export-card` | Download / copy `.research` package |

Each endpoint accepts `?job_id=<id>` and returns a self-contained HTMX fragment
for `hx-swap-oob` injection. See
[`skills/research/deep-research/references/a2ui-components.md`](references/a2ui-components.md)
for full prop schemas.

## 10-Stage Pipeline

Stages execute sequentially. Each stage's output is the next stage's input.

| Stage | Name | Sub-skill | Model | Key integration |
|-------|------|-----------|-------|-----------------|
| 01 | Planner | [stage-01-planner](skills/stage-01-planner/SKILL.md) | frontier | sequential_thinking |
| 02 | Search | [stage-02-search](skills/stage-02-search/SKILL.md) | medium | firecrawl_search, tavily_search |
| 03 | Retrieve | [stage-03-retrieve](skills/stage-03-retrieve/SKILL.md) | medium | firecrawl_scrape |
| 04 | Collect | [stage-04-collect](skills/stage-04-collect/SKILL.md) | medium | surreal-memory create_entity |
| 05 | Verify | [stage-05-verify](skills/stage-05-verify/SKILL.md) | frontier | sycophancy-correction |
| 06 | Resolve | [stage-06-resolve](skills/stage-06-resolve/SKILL.md) | frontier | pmpo-elicit |
| 07 | Graph | [stage-07-graph](skills/stage-07-graph/SKILL.md) | frontier | surreal-memory create_relation |
| 08 | Cite | [stage-08-cite](skills/stage-08-cite/SKILL.md) | small | surreal-memory add_memory |
| 09 | Report | [stage-09-report](skills/stage-09-report/SKILL.md) | frontier | learn-grade |
| 10 | Export | [stage-10-export](skills/stage-10-export/SKILL.md) | small | OKF v0.1 |

**Pipeline flow:**

```
QUERY
  │
  ▼ Stage 01 — Planner
  Sub-questions + search strategy + token budget
  │
  ▼ Stage 02 — Search
  Source URLs (deduplicated, domain-filtered)
  │
  ▼ Stage 03 — Retrieve
  Content chunks (≤2K tokens each)
  │
  ▼ Stage 04 — Collect
  Indexed sources → surreal-memory entities
  │
  ▼ Stage 05 — Verify
  Credibility scores (0–100) per source
  │
  ▼ Stage 06 — Resolve
  Contradictions detected + resolved (or escalated)
  │
  ▼ Stage 07 — Graph
  Knowledge graph: entities + relations in surreal-memory
  │
  ▼ Stage 08 — Cite
  Citation list with confidence scores
  │
  ▼ Stage 09 — Report
  Synthesized OKF report (Feynman quality gate)
  │
  ▼ Stage 10 — Export
  .research package on disk
```

## .research Package Format

Output packages follow OKF v0.1 with Prometheus research extensions.

**Directory layout:**
```
<job_id>/
  manifest.json        # OKF metadata + research extensions
  index.md             # Human-readable entry point
  sources/             # Raw collected sources (one JSON per source)
  graph.json           # Knowledge graph export
  citations.json       # Citation list with confidence scores
  contradictions.json  # Contradiction log (resolved + unresolved)
  report.md            # Final synthesis (OKF frontmatter)
```

**`report.md` frontmatter (OKF v0.1 + extensions):**
```yaml
---
type: research-report
title: <query title>
date: <ISO 8601>
confidence: <0.0–1.0>
verification_status: verified | partial | unverified
sources_count: <n>
feynman_grade: <0.0–1.0 or null>
contradictions_resolved: <n>
okf_version: '0.1'
---
```

For full format spec, see [references/research-package-spec.md](references/research-package-spec.md).

## Integration Guide

### surreal-memory

Stage 04 indexes each source as a `ResearchSource` entity:
```
create_entity(name=<url>, entityType="ResearchSource",
  observations=["credibility:<score>", "date:<date>", "excerpt:<text>"])
```

Stage 07 builds relations:
```
create_relation(from=<claim>, to=<source>, relationType="cites")
create_relation(from=<claimA>, to=<claimB>, relationType="contradicts")
```

### liter-llm-bridge

Model routing is automatic via the `model_routing` frontmatter block. Set
`LITER_LLM_BRIDGE_ENABLED=1` to activate. When disabled, all stages use the
session default model. See [references/model-routing.md](references/model-routing.md).

### Feynman Quality Gate (Stage 09)

The report draft passes through `learn-grade` before delivery:
- Required: overall score ≥ 0.7 AND `misconceptions_absent == 1.0`
- Failure: report is rejected and Stage 09 re-synthesizes with the grade feedback
- Unavailable: gate is skipped, `feynman_grade: null` in manifest

See [references/feynman-quality-gate.md](references/feynman-quality-gate.md).

### sycophancy-correction (Stage 05)

Source framing is checked via `detect_sycophancy` during verification. Sources
that frame claims with excessive confidence or suppress contradictory evidence
receive a credibility penalty. See [references/sycophancy-correction-integration.md](references/sycophancy-correction-integration.md).

## Examples

### Example 1: Technology landscape (deep)

```
/deep-research "Current state of vector databases for production RAG systems"
```

Pipeline: Planner decomposes into 6 sub-questions → Search finds 45 sources →
Retrieve chunks all pages → Collect indexes into surreal-memory → Verify filters
to 31 credible sources → Resolve 3 contradictions (version conflicts) → Graph
builds entity map of 12 databases → Cite generates APA citations → Report
synthesizes 2,400-word analysis → Export creates `vector-db-rag-2026/` package.

### Example 2: Competitive analysis (shallow)

```
/deep-research --depth shallow "How does Temporal compare to Conductor and Apache Airflow?"
```

Pipeline runs Stages 01-05 only. Produces a quick credibility-scored source list
and brief comparison without knowledge graph or formal report.

### Example 3: Regulatory research (exhaustive + KB)

```
/deep-research --depth exhaustive --kb dify:legal-kb "EU AI Act obligations for high-risk AI systems"
```

KB grounds Stage 01 planning with internal legal documents. Web search augments
with current regulatory guidance. Contradiction resolution escalates to pmpo-elicit
for legal ambiguities.

## Common Issues

**"No search results returned"**
→ Check `TAVILY_API_KEY` or `FIRECRAWL_API_KEY` is set. At least one must be
  present. Run `bash scripts/run-research.sh --check-tools`.

**"Stage 06 stalled on contradiction"**
→ Set `RESEARCH_AUTO_ESCALATE=1` to automatically invoke pmpo-elicit for
  unresolvable contradictions, or add `--no-escalate` to skip resolution and
  mark contradictions as unresolved in the package.

**"surreal-memory not available"**
→ Stages 04 and 07 degrade gracefully — sources are tracked in-memory and
  graph.json is still written to disk. surreal-memory enables cross-session
  querying; it is not required for a single research run.

**"Feynman gate rejected the report"**
→ The grade feedback is shown inline. The most common reason is missing evidence
  for key claims (`misconceptions_absent < 1.0`). Add `--skip-feynman` to
  bypass the gate (not recommended for production research).

**"Package already exists at output path"**
→ Research outputs to `~/.prometheus/research/<job_id>/` by default. Set
  `OUTPUT_DIR` to a custom path, or use `--overwrite` to replace.
