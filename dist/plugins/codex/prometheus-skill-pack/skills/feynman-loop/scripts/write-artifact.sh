#!/usr/bin/env bash
# write-artifact.sh — Write a feynman-loop artifact JSON to the goals artifact store.
#
# Usage:
#   write-artifact.sh --goal-id <id> --artifact-json '<json>'
#
# Creates (change-rah-008, the one artifact path shared by feynman-loop,
# learn-retain, and learn-certify):
#   ${PROMETHEUS_LEARN_HOME:-~/.prometheus/learn}/goals/<goal-id>/artifacts/<concept-id>/<artifact-id>.json
#
# learn-retain globs artifacts/<concept-id>/*.json for the most recent artifact;
# learn-certify reads the artifacts/<concept-id>/ directory. Both resolve the
# file this script writes, so concept_id is required, not informational.
#
# Exits 0 and prints {"ok":true,"path":"...","artifact_id":"...","concept_id":"..."} on success.
# Exits 1 and prints {"ok":false,"error":"..."} on failure.

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────
GOAL_ID=""
ARTIFACT_JSON=""

while [ $# -gt 0 ]; do
  case "$1" in
    --goal-id)
      GOAL_ID="$2"
      shift 2
      ;;
    --artifact-json)
      ARTIFACT_JSON="$2"
      shift 2
      ;;
    *)
      echo "{\"ok\":false,\"error\":\"Unknown argument: $1\"}" >&2
      exit 1
      ;;
  esac
done

# ── Validation ────────────────────────────────────────────────────────────────
if [ -z "$GOAL_ID" ]; then
  echo '{"ok":false,"error":"--goal-id is required"}' >&2
  exit 1
fi

if [ -z "$ARTIFACT_JSON" ]; then
  echo '{"ok":false,"error":"--artifact-json is required"}' >&2
  exit 1
fi

# Validate that artifact_json is parseable JSON
if ! echo "$ARTIFACT_JSON" | jq empty 2>/dev/null; then
  echo '{"ok":false,"error":"--artifact-json is not valid JSON"}' >&2
  exit 1
fi

# ── Extract artifact_id ───────────────────────────────────────────────────────
ARTIFACT_ID=$(echo "$ARTIFACT_JSON" | jq -r '.artifact_id // empty')

if [ -z "$ARTIFACT_ID" ]; then
  echo '{"ok":false,"error":"artifact_json is missing artifact_id field"}' >&2
  exit 1
fi

# ── Verification and provenance (change-rah-005) ─────────────────────────────
# Every transfer score carries a verification label with evidence, and the
# artifact names the grade file and corpus it came from. An artifact without
# them is refused: a mastery record that cannot say how it was checked is not a
# record. Labels follow the four-value vocabulary shared with deep-research
# (verified | unverified | blocked | inferred).
VALIDATION_ERROR=$(echo "$ARTIFACT_JSON" | jq -r '
  def labels: ["verified","unverified","blocked","inferred"];
  if (.transfer_scores | type) != "array" then "transfer_scores must be an array"
  elif (.verification | type) != "array" then "verification must be an array of {label, evidence}, one per transfer score"
  elif (.verification | length) != (.transfer_scores | length) then "verification must have one entry per transfer score (\(.transfer_scores|length) scores, \(.verification|length) entries)"
  elif ([.verification[] | select(type != "object")] | length) > 0 then "every verification entry must be an object {label, evidence}"
  elif ([.verification[] | select((.label|type) != "string" or ((.label) as $l | labels | index($l) | not))] | length) > 0 then "every verification entry needs a label in verified|unverified|blocked|inferred"
  elif ([.verification[] | select((.evidence|type) != "string" or (.evidence|length) == 0)] | length) > 0 then "every verification entry needs non-empty evidence (grade file, transfer problem index, corpus_ref)"
  elif (.provenance | type) != "object" then "provenance must be an object"
  elif ((.provenance.grade_file // "") | length) == 0 then "provenance.grade_file is required"
  elif ((.provenance.corpus_path // "") | length) == 0 then "provenance.corpus_path is required"
  else "" end')
if [ -n "$VALIDATION_ERROR" ]; then
  jq -n --arg e "$VALIDATION_ERROR" '{"ok":false,"error":$e}' >&2
  exit 1
fi

# ── Extract concept_id (path component) ──────────────────────────────────────
CONCEPT_ID=$(echo "$ARTIFACT_JSON" | jq -r '.concept_id // empty')

if [ -z "$CONCEPT_ID" ]; then
  echo '{"ok":false,"error":"artifact_json is missing concept_id field (it names the artifact directory)"}' >&2
  exit 1
fi

# Ids become path components: refuse separators and dot-segments so a crafted
# goal_id, concept_id, or artifact_id cannot escape the learn home.
for _component in "$GOAL_ID" "$CONCEPT_ID" "$ARTIFACT_ID"; do
  case "$_component" in
    .|..|*/*|*\\*)
      echo '{"ok":false,"error":"goal_id, concept_id, and artifact_id must be single path components (no /, \\, ., or ..)"}' >&2
      exit 1
      ;;
  esac
done

# ── Ensure the concept's artifact directory exists ───────────────────────────
LEARN_HOME="${PROMETHEUS_LEARN_HOME:-${HOME}/.prometheus/learn}"
ARTIFACTS_DIR="${LEARN_HOME}/goals/${GOAL_ID}/artifacts/${CONCEPT_ID}"
mkdir -p "$ARTIFACTS_DIR"

# ── Write artifact file ───────────────────────────────────────────────────────
ARTIFACT_PATH="${ARTIFACTS_DIR}/${ARTIFACT_ID}.json"
echo "$ARTIFACT_JSON" | jq '.' > "$ARTIFACT_PATH"

# ── Emit result ───────────────────────────────────────────────────────────────
jq -n \
  --arg path "$ARTIFACT_PATH" \
  --arg artifact_id "$ARTIFACT_ID" \
  --arg concept_id "$CONCEPT_ID" \
  '{"ok":true,"path":$path,"artifact_id":$artifact_id,"concept_id":$concept_id}'
