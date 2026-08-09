# Eval Metrics Report

_Generated: 2026-07-16T21:00:00Z_

**Caveat**: 24/24 ground-truth items are still in `draft` review status — these metrics are provisional pending human review.

## Misconceptions Detection (binary classification)

- Precision: **0.9231**
- Recall: **1.0**
- F1: **0.96**
- Accuracy: **0.9583**
- Confusion matrix: TP=12 FP=1 TN=11 FN=0 (n=24)

### Disagreements

- `sp-003-incomplete-vague-mcp` (incomplete): gold_present=False, grader_present=True

## Continuous Dimension Correlation (grader score vs. gold score)

| Dimension | Pearson r | Spearman r | MAE |
|---|---|---|---|
| completeness | 0.9079 | 0.738 | 0.1571 |
| accuracy | 0.9377 | 0.935 | 0.0771 |
| clarity | 0.6093 | 0.6245 | 0.0879 |

## Worst 5 Items by Mean Absolute Error

| Item | Domain | Tier | Mean Abs Error |
|---|---|---|---|
| sp-001-strong-portability | skill-pack | strong | 0.2733 |
| kbd-001-strong-full-cycle | kbd-lifecycle | strong | 0.2333 |
| sp-002-strong-hooks-and-validate | skill-pack | strong | 0.2233 |
| cr-008-flawed-plants-no-respiration | cellular-respiration | factually-flawed | 0.1667 |
| sp-003-incomplete-vague-mcp | skill-pack | incomplete | 0.15 |
