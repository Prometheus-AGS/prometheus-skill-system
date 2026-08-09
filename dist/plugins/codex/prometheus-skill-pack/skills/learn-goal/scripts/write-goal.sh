#!/usr/bin/env bash
# write-goal.sh — Writes a learn-goal artifact to ~/.prometheus/learn/goals/<goal_id>/goal.json
#
# Usage:
#   write-goal.sh --goal-json '<json>'
#
# Output (stdout):
#   {"ok":true,"path":"<absolute path to goal.json>"}
#
# Exit codes:
#   0 — success
#   1 — fatal error (bad args, missing goal_id, unwritable directory, etc.)

set -euo pipefail

log_error() { echo "[write-goal] ERROR: $*" >&2; }

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

GOAL_JSON=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --goal-json) GOAL_JSON="$2"; shift 2 ;;
    --) shift; break ;;
    *) log_error "Unknown argument: $1"; exit 1 ;;
  esac
done

if [[ -z "$GOAL_JSON" ]]; then
  log_error "--goal-json is required"
  exit 1
fi

# ---------------------------------------------------------------------------
# Validate JSON and extract goal_id
# ---------------------------------------------------------------------------

if ! command -v jq >/dev/null 2>&1; then
  log_error "jq is required but not installed"
  exit 1
fi

if ! echo "$GOAL_JSON" | jq empty 2>/dev/null; then
  log_error "--goal-json is not valid JSON"
  exit 1
fi

GOAL_ID=$(echo "$GOAL_JSON" | jq -r '.goal_id // empty')

if [[ -z "$GOAL_ID" ]]; then
  log_error "goal_id field is missing or empty in provided JSON"
  exit 1
fi

# ---------------------------------------------------------------------------
# Create directory and write file
# ---------------------------------------------------------------------------

GOAL_DIR="${HOME}/.prometheus/learn/goals/${GOAL_ID}"

if ! mkdir -p "$GOAL_DIR" 2>/dev/null; then
  log_error "Cannot create goal directory: $GOAL_DIR"
  exit 1
fi

GOAL_PATH="${GOAL_DIR}/goal.json"

if ! echo "$GOAL_JSON" | jq '.' > "$GOAL_PATH" 2>/dev/null; then
  log_error "Failed to write goal.json to: $GOAL_PATH"
  exit 1
fi

# ---------------------------------------------------------------------------
# Confirm
# ---------------------------------------------------------------------------

echo "{\"ok\":true,\"path\":\"${GOAL_PATH}\"}"
exit 0
