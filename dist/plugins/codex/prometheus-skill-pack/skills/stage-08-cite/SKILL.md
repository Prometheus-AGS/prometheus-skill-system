---
name: stage-08-cite
description: >
  Deep research Stage 08 — Cite. Generates formatted citations for all verified
  sources used in resolved claims. Assigns per-citation confidence scores.
  Supports APA 7, MLA 9, Chicago 17, IEEE, and Vancouver styles.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-08, citations, bibliography, confidence-scores]
---

# Stage 08 — Cite

## Purpose

Generate a complete bibliography for all sources that contributed to resolved
claims. Assign per-citation confidence scores derived from credibility ratings.
Store citations in surreal-memory and write `citations.json` to the package.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `verified_sources` | object[] | Credibility-scored sources from Stage 05 |
| `resolved_claims` | object[] | Claim-to-source linkage from Stage 06 |
| `entity_ids` | object | surreal-memory entity IDs from Stage 07 |
| `citation_style` | enum | APA/MLA/Chicago/IEEE/Vancouver (default: APA) |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `citations` | object[] | `{id, formatted, url, credibility_score, used_in_claims[]}` |
| `citation_count` | int | Total citations |
| `citations_json_path` | string | Path to `citations.json` in the package |

## Instructions

1. **Extract metadata** — for each source, extract: title, authors, publication
   date, publisher/domain, URL, access date. Use metadata from Stage 03's
   retrieved content when available; fall back to URL-derived metadata.

2. **Format citations** — apply the requested style (default APA 7). Use
   `references/citation-formats.md` as the formatting reference.
   Override via `RESEARCH_CITATION_STYLE` environment variable.

3. **Assign confidence scores** — map credibility score → citation confidence:
   - 80–100 → high (0.85–1.0)
   - 60–79 → medium (0.65–0.84)
   - 40–59 → low (0.40–0.64)

4. **Store in surreal-memory** — `add_memory(content=<citation>, user_id=<job_id>)`
   for cross-session citation retrieval.

5. **Write `citations.json`** — emit the full citation list with confidence scores.

## Integration

`surreal-memory` MCP: `add_memory` for persistent citation storage
`references/citation-formats.md` for style formatting rules
Environment: `RESEARCH_CITATION_STYLE` (APA|MLA|Chicago|IEEE|Vancouver)

## Example

**Source:** `https://qdrant.tech/benchmarks/` (credibility: 77)

**APA 7 output:**
```
Qdrant Team. (2025, March). ANN benchmarks: Qdrant v1.8 performance results.
Qdrant. https://qdrant.tech/benchmarks/
```

**Citation object:**
```json
{
  "id": "cite-001",
  "formatted": "Qdrant Team. (2025, March)...",
  "url": "https://qdrant.tech/benchmarks/",
  "credibility_score": 77,
  "confidence": 0.82,
  "used_in_claims": ["claim-001", "claim-003"]
}
```
