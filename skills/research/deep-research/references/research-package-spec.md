# Research Package Specification

**This document is normative.** It is the single prose contract for the
research package that the deep-research pipeline produces. Its machine
counterpart is [`schemas/research-manifest.schema.json`](schemas/research-manifest.schema.json),
which is authoritative for `manifest.json` field names, types, and enums.
`scripts/export-package.sh` writes the manifest, `scripts/check-research-package.sh`
validates it in shell, and `prometheus-research` validates it in Rust before
`research_export` returns. Any other description of the package in this skill
(the SKILL.md example, stage skills, agent prompts) is a copy of this document
and is checked against it; when they disagree, this document and the schema win.

Contract version: `format_version` 2.0.0 (2026-09-05, change-rah-002). Manifests
with `format_version` 1.x predate the schema and do not validate.

## Location and naming

Packages live under the research output root:

```
${RESEARCH_OUTPUT_DIR:-~/.prometheus/research}/<package_id>/
```

`<package_id>` is `<slug>-<yyyymmdd>-<4hex>`, where `<slug>` is at most five
lowercase hyphenated words derived from the query by
`shared/scripts/lib/slug.sh` (the same rule `learn-goal` uses for goal ids).
Generic names such as `research`, `report`, or `summary` are never a slug on
their own; concurrent runs cannot collide because the date and random suffix are
part of the id. The daemon's own job id (`job-<epoch>-<uuid8>`) is recorded in
the manifest as `job_id` so checkpoints and packages can be joined; it is not
the directory name.

There is no `.research` suffix. The directory is the package.

The legacy root `~/.research-jobs/` is not read by anything after change-rah-002.
The daemon's `status` output names it when it still exists so an operator can
delete it by hand; nothing is migrated.

## Directory structure

```
<package_id>/
├── manifest.json          # Package metadata and provenance (schema-validated)
├── index.md               # Human-readable entry point
├── report.md              # Final synthesis (OKF frontmatter)
├── <slug>.provenance.md   # Provenance sidecar, written on every exit path
├── plan.md                # Stage 01 plan; task ledger, verification log, decision log
├── checkpoint.json        # Driver checkpoint (last completed stage, timestamps)
├── sources/
│   ├── url-list.json      # Stage 02 output
│   ├── chunk-<n>.json     # Stage 03 output
│   ├── registry.json      # Stage 04 output
│   └── credibility.json   # Stage 05 output
├── graph.json             # Knowledge graph (topics, claims, relations)
├── citations.json         # Formatted citations
├── contradictions.json    # Contradiction log (resolved + unresolved)
└── sensitivity.json       # Rank sensitivity written by stage 05 (optional: absent when stage 05 was skipped)
```

`plan.md` is Markdown, not JSON. Stage 01 writes it and the driver appends to its
three maintained sections at every stage boundary. No consumer reads `plan.json`.

## manifest.json

Every field is required; nullable fields carry `null` explicitly. The schema is
the authority; this example is a literal, schema-valid copy for readers:

```json
{
  "format": "research-package",
  "format_version": "2.0.0",
  "okf_type": "research-session",
  "package_id": "vector-db-rag-20260905-a1f3",
  "job_id": "job-1788000774-ffa477f9",
  "query": "Current state of vector databases for production RAG systems",
  "depth": "deep",
  "scale": "full",
  "created_at": "2026-09-05T10:00:00Z",
  "completed_at": "2026-09-05T11:02:14Z",
  "stages_completed": ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"],
  "sources_count": 31,
  "claims_count": 87,
  "confidence": 0.74,
  "verification_status": "verified",
  "verification_verdict": "PASS WITH NOTES",
  "feynman_grade": 0.82,
  "feynman_gate_used": true,
  "misconceptions_absent": 1.0,
  "contradictions_detected": 4,
  "contradictions_resolved": 3,
  "contradictions_unresolved": 1,
  "surreal_memory_used": true,
  "sycophancy_correction_used": true,
  "adversarial_review_used": true,
  "citation_style": "APA",
  "kb_ids": [],
  "model_routing": {
    "01": "frontier", "02": "medium", "03": "medium", "04": "medium",
    "05": "frontier", "06": "frontier", "07": "frontier", "08": "small",
    "09": "frontier", "10": "small"
  },
  "files": {
    "report": "report.md",
    "provenance": "vector-db-rag.provenance.md",
    "plan": "plan.md",
    "graph": "graph.json",
    "citations": "citations.json",
    "contradictions": "contradictions.json",
    "index": "index.md",
    "sources_dir": "sources/"
  }
}
```

