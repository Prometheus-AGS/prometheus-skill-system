---
name: stage-10-export
description: >
  Deep research Stage 10 — Export. Assembles the complete .research package from
  all stage outputs. Writes manifest.json (OKF + Prometheus extensions), index.md,
  and all data files. Optionally ingests the package into surreal-memory palace.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-10, export, research-package, okf, manifest]
---

# Stage 10 — Export

## Purpose

Assemble the final `.research` package on disk. Write `manifest.json` with
complete OKF metadata and Prometheus research extensions. Create a human-readable
`index.md` entry point. Optionally ingest the package into surreal-memory palace
for semantic search in future sessions.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `job_id` | string | Research job identifier |
| `query` | string | Original research query |
| `report_md` | string | Synthesized report from Stage 09 |
| `graph_json` | object | Knowledge graph from Stage 07 |
| `citations` | object[] | Citation list from Stage 08 |
| `contradictions_log` | object[] | Contradiction log from Stage 06 |
| `confidence` | float | Report confidence score |
| `feynman_grade` | float | Quality gate score (null if skipped) |
| `output_dir` | string | Target directory (default: `~/.prometheus/research/<job_id>/`) |

## Output

Package written to `<output_dir>/`:

| File | Description |
|------|-------------|
| `manifest.json` | OKF metadata + Prometheus extensions |
| `index.md` | Human-readable entry point with links |
| `report.md` | Full research report (from Stage 09) |
| `graph.json` | Knowledge graph export |
| `citations.json` | Formatted citation list |
| `contradictions.json` | Contradiction log |
| `sources/` | Directory of raw source JSON files |

## Instructions

1. **Assemble directory** — create `<output_dir>/` and all subdirectories.
   Copy or write each artifact to its canonical path.

2. **Write `manifest.json`** — populate `templates/research-package-manifest.json`
   schema with all stage outputs. Set `stages_completed` to the list of stages
   that ran successfully.

3. **Write `index.md`** — human-readable entry point with:
   - Query and depth
   - Key stats: sources, confidence, contradictions resolved
   - Links to report, graph, citations
   - Quick access: top 3 findings

4. **Validate package** — verify all expected files exist and `manifest.json`
   parses as valid JSON. Exit non-zero if any file is missing.

5. **Palace ingest** (optional, when `RESEARCH_AUTO_INGEST=1`):
   ```
   palace_ingest(content=<report_md>, metadata={job_id, query, confidence})
   ```
   This enables future `palace_recall(query)` to find this research.

6. **Run `post-export.sh`** — fires the post-export hook for logging and side effects.

## Integration

`scripts/export-package.sh` for assembly automation
`surreal-memory` MCP: `palace_ingest` for optional semantic indexing
`hooks/post-export.sh` for post-assembly side effects
`templates/research-package-manifest.json` for manifest schema

## Example

**`manifest.json` (excerpt):**
```json
{
  "version": "1.0.0",
  "okf_version": "0.1",
  "job_id": "vdb-rag-2026-001",
  "query": "Current state of vector databases for production RAG",
  "depth": "deep",
  "created_at": "2026-07-08T14:30:00Z",
  "stages_completed": ["01","02","03","04","05","06","07","08","09","10"],
  "sources_count": 31,
  "graph_nodes": 47,
  "contradictions_resolved": 3,
  "confidence": 0.74,
  "feynman_grade": 0.82,
  "files": {
    "report": "report.md",
    "graph": "graph.json",
    "citations": "citations.json"
  }
}
```
