---
name: stage-05-verify
description: >
  Deep research Stage 05 — Verify. Scores each source for credibility using a
  5-dimension rubric. Applies sycophancy-correction bias detection to source
  framing. Filters low-credibility sources before graph building.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-05, verify, credibility, sycophancy-correction, source-evaluation]
---

# Stage 05 — Verify

## Purpose

Assign a credibility score (0–100) to each indexed source using a structured
rubric. Apply sycophancy-correction bias detection to identify sources that
over-claim or suppress contradictory evidence. Filter out sources below the
credibility threshold (default: 40). Pass the verified source list to Stage 06.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `source_registry` | object[] | Indexed sources from Stage 04 |
| `credibility_threshold` | int | Minimum score to retain (default: 40) |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `verified_sources` | object[] | Sources with `credibility_score` and `flags[]` |
| `filtered_sources` | object[] | Sources below threshold (logged, not used) |
| `credibility_scores` | object | `{url: score}` map for Stages 06–09 |

## Instructions

1. **Score each source** on 5 dimensions (see `templates/source-evaluation.md`):
   - Domain authority (0–25): `.edu`/`.gov`/established publications = higher
   - Publication recency (0–20): within 12 months = max; >3 years = 0–5
   - Author credentials (0–20): named author with verifiable expertise = max
   - Cross-reference count (0–20): cited by ≥3 other retrieved sources = max
   - Methodology transparency (0–15): shows data, methodology, or primary source

2. **Run sycophancy bias check** — call `detect_sycophancy` on each source's
   extracted claims. Penalize 10–20 points for high/critical severity patterns
   (over-confidence, contradiction suppression). See
   `references/sycophancy-correction-integration.md`.

3. **Apply threshold** — sources scoring < 40 are moved to `filtered_sources`.
   Log the reason. Do not remove them from disk (Stage 06 may reference them for
   contradiction detection).

4. **Update surreal-memory** — add `credibility:<score>` observation to each
   `ResearchSource` entity if surreal-memory is available.

5. **Write output** — emit `<job_id>/sources/credibility.json`.

## Integration

`sycophancy-correction` MCP: `detect_sycophancy(text, strictness="standard")`
`surreal-memory` MCP: `add_observations(entityName=<url>, observations=[...])`
Rubric template: `templates/source-evaluation.md`

## Example

**Source:** `https://qdrant.tech/benchmarks/`
```
Domain authority: 20/25 (vendor docs, credible)
Publication recency: 18/20 (2025-Q1)
Author credentials: 15/20 (named engineers)
Cross-reference count: 16/20 (cited by 4 other sources)
Methodology transparency: 13/15 (shows benchmark methodology)
Sycophancy penalty: -5 (mild over-confidence in claims)
Final score: 77/100 → RETAINED
```
