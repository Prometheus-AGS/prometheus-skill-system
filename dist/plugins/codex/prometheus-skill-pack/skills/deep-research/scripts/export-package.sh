#!/usr/bin/env bash
set -euo pipefail

# Assemble the .research package from stage outputs.
# Usage: JOB_ID="..." OUTPUT_DIR="..." bash export-package.sh

JOB_ID="${1:-${JOB_ID:-}}"
OUTPUT_DIR="${2:-${OUTPUT_DIR:-$HOME/.prometheus/research/${JOB_ID}}}"

if [[ -z "$JOB_ID" ]]; then
  echo '{"error": "JOB_ID is required"}' >&2
  exit 1
fi

log() { echo "[export-package] $*" >&2; }

log "Assembling .research package for job: $JOB_ID"
log "Output directory: $OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR/sources"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Write index.md if report exists
if [[ -f "$OUTPUT_DIR/report.md" ]]; then
  TITLE=$(grep '^title:' "$OUTPUT_DIR/report.md" | head -1 | sed 's/title: //' | tr -d '"')
  CONF=$(grep '^confidence:' "$OUTPUT_DIR/report.md" | head -1 | sed 's/confidence: //')
else
  TITLE="Research Package: $JOB_ID"
  CONF="null"
fi

cat > "$OUTPUT_DIR/index.md" <<INDEX
# Research Package: $JOB_ID

**Created:** $TIMESTAMP
**Confidence:** $CONF

## Contents

- [Report](report.md) — Full research synthesis
- [Graph](graph.json) — Knowledge graph
- [Citations](citations.json) — Source bibliography
- [Contradictions](contradictions.json) — Contradiction log
- [Sources](sources/) — Raw retrieved content
- [Manifest](manifest.json) — Package metadata

## Quick Access

See [report.md](report.md) for findings.
INDEX

# Write manifest.json
SOURCES_COUNT=$(ls "$OUTPUT_DIR/sources/"*.json 2>/dev/null | wc -l | tr -d ' ')

cat > "$OUTPUT_DIR/manifest.json" <<MANIFEST
{
  "version": "1.0.0",
  "okf_version": "0.1",
  "job_id": "$JOB_ID",
  "created_at": "$TIMESTAMP",
  "sources_count": $SOURCES_COUNT,
  "confidence": $CONF,
  "feynman_grade": null,
  "contradictions_resolved": 0,
  "graph_nodes": 0,
  "files": {
    "report": "report.md",
    "graph": "graph.json",
    "citations": "citations.json",
    "contradictions": "contradictions.json",
    "index": "index.md"
  }
}
MANIFEST

# Initialize empty output files if not present
[[ -f "$OUTPUT_DIR/graph.json" ]]         || echo '{"nodes":[],"edges":[]}' > "$OUTPUT_DIR/graph.json"
[[ -f "$OUTPUT_DIR/citations.json" ]]      || echo '[]' > "$OUTPUT_DIR/citations.json"
[[ -f "$OUTPUT_DIR/contradictions.json" ]] || echo '[]' > "$OUTPUT_DIR/contradictions.json"

# Validate manifest
if ! python3 -m json.tool "$OUTPUT_DIR/manifest.json" > /dev/null 2>&1; then
  echo '{"error": "manifest.json is not valid JSON"}' >&2
  exit 1
fi

echo "{\"status\": \"exported\", \"job_id\": \"$JOB_ID\", \"output_dir\": \"$OUTPUT_DIR\"}"
log "Package assembled at: $OUTPUT_DIR"

# Fire post-export hook if present
HOOK_DIR="$(dirname "$0")/../hooks"
if [[ -x "$HOOK_DIR/post-export.sh" ]]; then
  PACKAGE_PATH="$OUTPUT_DIR" bash "$HOOK_DIR/post-export.sh" || true
fi
