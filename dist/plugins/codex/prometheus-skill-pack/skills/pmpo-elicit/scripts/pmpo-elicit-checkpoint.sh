#!/usr/bin/env bash
# pmpo-elicit-checkpoint.sh — write an async elicitation checkpoint and exit BLOCKED (2)
#
# Usage:
#   pmpo-elicit-checkpoint.sh <elicit-dir> <question> <criticality> <caller> [hint1] [hint2] ...
#
# Exit codes:
#   0 — unexpected (internal only)
#   1 — error (bad args, write failure)
#   2 — BLOCKED: checkpoint written, awaiting result.json from operator

set -euo pipefail

if [[ $# -lt 4 ]]; then
  echo "[pmpo-elicit-checkpoint] ERROR: usage: $0 <elicit-dir> <question> <criticality> <caller> [hints...]" >&2
  exit 1
fi

ELICIT_DIR="$1"
QUESTION="$2"
CRITICALITY="$3"
CALLER="$4"
shift 4
HINTS=("$@")

CRITICALITY_VALUES="low medium high blocking"
if ! echo "$CRITICALITY_VALUES" | grep -qw "$CRITICALITY"; then
  echo "[pmpo-elicit-checkpoint] ERROR: criticality must be one of: $CRITICALITY_VALUES" >&2
  exit 1
fi

TIMESTAMP="$(date +%s)"
ISO_TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
ID="${CALLER}-${TIMESTAMP}"

mkdir -p "$ELICIT_DIR"

# Build hints JSON array
HINTS_JSON="[]"
if [[ ${#HINTS[@]} -gt 0 ]]; then
  HINTS_JSON="["
  for i in "${!HINTS[@]}"; do
    if [[ $i -gt 0 ]]; then HINTS_JSON="${HINTS_JSON},"; fi
    # Escape double quotes in hint
    HINT_ESCAPED="${HINTS[$i]//\"/\\\"}"
    HINTS_JSON="${HINTS_JSON}\"${HINT_ESCAPED}\""
  done
  HINTS_JSON="${HINTS_JSON}]"
fi

QUESTION_ESCAPED="${QUESTION//\"/\\\"}"
CALLER_ESCAPED="${CALLER//\"/\\\"}"

# Write request.json
cat > "${ELICIT_DIR}/request.json" <<EOF
{
  "kind": "request",
  "id": "${ID}",
  "question": "${QUESTION_ESCAPED}",
  "context": "",
  "hints": ${HINTS_JSON},
  "criticality": "${CRITICALITY}",
  "caller": "${CALLER_ESCAPED}",
  "write_back_path": ""
}
EOF

# Write checkpoint.json
cat > "${ELICIT_DIR}/checkpoint.json" <<EOF
{
  "id": "${ID}",
  "caller": "${CALLER_ESCAPED}",
  "timestamp": "${ISO_TIMESTAMP}",
  "status": "pending"
}
EOF

# Write human-readable prompt
HINTS_TEXT=""
if [[ ${#HINTS[@]} -gt 0 ]]; then
  HINTS_TEXT="Hints: $(IFS=', '; echo "${HINTS[*]}")"$'\n'
fi

cat > "${ELICIT_DIR}/request-prompt.txt" <<EOF
[pmpo-elicit] Question from ${CALLER}
ID: ${ID}
Criticality: ${CRITICALITY}

${QUESTION}

${HINTS_TEXT}
To respond, write result.json in this directory with:
{
  "kind": "result",
  "id": "${ID}",
  "answer": "<your answer here>",
  "provenance": "user"
}

Optional fields: source_ref, confidence (0-1), evidence ([{source_url, claim}])
EOF

echo "[pmpo-elicit-checkpoint] Checkpoint written: ${ELICIT_DIR}" >&2
echo "[pmpo-elicit-checkpoint] ID: ${ID}" >&2
echo "[pmpo-elicit-checkpoint] Awaiting result.json — loop is BLOCKED" >&2

exit 2
