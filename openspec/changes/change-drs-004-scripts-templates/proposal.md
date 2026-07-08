---
id: change-drs-004-scripts-templates
title: Write 5 scripts + 5 templates for deep-research skill
phase: phase-deep-research-skill
priority: P1
effort: M
wave: 2
agent: general-purpose
status: pending
gap_id: G-04
verdict: BUILD
depends_on: change-drs-001-directory-structure
scope:
  - skills/research/deep-research/scripts/run-research.sh
  - skills/research/deep-research/scripts/export-package.sh
  - skills/research/deep-research/scripts/verify-sources.sh
  - skills/research/deep-research/scripts/build-graph.sh
  - skills/research/deep-research/scripts/detect-contradictions.sh
  - skills/research/deep-research/templates/research-plan.md
  - skills/research/deep-research/templates/source-evaluation.md
  - skills/research/deep-research/templates/contradiction-resolution.md
  - skills/research/deep-research/templates/report-template.md
  - skills/research/deep-research/templates/research-package-manifest.json
---

# change-drs-004 — Scripts + Templates

## Context

Scripts provide automation entry points for the pipeline stages. Templates
provide structured prompts and output formats for the harness to use.

## Scripts (5 files)

All scripts must:
- Begin with `#!/usr/bin/env bash` + `set -euo pipefail`
- Accept arguments via env vars or positional params
- Output JSON on success, human-readable error on failure
- Be marked executable (`chmod +x`)

### `run-research.sh`
Entry point. Accepts `QUERY`, `DEPTH` (shallow/deep/exhaustive), `KB_IDS`.
Prints stage-by-stage progress. Exits 0 on success, 1 on failure.

### `export-package.sh`
Accepts `JOB_ID`, `OUTPUT_DIR`. Creates `.research` package directory with
OKF-compliant `index.md`, `sources/`, `graph.json`, `citations.json`,
`contradictions.json`, `report.md`. Exits 0 on success.

### `verify-sources.sh`
Accepts `SOURCE_URLS` (newline-separated). For each URL, checks domain
reputation heuristics (known quality domains, publication date, author info).
Outputs JSON array of `{url, credibility_score, flags[]}`.

### `build-graph.sh`
Accepts `SOURCES_JSON` (path to collected sources). Extracts entities and
relations. Outputs graph JSON compatible with surreal-memory `create_entity`
and `create_relation` tool inputs.

### `detect-contradictions.sh`
Accepts `SOURCES_JSON`. Compares claims across sources on the same topic.
Outputs JSON array of `{claim_a, source_a, claim_b, source_b, topic, severity}`.

## Templates (5 files)

### `research-plan.md`
Structured markdown template for Stage 1 output:
```markdown
# Research Plan: {{query}}

## Sub-questions
1. {{sub_question_1}}
...

## Stage Sequence
{{stages}}

## Estimated Tokens
{{token_estimate}}

## Confidence Threshold
{{threshold}}
```

### `source-evaluation.md`
Rubric for Stage 5 credibility scoring:
- Domain authority (0-25 points)
- Publication recency (0-20 points)
- Author credentials (0-20 points)
- Cross-reference count (0-20 points)
- Methodology transparency (0-15 points)
Total: 0-100 credibility score

### `contradiction-resolution.md`
Template for Stage 6 resolution output with side-by-side claim comparison,
resolution strategy (source authority / recency / consensus / escalate),
and final resolved position with confidence.

### `report-template.md`
OKF-compliant report structure:
```markdown
---
type: research-report
title: {{title}}
date: {{date}}
confidence: {{score}}
verification_status: {{status}}
sources_count: {{n}}
feynman_grade: {{grade}}
---

# {{title}}
...standard sections...

## Sources
...cited sources list...
```

### `research-package-manifest.json`
JSON schema for `.research` package `manifest.json`:
```json
{
  "version": "1.0.0",
  "okf_version": "0.1",
  "job_id": "...",
  "query": "...",
  "depth": "...",
  "created_at": "...",
  "stages_completed": [...],
  "sources_count": 0,
  "graph_nodes": 0,
  "contradictions_resolved": 0,
  "confidence": 0.0,
  "feynman_grade": null,
  "files": {...}
}
```

## Acceptance Criteria

- [ ] All 5 scripts exist in `skills/research/deep-research/scripts/`
- [ ] All 5 scripts are executable (`ls -la scripts/` shows `-rwxr-xr-x`)
- [ ] All 5 scripts have `#!/usr/bin/env bash` + `set -euo pipefail`
- [ ] All 5 templates exist in `skills/research/deep-research/templates/`
- [ ] `research-package-manifest.json` is valid JSON (`python3 -m json.tool`)
