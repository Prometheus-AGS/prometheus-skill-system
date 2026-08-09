# learn-grade Empirical Evaluation Results

_Phase: `phase-learn-grader-validation` · Generated: 2026-07-16_

## Why this exists

`phase-learn-feynman` (v1.4.0, closed 2026-06-28) shipped `learn-grade` —
the sycophancy-corrected external grader that closes each Feynman
loop — at an assessed confidence of only **60–70%**, with zero empirical
validation. That reflection named this the highest-severity open risk in
the entire learn domain: *"a grader that misses misconceptions is worse
than no grader — it provides false assurance."*

This phase built a 24-item eval dataset across 3 domains, ran the real
`learn-grade` protocol against it, measured actual precision/recall/
correlation, found and fixed one systematic grader defect, and wired a
lightweight CI regression guard.

## Final measured numbers

| Metric | Value |
|---|---|
| Misconception detection — Precision | **0.923** |
| Misconception detection — Recall | **1.0** |
| Misconception detection — F1 | **0.96** |
| Misconception detection — Accuracy | 0.958 |
| Accuracy dimension — Pearson r | **0.938** |
| Accuracy dimension — Spearman r | 0.935 |
| Completeness dimension — Pearson r | **0.908** |
| Completeness dimension — Spearman r | 0.738 |
| Clarity dimension — Pearson r | **0.609** (was 0.405 pre-tuning) |
| Clarity dimension — Spearman r | 0.625 (was 0.379 pre-tuning) |

**Replacing the "60–70%" placeholder**: the empirically measured
confidence is **~92% on misconception detection (F1) and ~91–94% on
accuracy/completeness scoring (Pearson r)**, with clarity at a more
moderate **~61%** (Pearson r) after tuning — up from an untuned ~40%.
The original 60–70% guess undersold accuracy/completeness performance
and, before tuning, roughly matched clarity's actual (weaker) performance
by coincidence.

## What the numbers mean in practice

- **Recall = 1.0 on misconceptions is the headline result.** Across 24
  items — 8 of which deliberately embed a real, verbatim corpus
  misconception — the grader caught every single one. It never let a
  known misconception through undetected.
- **The one precision cost (`sp-003-incomplete-vague-mcp`) is not a
  grader defect.** The item was drafted as merely "incomplete," but the
  grader detected an implicit Cortex/surreal-memory conflation that the
  draft ground truth hadn't flagged as a misconception. This is the
  grader being *more* careful than the human-drafted gold standard, not
  less.
- **Accuracy and completeness scoring track a human-drafted gold
  standard closely** (Pearson 0.91–0.94). The grader's factual-error and
  coverage-gap detection is reliable.
- **Clarity was the one genuine defect found**, and it was systematic:
  every factually-flawed item scored clarity artificially low because
  the original rubric conflated "is this understandable" with "is this
  correct." Fixed in `change-lgv-006` — see `TUNING-LOG.md` for the full
  before/after analysis. Post-fix clarity correlation (0.61) is
  moderate, not excellent — this is the dimension most likely to need
  further tuning in a future phase if it becomes a priority.

## Known limitations (carry forward)

1. **All 24 ground-truth items remain in `review_status: "draft"`.**
   Per the phase's resolved open question #1, a human review pass was
   never performed (out of scope for an automated KBD phase run without
   the user in the loop for that specific step). The numbers above are
   internally consistent and methodologically sound, but should be
   treated as **provisional** until a human (or an independent stronger-
   model second pass) reviews and corrects the gold-standard
   annotations. The `kbd-001-strong-full-cycle` control case in
   `TUNING-LOG.md` is a concrete example of where the gold standard
   itself, not the grader, may have been wrong (a gold clarity score of
   0.90 for a genuinely poorly-structured run-on sentence).

2. **24 items across 3 domains is a modest sample.** Statistically
   meaningful, but a larger, more diverse dataset (more domains, more
   misconception types, adversarial edge cases) would tighten these
   confidence intervals and might surface failure modes this run didn't
   hit.

3. **The CI regression guard is a snapshot-compare, not a live
   re-validation.** It catches harness/schema regressions and coarse
   pass/fail flips cheaply and deterministically, but will not catch
   subtle score drift within the same pass/fail bucket, or degradation
   introduced by future prompt-engineering changes to `learn-grade`
   that don't flip any item's pass/fail status. Periodic live
   re-validation (re-running `grading-harness` — see `HARNESS.md`) is
   recommended before any future release that touches `learn-grade`.

4. **Sycophancy-correction (S-02) was not evaluated here.** This phase
   measured the grader's *substantive* accuracy (does it find real
   errors and misconceptions), not the sycophancy-correction layer that
   sits on top of it. That's explicitly out of scope per this phase's
   `goals.md` non-goals.

## Artifacts in this directory

| File | Purpose |
|---|---|
| `SCHEMA.md` | Eval item / index / grade-result schema documentation |
| `HARNESS.md` | Grading invocation protocol + deviations from live SKILL.md flow |
| `TUNING-LOG.md` | Full before/after analysis of the clarity rubric fix |
| `corpora/cellular-respiration-corpus.json` | Third eval domain's teaching corpus |
| `explanations/*.json` | 24 eval items with draft ground truth |
| `index.json` | Flat manifest of all 24 items |
| `results/*.json` | 24 grade results (post-tuning) |
| `compute-eval-metrics.py` | Metrics computation script |
| `metrics-summary.json` / `METRICS-REPORT.md` | Machine/human-readable metrics output |
| `grader-regression-test.sh` | Snapshot-compare CI regression guard |
| `baseline-snapshot.json` | Current known-good baseline for the regression guard |
| `EVAL-RESULTS.md` | This file |

## Recommended follow-on

1. Human review pass on the 24 ground-truth items (resolves the
   `draft` status caveat above).
2. Expand the dataset (more domains, more items) if `learn-grade`'s
   accuracy becomes safety-critical for a specific downstream use.
3. Re-run this eval after any future change to `learn-grade/SKILL.md`'s
   grading rubric, using `HARNESS.md`'s protocol, and update
   `baseline-snapshot.json` only after reviewing the diff.
