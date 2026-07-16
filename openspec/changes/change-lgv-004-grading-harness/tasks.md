# Tasks — change-lgv-004-grading-harness

- [ ] Write scripts/run-eval-harness.md documenting the per-item agent invocation protocol
- [ ] Run learn-grade (agent-executed) against all 20+ eval items via parallel fan-out
- [ ] Capture each result as references/eval-dataset/results/<item_id>.json matching the grade result schema from learn-grade/SKILL.md
- [ ] Verify every item produced a result (no silent drops)
- [ ] Spot-check 3-5 results manually for sanity before proceeding to metrics
- [ ] Commit the change
