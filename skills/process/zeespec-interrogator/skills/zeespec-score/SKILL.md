---
license: MIT
name: zeespec-score
version: '1.0.0'
description: >
  Score and report coverage on an existing ZeeSpec interrogation record.
  Reads the recorded answers from state and produces a coverage-score.json
  with per-dimension scores and a GO/CAUTION/NO-GO recommendation without
  re-running the interrogation.
metadata:
  tags: [process, orchestration, automation]
---

# /zeespec-score

Computes or refreshes the coverage score for an existing interrogation.
Use this after a partial session to check standing, or to re-score after
manually editing answers in the state file.

## Usage

```
/zeespec-score "<subject_name>"
```

## Setup

1. Parse `subject_name`
2. Load `scripts/state-resolve-provider.sh`
3. Verify state exists: `.zeespec/subjects/<subject_name>/state.json`
4. Run `scripts/score-coverage.sh <subject_name>`
5. Load `prompts/score.md` and display results
6. Output per-dimension table and GO/CAUTION/NO-GO recommendation to console

## Output

Per-dimension score table + aggregate + recommendation printed to console.
`coverage-score.json` written to state directory.
State file updated with `coverage_score`.
