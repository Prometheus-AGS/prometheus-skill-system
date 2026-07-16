# Proposal — change-lgv-004-grading-harness

Write the harness that runs `learn-grade`'s agent-executed protocol
against every item in `index.json`. Since grading is prose-executed (not
a script — see assessment.md), the harness packages each item's inputs
and drives an `Agent` invocation per item following the SKILL.md
protocol verbatim, capturing the resulting grade JSON.

Use a fan-out pattern (parallel `Agent` calls or a `Workflow`
`pipeline()`) rather than 20+ sequential manual turns, per the
assessment's cost/batching recommendation.

Store raw results (one grade JSON per item) under
`references/eval-dataset/results/<item_id>.json`.

## Goal
G-02.
