#!/usr/bin/env bash
# score-coverage.sh — Compute ZeeSpec coverage score from interrogation record
# Usage: score-coverage.sh <subject_name>
# Reads: .zeespec/<subject>/state.json → interrogation_record
# Writes: .zeespec/<subject>/coverage-score.json
# Output: JSON coverage score to stdout
# Exit 0 = OK, Exit 1 = error

set -euo pipefail

SUBJECT_NAME="${1:?Usage: score-coverage.sh <subject_name>}"
STATE_DIR="${ZEESPEC_STATE_DIR:-.zeespec}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
STATE_FILE="${SUBJECT_DIR}/state.json"
SCORE_FILE="${SUBJECT_DIR}/coverage-score.json"

if [ ! -f "$STATE_FILE" ]; then
  echo "❌ No state found for subject: ${SUBJECT_NAME}" >&2
  exit 1
fi

python3 << 'SCORE_SCRIPT'
import json, os, sys

subject_name = os.environ.get("SUBJECT_NAME", "")
state_dir = os.environ.get("STATE_DIR", ".zeespec")
timestamp = os.environ.get("TIMESTAMP", "")
subject_dir = f"{state_dir}/subjects/{subject_name}"
state_file = f"{subject_dir}/state.json"
score_file = f"{subject_dir}/coverage-score.json"

state = json.load(open(state_file))
record = state.get("interrogation_record", {})
answers = record.get("answers", {})

CRITICAL_THRESHOLDS = {
    "why": 0.70,
    "who": 0.65,
    "when": 0.60,
    "what": 0.50,
    "where": 0.50,
    "how": 0.50,
}

WEIGHT = {"defined": 1.0, "partial": 0.5, "implicit": 0.0}

def score_dimension(dim_answers):
    if not dim_answers:
        return {
            "raw_score": 0.0,
            "defined_count": 0,
            "partial_count": 0,
            "implicit_count": 10,
            "skipped": True
        }
    total = 10
    weights = [WEIGHT.get(a.get("classification", "implicit"), 0.0) for a in dim_answers]
    padded = weights + [0.0] * (total - len(weights))
    raw = sum(padded) / total
    return {
        "raw_score": round(raw, 4),
        "defined_count": sum(1 for a in dim_answers if a.get("classification") == "defined"),
        "partial_count": sum(1 for a in dim_answers if a.get("classification") == "partial"),
        "implicit_count": sum(1 for a in dim_answers if a.get("classification") == "implicit") + (total - len(dim_answers)),
        "skipped": False
    }

per_dimension = {}
critical_failures = []
active_scores = []

for dim in ["why", "who", "when", "what", "where", "how"]:
    dim_answers = answers.get(dim, [])
    result = score_dimension(dim_answers)
    threshold = CRITICAL_THRESHOLDS[dim]
    met = result["raw_score"] >= threshold

    if result["raw_score"] < 0.85:
        status = "sufficient" if result["raw_score"] >= 0.85 else ("partial" if result["raw_score"] >= 0.60 else "insufficient")
    else:
        status = "sufficient"

    status = "sufficient" if result["raw_score"] >= 0.85 else ("partial" if result["raw_score"] >= 0.60 else "insufficient")

    per_dimension[dim] = {
        "raw_score": result["raw_score"],
        "status": status,
        "critical_threshold": threshold,
        "critical_threshold_met": met,
        "defined_count": result["defined_count"],
        "partial_count": result["partial_count"],
        "implicit_count": result["implicit_count"],
        "skipped": result["skipped"]
    }

    if dim in ("why", "who", "when") and not met:
        critical_failures.append(dim)

    if not result["skipped"]:
        active_scores.append(result["raw_score"])

aggregate = round(sum(active_scores) / len(active_scores), 4) if active_scores else 0.0
aggregate_status = "sufficient" if aggregate >= 0.85 else ("partial" if aggregate >= 0.60 else "insufficient")

if critical_failures:
    recommendation = "NO-GO"
    reason = f"Critical dimension(s) below threshold: {', '.join(critical_failures)}"
elif aggregate >= 0.85:
    recommendation = "GO"
    reason = f"Aggregate coverage {aggregate:.0%} meets sufficient threshold (>=85%)"
elif aggregate >= 0.60:
    recommendation = "CAUTION"
    reason = f"Aggregate coverage {aggregate:.0%} is partial (60–84%)"
else:
    recommendation = "NO-GO"
    reason = f"Aggregate coverage {aggregate:.0%} is insufficient (<60%)"

score = {
    "computed_at": timestamp,
    "per_dimension": per_dimension,
    "aggregate_score": aggregate,
    "aggregate_status": aggregate_status,
    "critical_failures": critical_failures,
    "go_recommendation": recommendation,
    "recommendation_reason": reason
}

json.dump(score, open(score_file, "w"), indent=2)

# Update state
state["coverage_score"] = score
state["updated_at"] = timestamp
json.dump(state, open(state_file, "w"), indent=2)

print(json.dumps(score, indent=2))
SCORE_SCRIPT

exit 0
