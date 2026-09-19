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
| `verified_sources` | object[] | Sources with `credibility_score`, `flags[]`, and `claims[]` of `{text, label, evidence}` (label in `verified`, `unverified`, `blocked`) |
| `filtered_sources` | object[] | Sources below threshold (logged, not used) |
| `credibility_scores` | object | `{url: score}` map for Stages 06–09 |

## Instructions

1. **Score each source** by running the scorer against the package:

   ```bash
   bash scripts/verify-sources.sh <package_dir> [--threshold 40] [--penalties penalties.json]
   ```

   `verify-sources.sh` delegates to `scripts/score-sources.py`, which scores
   the five dimensions of the rubric in `agents/source-verifier.md` with the
   documented weights, **only where the registry carries evidence**:

   | Dimension | Weight | Evidence it needs |
   |---|---|---|
   | Domain authority | 0.25 | the URL (always present) |
   | Author expertise | 0.20 | `author`/`authors`, plus `author_credentials` or `affiliation` |
   | Citation depth | 0.20 | `references`, `reference_count`, or `citation_count`; else reference markers in the fetched chunks |
   | Publication recency | 0.20 | `published`/`date` |
   | Factual verifiability | 0.15 | extracted claims (share that carry a number, date, or measurable term) |

   A dimension with no evidence is **excluded from that source's denominator**
   rather than scored zero, and the renormalised vector is written per source
   as `applied_weights` (it sums to 1 over the dimensions that were present).
   A source with only a URL is therefore scored on domain authority alone and
   flagged `partial_evidence:<missing dimensions>`, never penalised for
   metadata nobody recorded. Sources are sorted by score, descending. This is
   Feynman CLI's available-signal renormalisation (companion-inc/feynman, MIT)
   applied to web sources.

   The scorer also writes `<package_id>/sensitivity.json`: the same sources
   re-ranked under four alternate weight vectors (`balanced`,
   `authority_heavy`, `recency_heavy`, `methodology_heavy`) with each rank
   classified `stable` (never moves), `sensitive` (moves by at most two
   places), or `volatile` (moves further). A volatile rank is inspected before
   the order is treated as decisive; the sidecar and report may cite it.

2. **Run sycophancy bias check** — call `detect_sycophancy` on each source's
   extracted claims. Penalize 10–20 points for high/critical severity patterns
   (over-confidence, contradiction suppression). See
   `references/sycophancy-correction-integration.md`. Pass the penalties to the
   scorer as `--penalties penalties.json` (`{url: points}`, capped at 30) so
   they are recorded in `credibility.json` and preserved across the
   sensitivity profiles.

3. **Apply threshold** — sources scoring < 40 are moved to `filtered_sources`
   with a `filter_reason`. Do not remove them from disk (Stage 06 may reference
   them for contradiction detection).

4. **Label every claim** (vocabulary in `references/okf-research-format.md`,
   "Claim Labels"). For each claim extracted in Stage 04, re-read the fetched
   chunk it cites and decide:
   - `verified` — the chunk's text supports the specific statement, number, or
     conclusion. Topic overlap is not support. Record the supporting passage in
     `evidence`.
   - `blocked` — the chunk is missing, the URL was dead or redirected to
     unrelated content, or the parse failed. Record what was attempted.
   - `unverified` — the claim cites a source you did not re-read in this run.
   - `inferred` — never assigned here; Stages 06 and 09 assign it to their own
     inferences.
   A source whose URL is dead loses every `verified` label that depended solely
   on it; those claims become `blocked` with the URL named. Never copy a label
   from a search snippet or from an earlier run.

5. **Update surreal-memory** — add `credibility:<score>` observation to each
   `ResearchSource` entity if surreal-memory is available.

6. **Write output** — the scorer writes `<package_id>/sources/credibility.json`
   (`verified_sources[]`, `filtered_sources[]`, `credibility_scores{}`,
   `threshold`, `weights`; per source `credibility_score`, `applied_weights`,
   `signals{}`, `flags[]`, `claims[]`) and `<package_id>/sensitivity.json`.
   The scorer emits every claim as `{text, label: "unverified", evidence:
   null}`; step 4 rewrites the labels it has actually checked. The scorer
   never assigns `verified`.

## Integration

`scripts/verify-sources.sh` → `scripts/score-sources.py` (scoring, sensitivity)
`sycophancy-correction` MCP: `detect_sycophancy(text, strictness="standard")`
`surreal-memory` MCP: `add_observations(entityName=<url>, observations=[...])`
Rubric prose: `agents/source-verifier.md` (dimensions), `templates/source-evaluation.md` (per-dimension criteria)

## Example

**Source:** `https://qdrant.tech/benchmarks/` (author and date recorded, no reference list)
```
domain_authority      0.70  available   established primary documentation
author_expertise      0.60  available   named engineers, credentials not recorded
citation_depth        —     missing     no reference metadata and no fetched text
publication_recency   1.00  available   published 2025-02-10
factual_verifiability 0.75  available   3 of 4 claims carry a number or measurable term
applied_weights: domain 0.3125, author 0.25, recency 0.25, verifiability 0.1875 (sum 1.0; citation_depth excluded)
sycophancy penalty: -5
credibility_score: 71 → RETAINED, rank 2, sensitivity: sensitive (rank 1 under recency_heavy, 3 under methodology_heavy)
```
