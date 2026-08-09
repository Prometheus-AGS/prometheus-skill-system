# OKF Research Format

The `.research` package uses **Open Knowledge Format (OKF) v0.1** as its base,
extended with Prometheus research-specific fields.

OKF v0.1 base spec is vendored at `shared/references/okf-v0.1.md`.
OKF requires only a non-empty `type` frontmatter key. Unknown fields are
permitted and must not cause rejection (permissive consumption rule).

## Base OKF Fields (required)

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Must be `research-report` |

## Base OKF Fields (optional, recommended)

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Human-readable report title |
| `date` | string | ISO 8601 date of creation |
| `links` | string[] | Related OKF documents |
| `tags` | string[] | Searchable keywords |

## Prometheus Research Extensions

All extension fields are prefixed with their domain. Unknown consumers
(non-Prometheus OKF readers) will ignore them per OKF permissive consumption.

| Field | Type | Description |
|-------|------|-------------|
| `confidence` | float | Weighted average claim confidence (0.0–1.0) |
| `verification_status` | enum | `verified` / `partial` / `unverified` |
| `sources_count` | int | Number of sources contributing to findings |
| `feynman_grade` | float | learn-grade quality score (null if gate skipped) |
| `contradictions_resolved` | int | Count of contradictions resolved in Stage 06 |
| `okf_version` | string | OKF spec version used (`'0.1'`) |
| `job_id` | string | Research job identifier for cross-referencing |
| `query` | string | Original research query |
| `depth` | enum | `shallow` / `deep` / `exhaustive` |

## Full Frontmatter Example

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
job_id: vdb-rag-2026-001
query: "Current state of vector databases for production RAG systems"
depth: deep
tags: [vector-databases, rag, production, benchmarks]
links: []
---
```

## Verification Status Rules

| Status | Condition |
|--------|-----------|
| `verified` | Stage 05 ran AND all sources scored ≥ 40 AND feynman_grade ≥ 0.7 |
| `partial` | Stage 05 ran BUT some sources below threshold OR feynman gate skipped |
| `unverified` | Stage 05 did not run (shallow depth) |

## Package File Requirements

Every `.research` package must contain `manifest.json` and `report.md`.
All other files (`graph.json`, `citations.json`, `contradictions.json`, `sources/`)
are optional but created by default for `deep` and `exhaustive` depth runs.
