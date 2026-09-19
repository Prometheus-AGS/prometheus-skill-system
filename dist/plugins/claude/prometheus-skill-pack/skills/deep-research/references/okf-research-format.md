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
| `verification_status` | enum | `verified` / `partial` / `unverified`, derived from claim labels (see Verification Status Rules) |
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

## Claim Labels

Every claim in `graph.json`, every citation in `citations.json`, every entry in
`contradictions.json`, and every row of the report's evidence table carries one
label from this vocabulary. The labels are adapted from Feynman CLI
(companion-inc/feynman, MIT) and mean exactly this:

| Label | Meaning | Required evidence |
|-------|---------|-------------------|
| `verified` | A cited source was fetched and its text supports the specific statement, number, or conclusion attached to it. Topic overlap is not support. | the passage or artifact path, in `claims[].evidence` |
| `unverified` | A source is cited but its support for the statement was not checked. | none; the label itself is the admission |
| `blocked` | Verification was attempted and could not complete: dead link, fetch or parse failure, gateway unavailable, PDF parsing refused. | what was attempted and why it stopped, in `claims[].evidence` |
| `inferred` | The pipeline's own inference from other claims; no source states it. | the claims it is inferred from, in `claims[].sources` or `evidence` |

Rules that follow from the meanings:

- A `verified` label may only be assigned by a step that fetched the source in
  this run (stage 05 or the verifier agent), never copied from a search snippet.
- A claim with no source may only be `inferred`.
- Removing a dead source removes the `verified` label from every claim that
  depended solely on it; those claims become `blocked` with the dead URL named.
- Words such as `verified`, `confirmed`, or `checked` in the report prose are
  permitted only for claims whose label is `verified`.

A claim is **critical** (`claims[].critical: true`) when it appears in the
executive summary or as a Key Finding. Critical claims decide the package label.

## Verification Status Rules

`verification_status` in `report.md` frontmatter and `manifest.json` is
**derived** from the claim labels and the run state, never set by hand. The
drift check recomputes it and fails when the declared value differs.

The claim set is `graph.json.claims[]` when the graph has claims. A run
without a graph (direct or shallow scale skips stage 07) uses the claims
stage 05 labelled in `sources/credibility.json` (`verified_sources[].claims[]`),
and every one of them counts as critical.

| Status | Condition (first match wins) |
|--------|------------------------------|
| `unverified` | stage 05 is not in `stages_completed`, or the claim set is empty, or no claim carries a label |
| `partial` | any critical claim is `blocked`, `unverified`, or `inferred`; or any claim at all is `blocked` or `unverified`; or the adversarial review of the report returned CRITICAL, was refused (stage 05 artifact missing or invalid at review time), could not reach a judge, or was not run on a full-scale run; or the Feynman gate was skipped or failed |
| `verified` | every critical claim is `verified`, no claim is `blocked` or `unverified`, the Feynman gate was used and passed (`feynman_grade ≥ 0.7`, `misconceptions_absent == 1.0`), and the adversarial review ran without a CRITICAL finding |

The same rule as jq, over `graph.json` (`$g`), `sources/credibility.json`
(`$s`), and `checkpoint.json` (`$c`):

```jq
def claimset: if (($g.claims // []) | length) > 0 then $g.claims
              else [$s.verified_sources[]?.claims[]? | . + {critical: true}] end;
def labels: [claimset[] | .label | select(. != null)];
def critical: [claimset[] | select(.critical == true)];
if (($c.stages_completed | index("05")) == null) or ((labels | length) == 0) then "unverified"
elif ([critical[] | select(.label != "verified")] | length) > 0
     or ([claimset[] | select(.label == "blocked" or .label == "unverified")] | length) > 0
     or (($c.review.verdict // "") == "BLOCK")
     or (($c.blocked_review // null) != null)
     or ($c.scale == "full" and ($c.integrations.adversarial_review_used | not))
     or ($c.integrations.feynman_gate_used | not)
     or (($c.feynman_grade // 0) < 0.7)
     or (($c.misconceptions_absent // 0) < 1.0)
then "partial"
else "verified" end
```

`checkpoint.json.review`, `feynman_grade`, and `misconceptions_absent` are
written by the driver when the adversarial review (change-rah-006) and the
stage 09 gate record their results; until they exist a full run derives to
`partial`, which is the honest value for a run whose report was never judged.
The drift check (`scripts/check-research-package.sh --package`) applies this
rule and, when `checkpoint.json` lacks the two gate values, reads them from
the `report.md` frontmatter, which stage 09 writes from the same grade.
Stage 09 cannot know the review outcome, so the value it writes is
provisional: after the report review (between stage 09 and 10) the driver
rewrites `report.md`'s `verification_status` to `check-research-package.sh
--derive <dir>`, and stage 10 copies it into the manifest. A run whose
gates and review all pass therefore ends `verified`; one that did not ends
`partial` or `unverified`, never a value the rule would reject.

The manifest's `verification_verdict` (`PASS`, `PASS WITH NOTES`, `BLOCKED`)
is a separate axis about the run, defined in `research-package-spec.md`.

## Package File Requirements

Every `.research` package must contain `manifest.json` and `report.md`.
All other files (`graph.json`, `citations.json`, `contradictions.json`, `sources/`)
are optional but created by default for `deep` and `exhaustive` depth runs.
