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

# Validate JOB_ID (the run identifier; recorded in the manifest, not the directory name)
if [[ -z "$JOB_ID" ]]; then
  echo "[pre-research] ERROR: RESEARCH_JOB_ID is required but not set" >&2
  exit 1
fi

# The package directory is <slug>-<yyyymmdd>-<4hex> (research-package-spec.md).
# The driver derives it and passes RESEARCH_PACKAGE_ID; when a caller has only a
# job id, the directory falls back to it and the fallback is named so it is visible.
PACKAGE_ID="${RESEARCH_PACKAGE_ID:-}"
SLUG_LIB="$(cd "$(dirname "$0")/../../../.." 2>/dev/null && pwd)/shared/scripts/lib/slug.sh"
if [[ -f "$SLUG_LIB" ]]; then
  # shellcheck source=/dev/null
  . "$SLUG_LIB"
else
  # No library: the package-id rule is still enforced with the same pattern the
  # library uses (documented fallback; keep in step with package_id_is_valid).
  package_id_is_valid() { [[ "$1" =~ ^[a-z0-9]+(-[a-z0-9]+){0,4}-[0-9]{8}-[0-9a-f]{4}$ ]]; }
fi
if [[ -z "$PACKAGE_ID" ]]; then
  # No id from the caller: derive a conforming one from the query with the shared
  # slug library rather than blessing the job id as a directory name.
  if command -v package_id_new >/dev/null 2>&1; then
    PACKAGE_ID="$(package_id_new "$(slug_from_text "$QUERY")")"
    echo "[pre-research] NOTE: RESEARCH_PACKAGE_ID not set; derived $PACKAGE_ID from the query" >&2
  else
    echo "[pre-research] ERROR: RESEARCH_PACKAGE_ID not set and shared/scripts/lib/slug.sh not found; refusing to name the package after the job id" >&2
    exit 1
  fi
elif ! package_id_is_valid "$PACKAGE_ID"; then
  echo "[pre-research] ERROR: RESEARCH_PACKAGE_ID '$PACKAGE_ID' is not <slug>-<yyyymmdd>-<4hex> with a slug of at most five words" >&2
  exit 1
fi

# Check output directory
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.prometheus/research}"
mkdir -p "$OUTPUT_DIR/$PACKAGE_ID"
echo "[pre-research] Package directory: $OUTPUT_DIR/$PACKAGE_ID"

# Tool availability warnings (non-blocking)
if ! command -v python3 &>/dev/null; then
  echo "[pre-research] WARN: python3 not found — graph and contradiction scripts will fall back to stubs"
fi

if ! command -v jq &>/dev/null; then
  echo "[pre-research] WARN: jq not found — JSON processing will be limited"
fi

echo "[pre-research] Validation passed. Starting pipeline for: $QUERY (depth: $DEPTH)"
