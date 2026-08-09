# Coverage Scoring Reference

## Per-Question Scoring

| Classification | Weight |
|---|---|
| `defined` | 1.0 — clear, specific, complete |
| `partial` | 0.5 — directional but has unresolved gaps |
| `implicit` | 0.0 — skipped or "AI decides" |

## Per-Dimension Score

```
dimension_score = Σ(question_weights) / questions_in_dimension
```

Default: 10 questions per dimension. If a dimension was interrogated with
fewer questions (partial session), unanswered questions count as `implicit`.

## Aggregate Score

```
aggregate_score = mean(all non-skipped dimension scores)
```

Fully skipped dimensions (user explicitly skipped the entire dimension) are
excluded from the aggregate but noted in the manifest.

## Per-Dimension Thresholds

| Dimension | Critical Threshold | Failure Behavior |
|---|---|---|
| `why` | 0.70 | Forces NO-GO regardless of aggregate |
| `who` | 0.65 | Forces NO-GO regardless of aggregate |
| `when` | 0.60 | Forces NO-GO regardless of aggregate |
| `what` | 0.50 | Contributes to aggregate only |
| `where` | 0.50 | Contributes to aggregate only |
| `how` | 0.50 | Contributes to aggregate only |

`why`, `who`, and `when` are critical dimensions. Their threshold failure
overrides even a high aggregate score. The rationale: these three dimensions
define the purpose, governance, and event model of the system. A system with
excellent `how` coverage but no `why` definition cannot be validated as
correct — there is no agreed criterion for correctness.

## Aggregate Thresholds

| Aggregate Score | Status | GO Recommendation |
|---|---|---|
| >= 0.85 | `sufficient` | GO |
| 0.60 – 0.84 | `partial` | CAUTION |
| < 0.60 | `insufficient` | NO-GO |

## Recommendation Resolution Algorithm

```
IF any(critical dimension score < critical threshold):
    recommendation = NO-GO
    reason = "Critical dimension(s) below threshold: <list with scores>"
ELSE IF aggregate_score >= 0.85:
    recommendation = GO
ELSE IF aggregate_score >= 0.60:
    recommendation = CAUTION
ELSE:
    recommendation = NO-GO
    reason = "Aggregate score insufficient: <score> < 0.60"
```

## Blocked-Until Derivation

A gap is added to `blocked_until` when:
1. It is a `why`, `who`, or `when` implicit answer (always blocking)
2. It is a partial answer where the unresolved portion is a binary architectural decision
   (e.g., a `how` partial answer about whether to use CRDT vs. OT for sync)

A gap is NOT added to `blocked_until` when:
1. It is a `what`, `where`, or `how` implicit answer that does not affect architectural decisions
2. It is a partial answer where the resolved portion is sufficient for planning to proceed

The distinction between `gaps.critical`, `gaps.major`, and `blocked_until`:
- `gaps.critical` — dimension-based classification (why/who/when implicit answers)
- `gaps.major` — severity-based classification (what/where/how implicit, partial answers with significant unresolved gaps)
- `blocked_until` — action-based classification (must be resolved before proceeding)

These overlap but are not identical. A `gaps.critical` gap is almost always in
`blocked_until`. A `gaps.major` gap may or may not be in `blocked_until`.

## Configurable Thresholds

The default thresholds above can be overridden per-invocation via:

```json
{
  "coverage_threshold": 0.75,
  "dimension_thresholds": {
    "why": 0.80,
    "who": 0.70,
    "when": 0.65
  }
}
```

Override via the `coverage_threshold` input field. Per-dimension overrides
are provided in the request file under `dimension_thresholds`.

Lowering the `why` threshold below 0.70 is permitted but will generate a
manifest warning: "why threshold lowered below recommended minimum."
