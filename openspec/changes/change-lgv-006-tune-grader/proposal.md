# Proposal — change-lgv-006-tune-grader

Using change-lgv-005's failure-mode findings, adjust `learn-grade/SKILL.md`'s
grading rubric or instructions where systematic errors are found (e.g.,
consistently misses subtle misconceptions, over-penalizes unusual but
correct phrasing). Re-run the affected subset of eval items through the
change-lgv-004 harness to confirm the tuning improved the metric.

If change-lgv-005 finds no systematic failure pattern (only noise-level
variance), this change documents that finding with evidence rather than
inventing speculative tuning — a justified no-op is an acceptable
outcome here.

## Goal
G-04.
