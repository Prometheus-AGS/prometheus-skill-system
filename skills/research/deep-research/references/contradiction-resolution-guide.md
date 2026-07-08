# Contradiction Resolution Guide

Stage 06 applies these strategies in priority order. The first strategy
that produces a resolution with confidence ≥ 0.60 is used.

## Strategy 1: Source Authority

**Condition:** Credibility score gap between conflicting sources is ≥ 20 points.

**Resolution:** Take the position from the higher-credibility source.

**Confidence:** `(score_gap / 100) * 0.9` — a 30-point gap yields ~0.27 base,
scaled to max 0.90. Minimum confidence for this strategy: 0.60.

**Example:**
- Source A (score=85): "Qdrant achieves 500K QPS"
- Source B (score=40): "Qdrant achieves 120K QPS"
- Gap = 45 points → take Source A position. Confidence: 0.81.

## Strategy 2: Recency

**Condition:** Publication dates differ by > 12 months.

**Resolution:** Take the position from the more recently published source.
Rationale: for rapidly evolving technical topics, newer data supersedes older.

**Confidence:** Inversely proportional to remaining uncertainty. Base: 0.70.
Reduce by 0.10 if the newer source has lower credibility than the older.

**Example:**
- Source A (2025-Q1): "Qdrant 500K QPS"
- Source B (2023-Q2): "Qdrant 120K QPS"
- Date gap = 24 months → take Source A position. Confidence: 0.70.

## Strategy 3: Consensus

**Condition:** 3 or more sources agree on a position against 1 dissenting source.

**Resolution:** Take the majority position.

**Confidence:** `0.65 + (0.05 * agreement_count)`. Three agreeing = 0.80.

**Example:**
- Sources A, B, C: "Qdrant supports multi-tenancy natively"
- Source D: "Qdrant requires external sharding for multi-tenancy"
- 3 vs 1 → consensus position. Confidence: 0.80.

## Strategy 4: Escalation

**Condition:** None of the above strategies resolve the contradiction, OR
the contradiction is a legal/regulatory/safety claim where autonomous
resolution would be irresponsible.

**Resolution:** Escalate to human judgment via `pmpo-elicit`.

**On Claude Code platforms:**
```
AskUserQuestion(
  question = "Contradiction on '<topic>': Source A says '<positionA>', Source B says '<positionB>'. Which position should the research report take?",
  options = [
    { label: "Source A position", description: "<positionA> (credibility: <scoreA>)" },
    { label: "Source B position", description: "<positionB> (credibility: <scoreB>)" },
    { label: "Mark as unresolved", description: "Report both positions without resolution" }
  ]
)
```

**On other platforms:**
```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh" \
  "<job_id>/elicitations/resolve-<timestamp>" \
  "Contradiction on '<topic>': Source A says '<positionA>', Source B says '<positionB>'. Which?" \
  "high" "deep-research-stage-06" \
  "<positionA>" "<positionB>"
```

When `RESEARCH_AUTO_ESCALATE=0` (default) and no user is present: mark
as unresolved and include both positions in `contradictions.json`.

## Unresolved Contradictions

If no strategy resolves a contradiction:
- Record in `contradictions.json` with `resolved: false`
- Include both positions in the research report under "Contradictions" section
- Do not suppress either position — report the disagreement transparently
- Reduce the topic's confidence contribution by 0.20
