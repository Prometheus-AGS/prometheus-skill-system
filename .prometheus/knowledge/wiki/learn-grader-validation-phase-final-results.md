---
type: Reference
id: learn-grader-validation-phase-final-results
title: Learn Grader Validation Phase Final Results
tags:
- learn-grader-validation
- learn-grade
- regression-test
- evaluation-dataset
- ci-validation
- phase-completion
links:
- learn-grader-validation-executor-completion-record
sources:
- stdin
- manual:phase-learn-grader-validation
timestamp: 2026-07-16T20:58:38.547910+00:00
created_at: 2026-07-16T20:58:38.547910+00:00
updated_at: 2026-07-16T20:58:38.547910+00:00
revision: 0
---

## Phase Context

`phase-learn-grader-validation` was created to close the highest-severity open risk from `phase-learn-feynman` v1.4.0: `learn-grade` had shipped with only 60–70% assessed confidence and no empirical validation dataset. The risk was that a grader missing misconceptions would provide false assurance.

## Original Goals

- **G-01: Grader evaluation dataset**
  - Assemble 20+ Feynman explanations.
  - Cover at least 3 subject domains, e.g. STEM, humanities, technical/programming.
  - Add expert-authored ground-truth annotations:
    - misconceptions present
    - misconceptions absent
    - gold-standard score
  - Store under `skills/learn/learn-grade/references/eval-dataset/` using machine-readable JSON or YAML per explanation.
- **G-02: Run `learn-grade` against the dataset**
  - Feed each explanation through the actual `learn-grade` skill/script path, not a mock.
  - Capture grader score and misconception list.
  - Diff results against ground truth.

## Completion Summary

The phase completed after `kbd-apply` for `change-lgv-007-regression-test-and-docs` finished all 5/5 tasks and verified the results. The session reported:

```text
Phase: phase-learn-grader-validation
Step: 7 of 7 — all changes complete, all 6 goals MET
Next: /kbd-reflect phase-learn-grader-validation
```

This entry records the final measured validation outcome, unlike generic executor completion markers such as [Learn Grader Validation Executor Completion Record](/learn-grader-validation-executor-completion-record.md).

## Delivered Artifacts

- Snapshot-compare CI regression guard:
  - `grader-regression-test.sh`
  - `baseline-snapshot.json`
- CI integration:
  - `learn-grade-regression` job added to `validate.yml`
- Human-readable validation summary:
  - `EVAL-RESULTS.md`
- Documentation update:
  - `learn-grade/SKILL.md` no longer contains the original 60–70% confidence placeholder.
  - It now reports measured empirical validation metrics.

## Measured Results

`learn-grade` validation produced the following measured performance:

| Metric | Result |
|---|---:|
| Misconception detection F1 | 0.96 |
| Accuracy score correlation | r = 0.94 |
| Completeness score correlation | r = 0.91 |
| Clarity score correlation | r = 0.61 |

## Repository State

Both validation/regression commits were pushed to `main`.

# Citations

1. [1] stdin
2. [2] manual:phase-learn-grader-validation