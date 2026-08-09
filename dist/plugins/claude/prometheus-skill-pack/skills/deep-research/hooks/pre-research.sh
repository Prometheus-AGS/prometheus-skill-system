#!/usr/bin/env bash
set -euo pipefail

# Pre-research validation hook — runs before the pipeline starts.
# Validates required inputs and tool availability.

QUERY="${RESEARCH_QUERY:-}"
DEPTH="${RESEARCH_DEPTH:-deep}"
JOB_ID="${RESEARCH_JOB_ID:-}"

echo "[pre-research] Validating research job: ${JOB_ID:-unknown}"

# Validate QUERY
if [[ -z "$QUERY" ]]; then
  echo "[pre-research] ERROR: RESEARCH_QUERY is required but not set" >&2
  exit 1
fi

if [[ ${#QUERY} -lt 5 ]]; then
  echo "[pre-research] ERROR: RESEARCH_QUERY is too short (min 5 chars)" >&2
  exit 1
fi

# Validate DEPTH
case "$DEPTH" in
  shallow|deep|exhaustive) ;;
  *)
    echo "[pre-research] ERROR: RESEARCH_DEPTH must be shallow, deep, or exhaustive (got: $DEPTH)" >&2
    exit 1
    ;;
esac

# Validate JOB_ID
if [[ -z "$JOB_ID" ]]; then
  echo "[pre-research] ERROR: RESEARCH_JOB_ID is required but not set" >&2
  exit 1
fi

# Check output directory
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.research-jobs}"
mkdir -p "$OUTPUT_DIR/$JOB_ID"
echo "[pre-research] Output directory: $OUTPUT_DIR/$JOB_ID"

# Tool availability warnings (non-blocking)
if ! command -v python3 &>/dev/null; then
  echo "[pre-research] WARN: python3 not found — graph and contradiction scripts will fall back to stubs"
fi

if ! command -v jq &>/dev/null; then
  echo "[pre-research] WARN: jq not found — JSON processing will be limited"
fi

echo "[pre-research] Validation passed. Starting pipeline for: $QUERY (depth: $DEPTH)"
