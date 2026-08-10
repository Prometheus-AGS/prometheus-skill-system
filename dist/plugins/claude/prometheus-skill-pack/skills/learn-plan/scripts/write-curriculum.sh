#!/usr/bin/env bash
# write-curriculum.sh
# Persist a curriculum.json for a given goal-id.
#
# Usage:
#   write-curriculum.sh --goal-id <id> --curriculum-json '<json>'
#
# Exits 0 and prints {"ok":true,"path":"..."} on success.
# Exits 1 and prints {"ok":false,"error":"..."} on failure.

set -euo pipefail

GOAL_ID=""
CURRICULUM_JSON=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --goal-id)
      GOAL_ID="$2"
      shift 2
      ;;
    --curriculum-json)
      CURRICULUM_JSON="$2"
      shift 2
      ;;
    *)
      printf '{"ok":false,"error":"Unknown argument: %s"}\n' "$1" >&2
      exit 1
      ;;
  esac
done

# Validate required arguments
if [[ -z "$GOAL_ID" ]]; then
  printf '{"ok":false,"error":"--goal-id is required"}\n' >&2
  exit 1
fi

if [[ -z "$CURRICULUM_JSON" ]]; then
  printf '{"ok":false,"error":"--curriculum-json is required"}\n' >&2
  exit 1
fi

# Validate JSON (requires jq)
if ! command -v jq &>/dev/null; then
  printf '{"ok":false,"error":"jq is required but not found on PATH"}\n' >&2
  exit 1
fi

if ! echo "$CURRICULUM_JSON" | jq empty 2>/dev/null; then
  printf '{"ok":false,"error":"--curriculum-json is not valid JSON"}\n' >&2
  exit 1
fi

# Create goal directory if needed
GOAL_DIR="${HOME}/.prometheus/learn/goals/${GOAL_ID}"
mkdir -p "$GOAL_DIR"

OUTPUT_PATH="${GOAL_DIR}/curriculum.json"

# Write the curriculum
if ! echo "$CURRICULUM_JSON" | jq . > "$OUTPUT_PATH"; then
  printf '{"ok":false,"error":"Failed to write curriculum to %s"}\n' "$OUTPUT_PATH" >&2
  exit 1
fi

printf '{"ok":true,"path":"%s"}\n' "$OUTPUT_PATH"
