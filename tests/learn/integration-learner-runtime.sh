#!/usr/bin/env bash
# Real learner-model JSON-RPC + FSRS integration test.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="${LEARNER_MODEL_BIN:-$REPO_ROOT/substrate/learner-model/target/debug/learner-model}"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

if [[ ! -x "$BIN" ]]; then
  cargo build --manifest-path "$REPO_ROOT/substrate/learner-model/Cargo.toml" --bin learner-model
fi

rpc() {
  local request="$1"
  printf '%s\n' "$request" | "$BIN" "$TMP_ROOT/model"
}

SEED=$(jq -nc '{
  method:"seed_from_survey",
  params:{seed:{
    schema_version:"1.0.0",
    learner_id:"goal-runtime-test",
    subject:"Runtime verification",
    surveyed_at:"2026-07-18T12:00:00Z",
    mastery_priors:[{
      concept_id:"feedback-loops",
      estimated_mastery_prior:0.4,
      confidence:0.8,
      basis:"diagnostic_item"
    }],
    recursion_floor:[],
    misconceptions_detected:[]
  }}
}')
[[ "$(rpc "$SEED" | jq -r '.ok')" == "true" ]]

GET=$(jq -nc '{method:"get_concept",params:{learner_id:"goal-runtime-test",concept_id:"feedback-loops"}}')
[[ "$(rpc "$GET" | jq -r '.concept_id')" == "feedback-loops" ]]

OBSERVE=$(jq -nc '{method:"add_observation",params:{learner_id:"goal-runtime-test",concept_id:"feedback-loops",score:0.82,source_skill:"learn-grade"}}')
[[ "$(rpc "$OBSERVE" | jq -r '.ok')" == "true" ]]

REVIEW=$(jq -nc '{method:"review",params:{learner_id:"goal-runtime-test",concept_id:"feedback-loops",score:0.9,rating:"easy",timestamp:"2026-07-18T13:00:00Z",source_skill:"learn-retain"}}')
REVIEW_RESULT=$(rpc "$REVIEW")
[[ "$(jq -r '.ok' <<< "$REVIEW_RESULT")" == "true" ]]
[[ "$(jq -r '.fsrs_card.reps' <<< "$REVIEW_RESULT")" == "1" ]]
[[ "$(jq -r '.fsrs_card.last_review' <<< "$REVIEW_RESULT")" == "2026-07-18T13:00:00Z" ]]
[[ "$(jq -r '.fsrs_card.due' <<< "$REVIEW_RESULT")" > "2026-07-18T13:00:00Z" ]]

LOAD=$(jq -nc '{method:"load",params:{learner_id:"goal-runtime-test"}}')
MODEL=$(rpc "$LOAD")
[[ "$(jq -r '.concepts["feedback-loops"].observations | length' <<< "$MODEL")" == "2" ]]
[[ "$(jq -r '.concepts["feedback-loops"].fsrs_card.reps' <<< "$MODEL")" == "1" ]]

MISSING=$(jq -nc '{method:"add_observation",params:{learner_id:"goal-runtime-test",concept_id:"absent",score:1,source_skill:"learn-grade"}}')
rpc "$MISSING" | jq -e '.error | contains("Concept not found")' >/dev/null

echo "[PASS] learner-model JSON-RPC seed/get/observe/review/load runtime"
