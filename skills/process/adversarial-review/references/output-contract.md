# Output Contract

Canonical schema: [`../assets/schemas/findings.schema.json`](../assets/schemas/findings.schema.json).

## Findings document

```json
{
  "mode": "diff | artifact",
  "verdict": "PASS | BLOCK",
  "judge_model": "provider/model-id",
  "isolation_mode": "liter-llm | harness-native",
  "findings": [
    {
      "severity": "CRITICAL | WARNING | SUGGESTION",
      "file": "path or artifact name",
      "line": 42,
      "claim": "one-sentence defect statement",
      "evidence": "hunk / criterion / quote proving the claim",
      "suggested_fix": "optional concrete fix"
    }
  ]
}
```

- `verdict` is derived, never judged: `BLOCK` iff ≥1 `CRITICAL` finding.
  `dispatch-judge.sh` recomputes it after shape-checking, so a judge cannot
  hand-wave a `PASS` over its own CRITICAL findings.
- Findings missing `claim` or `evidence`, or with an unknown severity, are
  dropped during normalization — unsupported assertions never reach the gate.
- `isolation_mode` is diagnostic honesty: `harness-native` marks reviews run
  through the fallback subagent (same model family as the session), so later
  audits can weight them accordingly.

## Gate semantics per caller

### Diff mode (kbd-execute Per-Change QA Gate)

| Verdict / severity | Effect |
|---|---|
| `BLOCK` (any CRITICAL) | `certification: BLOCKED` in `progress.json`; fix the change, then re-run `refine-validate` **and** `adversarial-review` before archive |
| WARNING | persisted in `.kbd-orchestrator/phases/<phase>/review/<change-id>/`; archive proceeds |
| SUGGESTION | informational only |

### Artifact mode (kbd-assess / kbd-analyze / kbd-plan)

| Verdict / severity | Effect |
|---|---|
| `BLOCK` (any CRITICAL) | revise the artifact, re-vet; max 2 rounds, then accept with an **"Unresolved review findings"** section appended to the artifact so the next stage inherits them explicitly |
| WARNING | appended to the stage handoff summary (`kbd_stage_handoff_write`) |
| SUGGESTION | informational only |

The bounded revise loop is deliberate: like the sycophancy gates' 2-rejection
soft cap, it prevents an infinite loop while guaranteeing the failure is
**visible** (never silently accepted).

## Severity calibration

- `CRITICAL` — proceeding uncorrected produces defective output: shipped bug,
  violated acceptance criterion, breached blocking constraint, or (artifact
  mode) a downstream stage building on a wrong premise.
- `WARNING` — real risk or quality problem the team should see; not blocking.
- `SUGGESTION` — improvement; ignorable without consequence.

This matches the OpenSpec reporting convention already used across the repo —
this skill is a new producer of an existing shape, not a new concept.
