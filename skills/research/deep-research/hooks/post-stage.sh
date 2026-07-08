#!/usr/bin/env bash
set -euo pipefail

# Post-stage completion hook — runs after each pipeline stage completes.
# Logs stage completion with timing and writes a checkpoint.

STAGE="${RESEARCH_CURRENT_STAGE:-unknown}"
JOB_ID="${RESEARCH_JOB_ID:-unknown}"
STAGE_START="${RESEARCH_STAGE_START_TS:-}"
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.research-jobs}"

NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Compute elapsed time if start timestamp is available
ELAPSED=""
if [[ -n "$STAGE_START" ]]; then
  START_SEC=$(date -u -d "$STAGE_START" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$STAGE_START" +%s 2>/dev/null || echo "0")
  NOW_SEC=$(date -u +%s)
  ELAPSED_SEC=$(( NOW_SEC - START_SEC ))
  ELAPSED=" (${ELAPSED_SEC}s)"
fi

echo "[post-stage] Completed stage $STAGE for job $JOB_ID at $NOW$ELAPSED"

# Write a checkpoint file for resumability
CHECKPOINT_FILE="$OUTPUT_DIR/$JOB_ID/checkpoint.json"
if command -v jq &>/dev/null && [[ -f "$CHECKPOINT_FILE" ]]; then
  # Update existing checkpoint
  TMP=$(mktemp)
  jq --arg stage "$STAGE" --arg ts "$NOW" \
    '.last_completed_stage = $stage | .last_completed_at = $ts' \
    "$CHECKPOINT_FILE" > "$TMP" && mv "$TMP" "$CHECKPOINT_FILE"
else
  # Create checkpoint
  cat > "$CHECKPOINT_FILE" <<EOF
{
  "job_id": "$JOB_ID",
  "last_completed_stage": "$STAGE",
  "last_completed_at": "$NOW"
}
EOF
fi

echo "[post-stage] Checkpoint updated: $STAGE"
