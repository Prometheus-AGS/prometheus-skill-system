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

1. **Detect contradictions** — run the detector against the package:

   ```bash
   bash scripts/detect-contradictions.sh <package_dir> --semantic --out <package_dir>/contradictions.json
   ```

   It writes `contradictions.json` in the spec shape with two detection paths:
   - **numeric** — the same measurable topic (percentage, latency, throughput,
     storage) stated with values more than 2× apart across sources. Mechanical,
     so each entry is `label: inferred`, `resolved: false`, `strategy_tried:
     numeric`, for step 2 to resolve.
   - **semantic** (`--semantic`) — candidate pairs from different sources that
     share content words are put to the critic model through `kbd_complete`
     (`shared/scripts/lib/kbd-model-resolve.sh`, critic role). A pair the judge
     calls contradictory is recorded `label: inferred` with the judge's reason
     and confidence. When no gateway is reachable every candidate pair is
     recorded `label: blocked`, `resolved: false`, with the reason in
     `audit_trail`, and `detection.semantic.status` is `blocked`: an unchecked
     contradiction stays visible and fires `on-contradiction.sh` rather than
     disappearing. It is never described as checked.

   Claim ids in the entries are the content-addressed ids Stage 07 will assign
   (same hash function as `build-graph.sh`), so a resolved pair becomes a
   `contradicts` relation without re-matching text.

2. **Apply resolution strategy** (in order of precedence):
   - **Source authority** — if credibility scores differ by ≥20 points, take
     the higher-scoring source's position
   - **Recency** — if publication dates differ by >12 months, prefer the newer
   - **Consensus** — if 3+ sources agree against 1, take the majority position
   - **Escalate** — if none of the above resolves it, escalate

3. **Escalate to pmpo-elicit** (when `auto_escalate=true` or resolution fails):
   ```bash
   bash "${CLAUDE_PLUGIN_ROOT}/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh" \
     "<package_id>/elicitations/resolve-$(date +%s)" \
     "Contradiction on <topic>: Source A says X, Source B says Y. Which is correct?" \
     "high" "deep-research-stage-06"
   ```
   On Claude Code platforms, use `AskUserQuestion` with the two positions as options.

4. **Record all decisions** — write to `contradictions.json` with strategy used,
   confidence in the resolution, sources cited, and a `label` per entry
   (`references/okf-research-format.md`, "Claim Labels"): `verified` when the
   chosen position's claim is itself `verified` in `credibility.json`;
   `inferred` when the position was chosen by authority, recency, or consensus
   reasoning rather than stated by a verified source; `blocked` when the
   contradiction is escalated or left unresolved. A resolution never upgrades a
   claim's label; a `verified` label comes only from Stage 05.

5. **Emit resolved claim set** — JSON for Stages 07–09.

## Integration

`scripts/detect-contradictions.sh` for automated contradiction detection (numeric, and semantic via `kbd_complete`)
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
