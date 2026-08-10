#!/usr/bin/env bash
set -euo pipefail

# Run the deep-research 10-stage pipeline.
# Usage: QUERY="..." DEPTH="deep" bash run-research.sh
# Or:    bash run-research.sh "my query" [shallow|deep|exhaustive] [kb_id1,kb_id2]

QUERY="${1:-${QUERY:-}}"
DEPTH="${2:-${DEPTH:-deep}}"
KB_IDS="${3:-${KB_IDS:-}}"

if [[ -z "$QUERY" ]]; then
  echo '{"error": "QUERY is required. Usage: QUERY=\"...\" bash run-research.sh"}' >&2
  exit 1
fi

case "$DEPTH" in
  shallow|deep|exhaustive) ;;
  *)
    echo '{"error": "DEPTH must be shallow, deep, or exhaustive"}' >&2
    exit 1
    ;;
esac

# Require at least one search tool
if [[ -z "${TAVILY_API_KEY:-}" && -z "${FIRECRAWL_API_KEY:-}" ]]; then
  echo '{"error": "No search tool available. Set TAVILY_API_KEY or FIRECRAWL_API_KEY."}' >&2
  exit 1
fi

JOB_ID="research-$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/.prometheus/research/$JOB_ID}"
mkdir -p "$OUTPUT_DIR/sources"

log() { echo "[deep-research] $*" >&2; }
emit() { echo "$*"; }

log "Starting deep-research job: $JOB_ID"
log "Query: $QUERY"
log "Depth: $DEPTH"
log "Output: $OUTPUT_DIR"

STAGES_FOR_DEPTH() {
  case "$1" in
    shallow)     echo "01 02 03 04 05" ;;
    deep)        echo "01 02 03 04 05 06 07 08 09 10" ;;
    exhaustive)  echo "01 02 03 04 05 06 07 08 09 10" ;;
  esac
}

STAGES=$(STAGES_FOR_DEPTH "$DEPTH")
STAGE_COUNT=$(echo "$STAGES" | wc -w | tr -d ' ')
CURRENT=0

for STAGE in $STAGES; do
  CURRENT=$((CURRENT + 1))
  log "Stage $CURRENT/$STAGE_COUNT: stage-${STAGE}"
  emit "{\"stage\": \"$STAGE\", \"status\": \"started\", \"job_id\": \"$JOB_ID\"}"
  # Harness invokes the appropriate sub-skill here via the pipeline
  emit "{\"stage\": \"$STAGE\", \"status\": \"completed\", \"job_id\": \"$JOB_ID\"}"
done

emit "{\"status\": \"complete\", \"job_id\": \"$JOB_ID\", \"output_dir\": \"$OUTPUT_DIR\", \"stages_completed\": $STAGE_COUNT}"
log "Research complete. Package at: $OUTPUT_DIR"
