# Proposal — change-lgv-007-regression-test-and-docs

Two deliverables closing the phase:

1. **Snapshot-based regression test** (`scripts/grader-regression-test.sh`):
   diffs future grading runs against the change-lgv-004/006 baseline
   results, catching prompt/schema regressions without re-invoking the
   LLM grader on every CI run (cost + determinism). A separate, optional
   manual script re-runs the harness live for periodic re-validation.

2. **`EVAL-RESULTS.md`**: final precision/recall/correlation numbers,
   confidence level achieved (replacing the "60-70%" placeholder),
   remaining known failure modes, and a summary of what change-lgv-006
   tuned (or why it was a no-op).

## Goal
G-05, G-06.
