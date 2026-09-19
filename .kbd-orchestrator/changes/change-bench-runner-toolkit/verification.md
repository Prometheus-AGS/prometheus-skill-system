# Verification — change-bench-runner-toolkit

Repository: `prometheus-skill-pack`

## Acceptance criteria

- All 6 tasks complete (bash -n, /bin/bash -n, executable, file-content greps per task verify).
- `tools/bench-automation/run-bench-suite.sh --tasks 58` produces a complete package at `/Users/gqadonis/.prometheus/research/bench-g5/task-58/` with manifest.json, index.md, report.md, all 10 stage artifacts, AND `score-fact.py --package .../task-58/` returns `verified_claim_ratio > 0` (the drt-006 known gap closed by the label-claims.py pass).
- `tools/bench-automation/bench-runbook.sh` appends a row to `tests/bench/BENCH-RESULTS.md` for task-58 with the four numbers.

## Verify commands

```verify
bash -n skills/research/deep-research/scripts/run-stage.sh && /bin/bash -n skills/research/deep-research/scripts/run-stage.sh
bash -n tools/bench-automation/run-bench-suite.sh && /bin/bash -n tools/bench-automation/run-bench-suite.sh
bash -n tools/bench-automation/bench-runbook.sh && /bin/bash -n tools/bench-automation/bench-runbook.sh
python3 -c 'import ast; ast.parse(open("skills/research/deep-research/scripts/label-claims.py").read())'
test -x skills/research/deep-research/scripts/run-stage.sh
test -x tools/bench-automation/run-bench-suite.sh
test -x tools/bench-automation/bench-runbook.sh
test -s tools/bench-automation/README.md
```

## Evidence

Recorded at execution time: per-task BENCH-RESULTS.md rows; verified_claim_ratio per task; toolkit run logs; the full suite completes within wall-clock and gateway-quota bounds. No hosted CI.
