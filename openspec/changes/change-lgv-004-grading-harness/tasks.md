# Tasks — change-lgv-004-grading-harness

- [x] Write scripts/run-eval-harness.md documenting the per-item agent invocation protocol
- [x] Run learn-grade (agent-executed) against all 20+ eval items via parallel fan-out
- [x] Capture each result as references/eval-dataset/results/<item_id>.json matching the grade result schema from learn-grade/SKILL.md
- [x] Verify every item produced a result (no silent drops)
- [x] Spot-check 3-5 results manually for sanity before proceeding to metrics
- [x] Commit the change
