#!/usr/bin/env bash
# integration-basic-flow.sh
# Integration test: learn-goal → learn-survey → feynman-loop → learn-grade (Tier 0)
#
# Does NOT require:
#   - live MCP servers
#   - Rust binaries
#   - network access
#
# Requires: jq on PATH

set -euo pipefail

# ── Preconditions ─────────────────────────────────────────────────────────────

if ! command -v jq >/dev/null 2>&1; then
  echo "[FAIL] jq is required but not found on PATH" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WRITE_GOAL="${REPO_ROOT}/skills/learn/learn-goal/scripts/write-goal.sh"
WRITE_SURVEY="${REPO_ROOT}/skills/learn/learn-survey/scripts/write-survey-result.sh"
WRITE_ARTIFACT="${REPO_ROOT}/skills/learn/feynman-loop/scripts/write-artifact.sh"
WRITE_GRADE="${REPO_ROOT}/skills/learn/learn-grade/scripts/write-grade.sh"
FIXTURE_CORPUS="${REPO_ROOT}/tests/learn/fixtures/sample-kb/sample-corpus.json"

for script in "$WRITE_GOAL" "$WRITE_SURVEY" "$WRITE_ARTIFACT" "$WRITE_GRADE"; do
  if [[ ! -x "$script" ]]; then
    echo "[FAIL] script not found or not executable: $script" >&2
    exit 1
  fi
done

if [[ ! -f "$FIXTURE_CORPUS" ]]; then
  echo "[FAIL] fixture corpus not found: $FIXTURE_CORPUS" >&2
  exit 1
fi

# ── Setup ─────────────────────────────────────────────────────────────────────

TEST_DIR="$(mktemp -d)"
trap 'rm -rf "$TEST_DIR"' EXIT

TEST_GOAL_ID="test-linear-algebra-$(date +%s)"

# ── Step 1: Write goal ────────────────────────────────────────────────────────

CORPUS_PATH="${TEST_DIR}/corpus.json"
cp "$FIXTURE_CORPUS" "$CORPUS_PATH"

GOAL_JSON=$(jq -n \
  --arg goal_id "$TEST_GOAL_ID" \
  --arg corpus_path "$CORPUS_PATH" \
  '{
    goal_id: $goal_id,
    subject: "linear algebra basics",
    target_level: "novice",
    weekly_hours: 5,
    total_weeks: 4,
    feasibility: "GREEN",
    feasibility_note: "Well within typical time-to-mastery for novice level.",
    corpus_path: $corpus_path,
    kb_id: null,
    created_at: "2026-06-28T00:00:00Z"
  }')

WRITE_GOAL_OUT=$("$WRITE_GOAL" --goal-json "$GOAL_JSON")
if [[ $? -ne 0 ]]; then
  echo "[FAIL] write-goal.sh exited non-zero" >&2
  exit 1
fi

GOAL_OK=$(echo "$WRITE_GOAL_OUT" | jq -r '.ok')
if [[ "$GOAL_OK" != "true" ]]; then
  echo "[FAIL] write-goal.sh returned ok=false: $WRITE_GOAL_OUT" >&2
  exit 1
fi

# ── Step 2: Verify goal.json on disk ─────────────────────────────────────────

GOAL_PATH="${HOME}/.prometheus/learn/goals/${TEST_GOAL_ID}/goal.json"

if [[ ! -f "$GOAL_PATH" ]]; then
  echo "[FAIL] goal.json not found at expected path: $GOAL_PATH" >&2
  exit 1
fi

FEASIBILITY=$(jq -r '.feasibility' "$GOAL_PATH")
if [[ "$FEASIBILITY" != "GREEN" ]]; then
  echo "[FAIL] goal.json feasibility expected GREEN, got: $FEASIBILITY" >&2
  exit 1
fi

# ── Step 3: Write survey result ───────────────────────────────────────────────

SURVEY_JSON=$(jq -n --arg goal_id "$TEST_GOAL_ID" '{
  schema_version: "1.0.0",
  learner_id: "did:plc:test-learner",
  subject: "linear algebra basics",
  surveyed_at: "2026-06-28T00:00:00Z",
  mastery_priors: [
    {
      concept_id: "vectors",
      estimated_mastery_prior: 0.3,
      confidence: 0.7,
      basis: "survey_response"
    },
    {
      concept_id: "matrices",
      estimated_mastery_prior: 0.2,
      confidence: 0.6,
      basis: "survey_response"
    },
    {
      concept_id: "linear-independence",
      estimated_mastery_prior: 0.1,
      confidence: 0.5,
      basis: "default_prior"
    }
  ],
  recursion_floor: [],
  misconceptions_detected: []
}')

WRITE_SURVEY_OUT=$("$WRITE_SURVEY" --goal-id "$TEST_GOAL_ID" --result-json "$SURVEY_JSON")
if [[ $? -ne 0 ]]; then
  echo "[FAIL] write-survey-result.sh exited non-zero" >&2
  exit 1
fi