Field notes that the schema cannot express:

- `stages_completed` lists stages whose required artifacts validated, in
  execution order. `--resume` reads it. A `direct` scale run lists exactly
  `01, 02, 03, 05, 09, 10`.
- `verification_status` is derived from claim labels by the rule in
  [`okf-research-format.md`](okf-research-format.md), never set by hand.
- `verification_verdict` duplicates the sidecar's verdict so the two files
  cannot disagree; the drift check compares them.
- `model_routing` records the class liter-llm-bridge resolved per stage, or
  `session-default` when the bridge was disabled.

## report.md OKF frontmatter

```yaml
---
type: research-report
title: "..."
query: "..."
date: "2026-09-05"
confidence: 0.74
verification_status: verified
feynman_grade: 0.82
sources_count: 31
contradictions_resolved: 3
package_id: "vector-db-rag-20260905-a1f3"
job_id: "job-1788000774-ffa477f9"
tags: [deep-research, <topic-tag>]
links: []
---
```

`verification_status`, `confidence`, `feynman_grade`, `sources_count`, and
`contradictions_resolved` must equal the manifest values; the drift check
compares them.

## <slug>.provenance.md

Written by `scripts/write-provenance.sh` on every exit path of the driver,
including failure and interruption (change-rah-003).

```markdown
# Provenance: <query>

- **Package:** <package_id>
- **Date:** <created_at> to <completed_at or "not completed">
- **Scale:** direct | full
- **Stages completed:** 01 02 03 05 09 10
- **Sources consulted:** <n>
- **Sources accepted:** <n>
- **Sources rejected:** <n> (<reasons>)
- **Verification:** PASS | PASS WITH NOTES | BLOCKED
- **Blocked:** <stage and reason, or none>
- **Adversarial review:** <verdict and counts, or "blocked: judge unavailable">
- **Plan:** plan.md
- **Stage files:** <list>
```

## Claim labels

Every claim, citation, contradiction entry, and evidence-table row carries a
label from `verified | unverified | blocked | inferred`. The meanings, the
evidence each label requires, the `critical` flag, and the rule that derives
the package-level `verification_status` from them are normative in
[`okf-research-format.md`](okf-research-format.md) ("Claim Labels" and
"Verification Status Rules"). `check-research-package.sh --package` validates
`graph.json` against [`schemas/research-graph.schema.json`](schemas/research-graph.schema.json),
recomputes `verification_status`, and fails when the declared value differs.

## graph.json

Schema: [`schemas/research-graph.schema.json`](schemas/research-graph.schema.json).

```json
{
  "topics": [
    { "id": "topic-001", "name": "...", "claims": ["claim-3f2a9c1e0b7d4a55"] }
  ],
  "claims": [
    {
      "id": "claim-3f2a9c1e0b7d4a55",
      "text": "...",
      "label": "verified",
      "critical": true,
      "confidence": 0.82,
      "sources": ["source-hash-001"],
      "contradicts": [],
      "evidence": "source-hash-001 §3: \"...the exact passage that supports the statement...\""
    }
  ],
  "relations": [
    { "from": "claim-3f2a9c1e0b7d4a55", "to": "source-hash-001", "type": "cites" },
    { "from": "claim-...", "to": "claim-...", "type": "contradicts" }
  ]
}
```

Claim ids are content-addressed: `claim-` plus the first 16 hex characters of
`sha256("<package_id>:<normalised text>")`, where the text is lower-cased,
whitespace-collapsed, and stripped of trailing punctuation. `scripts/build-graph.sh`
and `scripts/detect-contradictions.sh` share the function, so a contradiction
entry names the ids the graph will carry. The same sentence appearing in two
artifacts is one claim: sources are the union and the label is the higher of
the copies (`verified` > `inferred` > `unverified` > `blocked`). The older
`claim-NNN` form still validates for packages written before change-rah-010.
`critical` marks claims that appear in the executive summary or as a Key
Finding.

## sensitivity.json

Written by stage 05 (`scripts/score-sources.py`) at the package root, alongside `manifest.json`,
and listed in the manifest as `files.sensitivity` when present. It answers
one question: **how much does the source order depend on the weighting?**

