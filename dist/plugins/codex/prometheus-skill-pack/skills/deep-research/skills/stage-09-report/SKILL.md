---
name: stage-09-report
description: >
  Deep research Stage 09 — Report. Synthesizes all resolved claims, citations,
  and graph data into an OKF-compliant research report. Passes the draft through
  the Feynman quality gate (learn-grade) before delivery. Confidence score is
  computed from source credibility and claim resolution confidence.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-09, report, synthesis, feynman-gate, learn-grade, okf]
---

# Stage 09 — Report

## Purpose

Synthesize all stage outputs into a cohesive, structured research report that
meets OKF v0.1 format requirements and Prometheus research extension schema.
Apply the Feynman quality gate via `learn-grade` to verify the report accurately
represents the evidence without gaps or misconceptions. Re-synthesize on failure.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `resolved_claims` | object[] | Full resolved claim set from Stage 06 |
| `citations` | object[] | Formatted citations from Stage 08 |
| `graph_json` | object | Knowledge graph from Stage 07 |
| `contradictions_log` | object[] | Contradiction resolution log from Stage 06 |
| `query` | string | Original research query |
| `skip_feynman` | bool | Bypass quality gate (default: false) |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `report_md` | string | Full OKF-compliant report content |
| `confidence` | float | Weighted average claim confidence |
| `feynman_grade` | float | learn-grade score (null if gate skipped) |
| `report_path` | string | Path to `report.md` in the package |

## Instructions

1. **Compute confidence score** — weighted average of all resolved claim
   confidences, weighted by source credibility. This becomes the report-level
   `confidence` field in OKF frontmatter.

2. **Structure the report** — using `templates/report-template.md`:
   - Executive summary (150–300 words)
   - Key findings (one section per resolved claim topic)
   - Evidence table (claims × sources × credibility)
   - Contradictions section (what was disputed and how resolved)
   - Limitations and gaps
   - Conclusion
   - References (formatted citations from Stage 08)

3. **Apply Feynman quality gate** (unless `skip_feynman=true`):
   - Call `learn-grade` with the draft report and a grading rubric derived
     from the sub-questions from Stage 01
   - Require: overall_score ≥ 0.7 AND `misconceptions_absent == 1.0`
   - On failure: incorporate grade feedback and re-synthesize once
   - Second failure: deliver with `feynman_grade` set and a warning

4. **Write OKF frontmatter** — populate all required + Prometheus extension fields.

5. **Write report** — emit `<job_id>/report.md`.

## Integration

`learn-grade` skill: quality gate grading
`templates/report-template.md`: section structure
`references/feynman-quality-gate.md`: gate configuration

## Example

**OKF frontmatter output:**
```yaml
---
type: research-report
title: "Vector Databases for Production RAG: 2025-2026 State"
date: 2026-07-08
confidence: 0.74
verification_status: verified
sources_count: 31
feynman_grade: 0.82
contradictions_resolved: 3
okf_version: '0.1'
---
```
