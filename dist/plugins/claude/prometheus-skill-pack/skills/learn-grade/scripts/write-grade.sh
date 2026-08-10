#!/usr/bin/env bash
# write-grade.sh — Write a learn-grade result JSON to the goals grade store.
#
# Usage:
#   write-grade.sh --goal-id <id> --grade-json '<json>'
#
# Creates:
#   ~/.prometheus/learn/goals/<goal-id>/grades/<grade-id>.json
#
# Exits 0 and prints {"ok":true,"path":"..."} on success.
# Exits 1 and prints {"ok":false,"error":"..."} on failure.

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────
GOAL_ID=""
GRADE_JSON=""

while [ $# -gt 0 ]; do
  case "$1" in
    --goal-id)
      GOAL_ID="$2"
      shift 2
      ;;
    --grade-json)
      GRADE_JSON="$2"
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

if [ -z "$GRADE_JSON" ]; then
  echo '{"ok":false,"error":"--grade-json is required"}' >&2
  exit 1
fi

# Validate that grade_json is parseable JSON
if ! echo "$GRADE_JSON" | jq empty 2>/dev/null; then
  echo '{"ok":false,"error":"--grade-json is not valid JSON"}' >&2
  exit 1
fi

# ── Extract grade_id ──────────────────────────────────────────────────────────
GRADE_ID=$(echo "$GRADE_JSON" | jq -r '.grade_id // empty')

if [ -z "$GRADE_ID" ]; then
  echo '{"ok":false,"error":"grade_json is missing grade_id field"}' >&2
  exit 1
fi

# ── Ensure grades directory exists ────────────────────────────────────────────
GRADES_DIR="${HOME}/.prometheus/learn/goals/${GOAL_ID}/grades"
mkdir -p "$GRADES_DIR"

# ── Write grade file ──────────────────────────────────────────────────────────
GRADE_PATH="${GRADES_DIR}/${GRADE_ID}.json"
echo "$GRADE_JSON" | jq '.' > "$GRADE_PATH"

# ── Emit result ───────────────────────────────────────────────────────────────
jq -n --arg path "$GRADE_PATH" '{"ok":true,"path":$path}'
