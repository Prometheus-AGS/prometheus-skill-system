#!/usr/bin/env bash
set -euo pipefail

# Post-stage completion hook — runs after each pipeline stage completes.
# Logs stage completion with timing and writes a checkpoint.

STAGE="${RESEARCH_CURRENT_STAGE:-unknown}"
JOB_ID="${RESEARCH_JOB_ID:-unknown}"
STAGE_START="${RESEARCH_STAGE_START_TS:-}"
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.prometheus/research}"
# Package directory: the driver passes RESEARCH_PACKAGE_PATH; older callers pass
# only ids, so fall back to <root>/<package_id> then <root>/<job_id>.
PKG_DIR="${RESEARCH_PACKAGE_PATH:-$OUTPUT_DIR/${RESEARCH_PACKAGE_ID:-$JOB_ID}}"

NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Elapsed time since the stage started (GNU date first, then BSD date in UTC).
ELAPSED=""
if [[ -n "$STAGE_START" ]]; then
  START_SEC=$(date -u -d "$STAGE_START" +%s 2>/dev/null || TZ=UTC date -j -u -f "%Y-%m-%dT%H:%M:%SZ" "$STAGE_START" +%s 2>/dev/null || echo "")
  if [[ -n "$START_SEC" ]]; then
    ELAPSED=" ($(( $(date -u +%s) - START_SEC ))s)"
  fi
fi

echo "[post-stage] Completed stage $STAGE for job $JOB_ID at $NOW$ELAPSED"

# Checkpoint. When the driver owns checkpoint.json (it carries stages_completed),
# only annotate it; never replace it. Without a driver checkpoint, keep the small
# legacy record so older callers still get resumability data.
mkdir -p "$PKG_DIR"
CHECKPOINT_FILE="$PKG_DIR/checkpoint.json"
if [[ -f "$CHECKPOINT_FILE" ]]; then
  if command -v jq &>/dev/null; then
    TMP=$(mktemp)
    jq --arg stage "$STAGE" --arg ts "$NOW" \
      '.last_completed_stage = $stage | .last_completed_at = $ts' \
      "$CHECKPOINT_FILE" > "$TMP" && mv "$TMP" "$CHECKPOINT_FILE"
  else
    echo "[post-stage] NOTE: jq not found; existing checkpoint.json left untouched (not annotated)" >&2
  fi
else
  cat > "$CHECKPOINT_FILE" <<EOF
{
  "job_id": "$JOB_ID",
  "last_completed_stage": "$STAGE",
  "last_completed_at": "$NOW"
}
EOF
fi

echo "[post-stage] Checkpoint updated: $STAGE"
