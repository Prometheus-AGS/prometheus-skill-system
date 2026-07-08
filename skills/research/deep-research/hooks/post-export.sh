#!/usr/bin/env bash
set -euo pipefail

# Post-export hook — fires after Stage 10 assembles the .research package.
# Logs the export path and optionally ingests into the palace.

JOB_ID="${RESEARCH_JOB_ID:-unknown}"
PACKAGE_PATH="${RESEARCH_PACKAGE_PATH:-}"
INGEST_PALACE="${RESEARCH_INGEST_PALACE:-0}"
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.research-jobs}"

NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)

echo "[post-export] Research package exported for job $JOB_ID at $NOW"

if [[ -n "$PACKAGE_PATH" ]]; then
  echo "[post-export] Package path: $PACKAGE_PATH"
  if [[ -d "$PACKAGE_PATH" ]]; then
    SIZE=$(du -sh "$PACKAGE_PATH" 2>/dev/null | cut -f1 || echo "unknown")
    echo "[post-export] Package size: $SIZE"
  fi
else
  echo "[post-export] WARN: RESEARCH_PACKAGE_PATH not set"
fi

# Append to export history
EXPORT_LOG="$OUTPUT_DIR/export-history.log"
echo "$NOW | $JOB_ID | $PACKAGE_PATH" >> "$EXPORT_LOG"

# Optionally ingest report into palace (palace_ingest is an MCP tool — cannot call directly from bash)
# Instead, emit a marker file that run-research.sh or the skill can detect
if [[ "$INGEST_PALACE" == "1" ]]; then
  REPORT_FILE="$OUTPUT_DIR/$JOB_ID/report.md"
  MANIFEST_FILE="$OUTPUT_DIR/$JOB_ID/manifest.json"
  if [[ -f "$REPORT_FILE" ]]; then
    INGEST_MARKER="$OUTPUT_DIR/$JOB_ID/.palace-ingest-requested"
    echo "$NOW" > "$INGEST_MARKER"
    echo "[post-export] Palace ingest requested. Marker written: $INGEST_MARKER"
    echo "[post-export] The skill will call palace_ingest with report.md and manifest.json"
  else
    echo "[post-export] WARN: report.md not found, cannot request palace ingest"
  fi
fi

echo "[post-export] Export hook complete."
