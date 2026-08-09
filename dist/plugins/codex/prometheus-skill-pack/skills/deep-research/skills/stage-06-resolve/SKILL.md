---
name: stage-06-resolve
description: >
  Deep research Stage 06 — Resolve. Detects contradictions between source claims
  and resolves them using structured strategies (authority, recency, consensus).
  Escalates to pmpo-elicit when contradictions cannot be resolved autonomously.
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-06, resolve, contradiction-detection, pmpo-elicit]
---

# Stage 06 — Resolve

## Purpose

Detect claims from different sources that directly contradict each other.
Apply resolution strategies to produce a single reconciled position per
contradicted topic. Escalate to human judgment via pmpo-elicit when contradictions
are genuinely ambiguous. Log all resolutions for the final report.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `source_registry` | object[] | Indexed sources from Stage 04 |
| `credibility_scores` | object | `{url: score}` from Stage 05 |
| `auto_escalate` | bool | Whether to call pmpo-elicit automatically (default: false) |

## Output

| Field | Type | Description |
|-------|------|-------------|
| `resolved_claims` | object[] | `{topic, position, confidence, strategy, sources_used}` |
| `unresolved_claims` | object[] | Claims escalated to human or marked pending |
| `contradictions_log` | object[] | Full contradiction pairs with resolution detail |

## Instructions

1. **Detect contradictions** — run `scripts/detect-contradictions.sh` against
   the source registry. This produces contradiction pairs grouped by topic.

2. **Apply resolution strategy** (in order of precedence):
   - **Source authority** — if credibility scores differ by ≥20 points, take
     the higher-scoring source's position
   - **Recency** — if publication dates differ by >12 months, prefer the newer
   - **Consensus** — if 3+ sources agree against 1, take the majority position
   - **Escalate** — if none of the above resolves it, escalate

3. **Escalate to pmpo-elicit** (when `auto_escalate=true` or resolution fails):
   ```bash
   bash "${CLAUDE_PLUGIN_ROOT}/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh" \
     "<job_id>/elicitations/resolve-$(date +%s)" \
     "Contradiction on <topic>: Source A says X, Source B says Y. Which is correct?" \
     "high" "deep-research-stage-06"
   ```
   On Claude Code platforms, use `AskUserQuestion` with the two positions as options.

4. **Record all decisions** — write to `contradictions.json` with strategy used,
   confidence in the resolution, and sources cited.

5. **Emit resolved claim set** — JSON for Stages 07–09.

## Integration

`scripts/detect-contradictions.sh` for automated contradiction detection
`pmpo-elicit` skill for human escalation
`templates/contradiction-resolution.md` for output format

## Example

**Contradiction detected:**
```
Topic: "Qdrant single-node throughput"
Source A (qdrant.tech, score=77): "500K queries/sec"
Source B (arxiv.org, score=85): "120K queries/sec at comparable hardware"
```

**Resolution:** Source authority (score gap = 8, < 20) + recency check → Source A is 2025, Source B is 2023 → **Recency resolution: take Source A position**. Confidence: 0.72.
