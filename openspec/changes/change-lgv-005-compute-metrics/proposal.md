# Proposal — change-lgv-005-compute-metrics

Compute the actual precision/recall + correlation numbers from
change-lgv-004's results vs change-lgv-002/003's ground truth.

- **misconceptions_absent**: binary classification — precision (of
  flagged misconceptions, how many are real) and recall (of real
  misconceptions, how many were flagged).
- **completeness / accuracy / clarity**: continuous 0-1 — Pearson and
  Spearman correlation between grader score and gold-standard score.

Output a metrics summary (JSON + human-readable table) and a list of
specific failure-mode examples (which items scored furthest from gold,
and why) to feed change-lgv-006.

## Goal
G-03.