```json
{
  "generated_at": "2026-09-06T10:04:12Z",
  "default_weights": { "domain_authority": 0.25, "author_expertise": 0.20, "citation_depth": 0.20, "publication_recency": 0.20, "factual_verifiability": 0.15 },
  "profiles": [ { "id": "balanced", "label": "Balanced", "description": "...", "weights": { "...": 0.2 } } ],
  "sources": [
    {
      "url": "https://...",
      "base_rank": 1, "base_score": 88,
      "rank_range": 0, "score_range": 6,
      "stability": "stable",
      "profile_ranks": [ { "profile_id": "balanced", "rank": 1, "score": 86, "applied_weights": { "...": 0.25 } } ],
      "drivers": ["strong on domain_authority, publication_recency"]
    }
  ],
  "summary": { "stable": 3, "sensitive": 1, "volatile": 0, "top_source": "https://...", "top_source_stable": true },
  "basis": ["..."]
}
```

Every source is scored on the five rubric dimensions **only where the
registry carries evidence**; a missing dimension is excluded from that
source's weight denominator and `credibility.json` records the renormalised
`applied_weights` per source. The four alternate profiles (`balanced`,
`authority_heavy`, `recency_heavy`, `methodology_heavy`) rerun the same
signals; `stability` is `stable` when the rank never moves, `sensitive` when
it moves by at most two places, `volatile` otherwise. Sycophancy penalties are
preserved across profiles. The report and the provenance sidecar may cite a
`volatile` top source as a reason the ranking is not decisive.

## citations.json

```json
{
  "style": "APA",
  "citations": [
    {
      "id": "cite-001",
      "url": "https://...",
      "formatted": "Author, A. (2025). Title. Publisher. https://...",
      "credibility_score": 77,
      "confidence": 0.82,
      "label": "verified"
    }
  ]
}
```

## contradictions.json

```json
{
  "contradictions": [
    {
      "id": "contra-001",
      "topic": "...",
      "claim_a": { "id": "claim-...", "text": "...", "source": "...", "credibility": 85 },
      "claim_b": { "id": "claim-...", "text": "...", "source": "...", "credibility": 40 },
      "strategy_tried": "source_authority",
      "resolved": true,
      "resolution": "claim_a",
      "confidence": 0.81,
      "label": "inferred",
      "audit_trail": "Score gap 45 points; took the higher-credibility position"
    }
  ]
}
```

An entry with `resolved: false` after stage 06 fires `hooks/on-contradiction.sh`.

`claim_a.credibility` and `claim_b.credibility` are the source's stage 05 score, or `null` when the source was never scored (a claim that reached the detector from the registry alone). Entries written by `scripts/detect-contradictions.sh` carry `strategy_tried` `numeric` or `semantic`; a semantic pair that could not be judged (no gateway, or an unparseable reply) is `label: blocked`, `resolved: false`, with the reason in `audit_trail`, and the file's `detection.semantic.status` says `blocked`.

## OKF extensions

The Prometheus research package extends OKF v0.1 with these fields:

| Field | OKF status | Description |
|---|---|---|
| `confidence` | extension | Credibility-weighted mean of claim confidences |
| `verification_status` | extension | `verified`, `partial`, or `unverified`, derived from claim labels |
| `feynman_grade` | extension | learn-grade overall_score (0.0 to 1.0) |
| `sources_count` | extension | Sources indexed |
| `contradictions_resolved` | extension | Contradictions auto-resolved |
| `package_id`, `job_id` | extension | Package directory name and run identifier |

## Validation

```bash
# Shell: validates the SKILL.md example and a fresh export against the schema
bash skills/research/deep-research/scripts/check-research-package.sh

# Shell: validate one package directory
bash skills/research/deep-research/scripts/check-research-package.sh --package ~/.prometheus/research/<package_id>
```

The shell check uses python3 `jsonschema` when it is importable. When it is not,
it compares the manifest's full key set and each declared type against the
schema's `properties` and reports `PASS WITH NOTES: jsonschema unavailable,
structural check only`. Either path exits non-zero on any mismatch.

## Palace ingestion

After export, the package can be ingested into the palace for later retrieval:

```bash
bash skills/research/deep-research/scripts/export-package.sh <package_id> --ingest-palace
```

`export-package.sh` cannot call the `palace_ingest` MCP tool from bash; with
`--ingest-palace` it writes `.palace-ingest-requested` in the package and
`hooks/post-export.sh` records the request, so the harness session that owns the
run performs the ingestion and removes the marker.
