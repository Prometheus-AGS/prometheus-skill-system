# Feynman Quality Gate

Stage 09 (Report) routes the draft report through `learn-grade` before delivery.
This gate ensures the report accurately represents the evidence without gaps,
misconceptions, or unsupported claims.

## Gate Threshold

Both conditions must be satisfied:

| Condition | Threshold | Meaning |
|-----------|-----------|---------|
| `overall_score` | ≥ 0.7 | Report adequately covers the research question |
| `misconceptions_absent` | == 1.0 | No factual errors or unsupported claims detected |

Failing either condition triggers a re-synthesis with the grade feedback incorporated.

## How learn-grade is Invoked

Stage 09 constructs a grading rubric from the Stage 01 sub-questions:

```
learn-grade:
  content: <draft_report_text>
  rubric:
    - "Does the report answer sub-question 1: <question>?"
    - "Does the report answer sub-question 2: <question>?"
    - "Are all claims supported by cited sources?"
    - "Are contradictions acknowledged and resolved?"
  strictness: standard
```

The grade is returned as:
```json
{
  "overall_score": 0.82,
  "misconceptions_absent": 1.0,
  "gaps": [],
  "feedback": "Report adequately covers all sub-questions..."
}
```

## Re-synthesis Loop

```
Draft report → learn-grade → PASS (both thresholds met) → deliver
                           → FAIL → incorporate feedback → re-synthesize once
                                                        → learn-grade again
                                                        → FAIL again → deliver with warning
```

Maximum re-synthesis attempts: 1. After the second failure, the report is
delivered with `feynman_grade` set to the failing score and a warning banner.

## Bypassing the Gate

```
/deep-research --skip-feynman "my query"
```

Or set `RESEARCH_SKIP_FEYNMAN=1`. When bypassed:
- `feynman_grade: null` in manifest
- `verification_status: partial` (unless all other gates pass at `verified`)

## Availability Check

When `learn-grade` skill is unavailable (not installed):
- Gate is skipped automatically
- `manifest.json` records `"feynman_gate_used": false`
- `feynman_grade: null`
- No error is raised; research continues to export

## Anti-Sycophancy

The Feynman gate specifically catches pedagogical sycophancy —
where a report sounds confident and comprehensive but has gaps in
evidence coverage. A report that makes the reader feel informed when
gaps exist is rejected (when `misconceptions_absent < 1.0`).
