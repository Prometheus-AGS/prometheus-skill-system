# Research Package Specification

The `.research` package is a portable, self-describing directory format for
exporting completed deep research sessions.

## Directory Structure

```
<job_id>.research/
├── manifest.json          # Package metadata and provenance
├── report.md              # Final research report (OKF frontmatter)
├── sources/
│   ├── registry.json      # All indexed sources with credibility scores
│   └── <source_hash>/     # Per-source extracted content (optional)
│       └── content.txt
├── graph.json             # Knowledge graph (topics, claims, relations)
├── citations.json         # Formatted citations (all styles generated)
├── contradictions.json    # Contradiction log (resolved + unresolved)
└── plan.json              # Original research plan from Stage 01
```

## manifest.json Schema

```json
{
  "format": "research-package",
  "format_version": "1.0.0",
  "okf_type": "research-session",
  "job_id": "<uuid>",
  "query": "original research query",
  "depth": "deep",
  "created_at": "2026-07-08T00:00:00Z",
  "completed_at": "2026-07-08T01:00:00Z",
  "stages_completed": 10,
  "sources_count": 24,
  "claims_count": 87,
  "confidence": 0.83,
  "verification_status": "verified",
  "feynman_grade": 0.86,
  "feynman_gate_used": true,
  "misconceptions_absent": 1.0,
  "contradictions_resolved": 3,
  "contradictions_unresolved": 1,
  "surreal_memory_used": true,
  "sycophancy_correction_used": true,
  "citation_style": "APA",
  "model_routing": {
    "planner": "frontier",
    "search": "medium",
    "verify": "frontier",
    "report": "frontier"
  }
}
```

## report.md OKF Frontmatter

```yaml
---
type: research-report
title: "..."
query: "..."
date: "2026-07-08"
confidence: 0.83
verification_status: verified
feynman_grade: 0.86
sources_count: 24
contradictions_resolved: 3
job_id: "<uuid>"
tags: [deep-research, <topic-tag>]
links: []
---
```

## graph.json Schema

```json
{
  "topics": [
    {
      "id": "topic-001",
      "name": "...",
      "claims": ["claim-001", "claim-002"]
    }
  ],
  "claims": [
    {
      "id": "claim-001",
      "text": "...",
      "confidence": 0.82,
      "sources": ["source-hash-001"],
      "contradicts": []
    }
  ],
  "relations": [
    { "from": "claim-001", "to": "source-hash-001", "type": "cites" },
    { "from": "claim-002", "to": "claim-003", "type": "contradicts" }
  ]
}
```

## citations.json Schema

```json
{
  "style": "APA",
  "citations": [
    {
      "id": "cite-001",
      "url": "https://...",
      "formatted": "Author, A. (2025). Title. Publisher. https://...",
      "credibility_score": 77,
      "confidence": 0.82
    }
  ]
}
```

## contradictions.json Schema

```json
{
  "contradictions": [
    {
      "id": "contra-001",
      "topic": "...",
      "claim_a": { "text": "...", "source": "...", "credibility": 85 },
      "claim_b": { "text": "...", "source": "...", "credibility": 40 },
      "strategy_tried": "source_authority",
      "resolved": true,
      "resolution": "claim_a",
      "confidence": 0.81,
      "audit_trail": "Score gap 45 points → took higher-credibility position"
    }
  ]
}
```

## OKF Extensions

The Prometheus `.research` package extends OKF v0.1 with these custom fields:

| Field | OKF status | Description |
|-------|-----------|-------------|
| `confidence` | extension | Weighted average source confidence |
| `verification_status` | extension | `verified`, `partial`, or `unverified` |
| `feynman_grade` | extension | learn-grade overall_score (0.0–1.0) |
| `sources_count` | extension | Total sources indexed |
| `contradictions_resolved` | extension | Count of auto-resolved contradictions |
| `job_id` | extension | Pipeline run identifier |

## Palace Ingestion

After export, the package can be ingested into the palace for future retrieval:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/research/deep-research/scripts/export-package.sh" \
  "<job_id>" --ingest-palace
```

This calls `palace_ingest` with the report.md content and manifest metadata.
