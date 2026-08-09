#!/usr/bin/env bash
# grader-regression-test.sh — snapshot-compare regression guard for
# learn-grade.
#
# Rationale (see assessment.md open question #2, resolved in plan.md):
# learn-grade is prose-executed (an LLM agent following SKILL.md), not a
# deterministic script. Re-running the full 24-item eval suite live on
# every CI run would be expensive and non-deterministic — a passing run
# today could fail tomorrow purely from model sampling variance, which
# would make this a flaky gate rather than a real regression signal.
#
# Instead, this script is a CHEAP, DETERMINISTIC snapshot-compare:
# it diffs the *shape* and *pass/fail status* of the current results/
# directory against the last known-good baseline captured in
# baseline-snapshot.json. It catches:
#   - A result file going missing (harness regression)
#   - A result file losing required schema fields (schema regression)
#   - An item that previously passed now failing pass/fail-wise, or vice
#     versa (a coarse behavior-change signal)
#
# It does NOT re-invoke the LLM grader and does NOT catch subtle score
# drift within the same pass/fail bucket — that requires the live
# re-validation script below, which is deliberately NOT part of CI.
#
# Usage:
#   grader-regression-test.sh                 # compare results/ vs baseline
#   grader-regression-test.sh --update-baseline  # snapshot current results/ as new baseline (manual, human-reviewed only)
#
# Exit codes: 0 = no regression, 1 = regression found, 2 = missing deps/files

set -euo pipefail

BASE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${BASE}/results"
BASELINE_PATH="${BASE}/baseline-snapshot.json"

if ! command -v python3 >/dev/null 2>&1; then
    echo "grader-regression-test: python3 is required" >&2
    exit 2
fi

if [ "${1:-}" = "--update-baseline" ]; then
    echo "grader-regression-test: snapshotting current results/ as new baseline"
    echo "grader-regression-test: this should only be run after a human has reviewed the diff"
    python3 - "$RESULTS_DIR" "$BASELINE_PATH" <<'PY'
import json, os, sys

results_dir, baseline_path = sys.argv[1], sys.argv[2]
snapshot = {}
for fname in sorted(os.listdir(results_dir)):
    if not fname.endswith(".json"):
        continue
    item_id = fname[: -len(".json")]
    r = json.load(open(os.path.join(results_dir, fname)))
    snapshot[item_id] = {
        "passed": r["passed"],
        "overall_score": r["overall_score"],
        "misconceptions_absent": r["scores"]["misconceptions_absent"],
        "required_fields_present": all(
            k in r
            for k in [
                "grade_id", "goal_id", "concept_id", "learner_id",
                "graded_at", "explanation_excerpt", "scores",
                "overall_score", "gaps", "transfer_problems",
                "passed", "pass_threshold",
            ]
        ),
    }

out = {"generated_at": "2026-07-16T22:00:00Z", "items": snapshot}
json.dump(out, open(baseline_path, "w"), indent=2)
print(f"Wrote baseline with {len(snapshot)} items to {baseline_path}")
PY
    exit 0
fi

if [ ! -f "$BASELINE_PATH" ]; then
    echo "grader-regression-test: no baseline-snapshot.json found." >&2
    echo "grader-regression-test: run with --update-baseline first (after human review)." >&2
    exit 2
fi

python3 - "$RESULTS_DIR" "$BASELINE_PATH" <<'PY'
import json, os, sys

results_dir, baseline_path = sys.argv[1], sys.argv[2]
baseline = json.load(open(baseline_path))["items"]

current = {}
for fname in sorted(os.listdir(results_dir)):
    if not fname.endswith(".json"):
        continue
    item_id = fname[: -len(".json")]
    r = json.load(open(os.path.join(results_dir, fname)))
    current[item_id] = {
        "passed": r["passed"],
        "overall_score": r["overall_score"],
        "misconceptions_absent": r["scores"]["misconceptions_absent"],
        "required_fields_present": all(
            k in r
            for k in [
                "grade_id", "goal_id", "concept_id", "learner_id",
                "graded_at", "explanation_excerpt", "scores",
                "overall_score", "gaps", "transfer_problems",
                "passed", "pass_threshold",
            ]
        ),
    }

failures = []

missing = set(baseline) - set(current)
for item_id in sorted(missing):
    failures.append(f"MISSING: {item_id} was in baseline but has no current result file")

new_items = set(current) - set(baseline)
if new_items:
    print(f"INFO: {len(new_items)} new item(s) not in baseline (not a failure): {sorted(new_items)}")

for item_id in sorted(set(baseline) & set(current)):
    b, c = baseline[item_id], current[item_id]
    if not c["required_fields_present"]:
        failures.append(f"SCHEMA: {item_id} is missing required grade-result fields")
    if b["passed"] != c["passed"]:
        failures.append(
            f"PASS/FAIL FLIP: {item_id} baseline passed={b['passed']} -> current passed={c['passed']} "
            f"(overall_score {b['overall_score']} -> {c['overall_score']})"
        )
    if b["misconceptions_absent"] != c["misconceptions_absent"]:
        failures.append(
            f"MISCONCEPTION FLIP: {item_id} baseline misconceptions_absent={b['misconceptions_absent']} "
            f"-> current={c['misconceptions_absent']}"
        )

if failures:
    print(f"grader-regression-test: FAIL — {len(failures)} regression(s) found")
    for f in failures:
        print(f"  - {f}")
    sys.exit(1)

print(f"grader-regression-test: OK — {len(set(baseline) & set(current))} items match baseline, no regressions")
PY