SURVEY_OK=$(echo "$WRITE_SURVEY_OUT" | jq -r '.ok')
if [[ "$SURVEY_OK" != "true" ]]; then
  echo "[FAIL] write-survey-result.sh returned ok=false: $WRITE_SURVEY_OUT" >&2
  exit 1
fi

# ── Step 4: Verify survey-result.json on disk ─────────────────────────────────

SURVEY_PATH="${HOME}/.prometheus/learn/goals/${TEST_GOAL_ID}/survey-result.json"

if [[ ! -f "$SURVEY_PATH" ]]; then
  echo "[FAIL] survey-result.json not found at: $SURVEY_PATH" >&2
  exit 1
fi

HAS_MASTERY_PRIORS=$(jq 'has("mastery_priors")' "$SURVEY_PATH")
if [[ "$HAS_MASTERY_PRIORS" != "true" ]]; then
  echo "[FAIL] survey-result.json missing mastery_priors key" >&2
  exit 1
fi

# ── Step 5: Validate corpus JSON structure ────────────────────────────────────

CORPUS_ID=$(jq -r '.corpus_id // empty' "$CORPUS_PATH")
if [[ -z "$CORPUS_ID" ]]; then
  echo "[FAIL] corpus.json missing corpus_id" >&2
  exit 1
fi

CORPUS_SUBJECT=$(jq -r '.subject // empty' "$CORPUS_PATH")
if [[ -z "$CORPUS_SUBJECT" ]]; then
  echo "[FAIL] corpus.json missing subject" >&2
  exit 1
fi

SOURCES_LEN=$(jq '.sources | length' "$CORPUS_PATH")
if [[ "$SOURCES_LEN" -lt 1 ]]; then
  echo "[FAIL] corpus.json sources array is empty" >&2
  exit 1
fi

# ── Step 6: Write feynman artifact ───────────────────────────────────────────

ARTIFACT_JSON=$(jq -n --arg goal_id "$TEST_GOAL_ID" '{
  artifact_id: "feynman-vectors-001",
  goal_id: $goal_id,
  concept_id: "vectors",
  iteration: 1,
  explanation_text: "A vector is an ordered list of numbers that has both magnitude and direction.",
  gaps_identified: [],
  overall_score: 0.75,
  passed: true,
  created_at: "2026-06-28T00:00:00Z"
}')

WRITE_ARTIFACT_OUT=$("$WRITE_ARTIFACT" --goal-id "$TEST_GOAL_ID" --artifact-json "$ARTIFACT_JSON")
if [[ $? -ne 0 ]]; then
  echo "[FAIL] write-artifact.sh exited non-zero" >&2
  exit 1
fi

ARTIFACT_OK=$(echo "$WRITE_ARTIFACT_OUT" | jq -r '.ok')
if [[ "$ARTIFACT_OK" != "true" ]]; then
  echo "[FAIL] write-artifact.sh returned ok=false: $WRITE_ARTIFACT_OUT" >&2
  exit 1
fi

# ── Step 7: Verify artifact file exists with overall_score ────────────────────

ARTIFACT_PATH="${HOME}/.prometheus/learn/goals/${TEST_GOAL_ID}/artifacts/feynman-vectors-001.json"

if [[ ! -f "$ARTIFACT_PATH" ]]; then
  echo "[FAIL] artifact file not found at: $ARTIFACT_PATH" >&2
  exit 1
fi

HAS_OVERALL_SCORE=$(jq 'has("overall_score")' "$ARTIFACT_PATH")
if [[ "$HAS_OVERALL_SCORE" != "true" ]]; then
  echo "[FAIL] artifact file missing overall_score field" >&2
  exit 1
fi

# ── Step 8: Write grade result ────────────────────────────────────────────────

GRADE_JSON=$(jq -n --arg goal_id "$TEST_GOAL_ID" '{
  grade_id: "grade-vectors-001",
  goal_id: $goal_id,
  concept_id: "vectors",
  artifact_id: "feynman-vectors-001",
  score: 0.75,
  passed: true,
  gaps: [],
  sycophancy_check: "PASS",
  graded_at: "2026-06-28T00:00:00Z"
}')

WRITE_GRADE_OUT=$("$WRITE_GRADE" --goal-id "$TEST_GOAL_ID" --grade-json "$GRADE_JSON")
if [[ $? -ne 0 ]]; then
  echo "[FAIL] write-grade.sh exited non-zero" >&2
  exit 1
fi

GRADE_OK=$(echo "$WRITE_GRADE_OUT" | jq -r '.ok')
if [[ "$GRADE_OK" != "true" ]]; then
  echo "[FAIL] write-grade.sh returned ok=false: $WRITE_GRADE_OUT" >&2
  exit 1
fi

# ── Step 9: Verify grade file on disk ────────────────────────────────────────

GRADE_PATH="${HOME}/.prometheus/learn/goals/${TEST_GOAL_ID}/grades/grade-vectors-001.json"

if [[ ! -f "$GRADE_PATH" ]]; then
  echo "[FAIL] grade file not found at: $GRADE_PATH" >&2
  exit 1
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo "[PASS] basic flow integration test"
exit 0
