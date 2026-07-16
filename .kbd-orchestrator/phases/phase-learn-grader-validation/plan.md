# Plan — phase-learn-grader-validation

_Generated: 2026-07-16_

## Change Backend
OpenSpec (`openspec/` directory present at project root)

## Analyze stage

Skipped, per assessment recommendation — this phase is content
construction + measurement, not library adoption. No external research
needed.

## Resolved open questions (from assessment)

1. **Ground-truth authorship** — Claude drafts explanations + candidate
   misconception annotations per domain; user reviews/corrects before the
   dataset is treated as ground truth. Flagged explicitly in change-002's
   tasks so it isn't silently skipped.
2. **G-05 regression style** — snapshot-compare (record G-02's grades as
   the "last known good" baseline; regression test diffs future grades
   against it). Live-agent re-grade documented as an optional manual
   script, not the CI-safe default.
3. **Domain selection — RESOLVED via codebase check.** Two real corpora
   already exist and meet the schema exactly:
   `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` (18 sources, 6
   misconceptions) and `docs/learn/meta-corpus/skill-pack-corpus.json`
   (16 sources, 7 misconceptions). Reuse both — only the third domain
   needs a fresh corpus (recommend a domain unrelated to software/KBD,
   e.g. a general-science or history topic, so the eval isn't
   self-referentially testing the pack's own docs).
4. **G-02 cost/batching** — use a `pipeline()`-style fan-out (or
   sequential `Agent` calls if Workflow orchestration isn't warranted for
   20 items) so grading runs are parallel, not 20+ sequential manual
   turns.

## Overview

7 changes. Content-construction changes (001-003) front-load because
every later change depends on the dataset existing. Harness + run (004)
depends on the dataset. Metrics (005) depends on results. Tuning (006) is
conditional on what 005 finds. Regression test + docs (007) closes the
phase.

## Changes

| # | Change ID | Goals | Description |
|---|-----------|-------|--------------|
| 1 | `change-lgv-001-third-domain-corpus` | G-01 | Build a fresh corpus JSON for a third domain (non-software, e.g. general science or history) in the `content-grounding.sh` schema: 10-15 sources, 3-5 misconceptions. Store at `skills/learn/learn-grade/references/eval-dataset/corpora/`. |
| 2 | `change-lgv-002-eval-explanations` | G-01 | Write 20+ Feynman-style explanations across the 3 domains (kbd-lifecycle, skill-pack, new third domain) — mix of good, partially-flawed, and misconception-containing explanations. Draft ground-truth annotations (per-dimension gold scores + misconceptions present/absent) for each. **Explicitly flag for human review before treating as ground truth** (open question #1). |
| 3 | `change-lgv-003-dataset-schema-and-index` | G-01 | Define and document the eval item schema (explanation + corpus_ref + ground_truth) in `references/eval-dataset/SCHEMA.md`; write an `index.json` listing all eval items for the harness to iterate. |
| 4 | `change-lgv-004-grading-harness` | G-02 | Write `scripts/run-eval.sh` (or a workflow script) that, for each eval item, packages the explanation + corpus and invokes the `learn-grade` protocol (agent-executed), capturing the resulting grade JSON. Store raw results under `references/eval-dataset/results/`. |
| 5 | `change-lgv-005-compute-metrics` | G-03 | Write `scripts/compute-eval-metrics.py` (or shell+jq): precision/recall for `misconceptions_absent` (binary), Pearson/Spearman correlation for `completeness`/`accuracy`/`clarity` vs gold scores. Output a metrics summary JSON + human-readable table. |
| 6 | `change-lgv-006-tune-grader` | G-04 | Based on change-005's failure-mode findings, adjust `learn-grade/SKILL.md`'s grading rubric/prompts where systematic errors are found. Re-run affected eval items (subset of change-004's harness) to confirm improvement. If no systematic failures found, this change documents "no tuning needed" with evidence — not skipped, just possibly a no-op with justification. |
| 7 | `change-lgv-007-regression-test-and-docs` | G-05, G-06 | Snapshot-compare regression script (`scripts/grader-regression-test.sh`) that diffs future grading runs against change-004's baseline results. Write `references/EVAL-RESULTS.md` with final precision/recall/correlation numbers, replacing the "60-70%" placeholder confidence with the measured value. Wire the regression script into `.github/workflows/validate.yml` or document it as a manual pre-release check if live-agent invocation in CI is judged too costly. |

**Total estimated tasks: ~40** (to be finalized per-change during `/kbd-apply`)

## Execution Order Rationale

- **001 → 002 → 003**: dataset construction must land before anything can
  run against it. 003 (schema + index) comes after the raw content (001,
  002) so the index reflects what was actually written, not a
  speculative shape.
- **004 depends on 001-003**: the harness needs the dataset to exist.
- **005 depends on 004**: metrics need real grading results.
- **006 depends on 005**: tuning targets specific failure modes found.
- **007 last**: the regression test needs a stable baseline (ideally
  post-tuning, from 006) and the docs need final numbers.

## Apply Commands

```
/kbd-apply change-lgv-001-third-domain-corpus
/kbd-apply change-lgv-002-eval-explanations
/kbd-apply change-lgv-003-dataset-schema-and-index
/kbd-apply change-lgv-004-grading-harness
/kbd-apply change-lgv-005-compute-metrics
/kbd-apply change-lgv-006-tune-grader
/kbd-apply change-lgv-007-regression-test-and-docs
```

## First Change

```
/kbd-apply change-lgv-001-third-domain-corpus
```
