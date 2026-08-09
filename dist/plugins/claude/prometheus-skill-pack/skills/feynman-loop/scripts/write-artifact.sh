#!/usr/bin/env bash
# write-artifact.sh — Write a feynman-loop artifact JSON to the goals artifact store.
#
# Usage:
#   write-artifact.sh --goal-id <id> --artifact-json '<json>'
#
# Creates:
#   ~/.prometheus/learn/goals/<goal-id>/artifacts/<artifact-id>.json
#
# Exits 0 and prints {"ok":true,"path":"..."} on success.
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

# ── Extract concept_id (informational) ───────────────────────────────────────
CONCEPT_ID=$(echo "$ARTIFACT_JSON" | jq -r '.concept_id // "unknown"')

# ── Ensure artifacts directory exists ────────────────────────────────────────
ARTIFACTS_DIR="${HOME}/.prometheus/learn/goals/${GOAL_ID}/artifacts"
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
